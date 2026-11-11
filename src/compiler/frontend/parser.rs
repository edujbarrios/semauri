struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,
    domains: &'a DomainRegistry,
    domain_scope_stack: Vec<String>,
}

impl<'a> Parser<'a> {
    fn new(tokens: Vec<Token>, domains: &'a DomainRegistry) -> Self {
        Self {
            tokens,
            current: 0,
            domains,
            domain_scope_stack: Vec::new(),
        }
    }

    fn parse(mut self) -> Result<Program> {
        let mut statements = Vec::new();
        while !self.check(TokenKind::Eof) {
            statements.push(self.statement()?);
        }
        let span = program_span(&statements);
        Ok(Program { statements, span })
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.check(TokenKind::Word)
            && self.peek().lexeme.eq_ignore_ascii_case("import")
            && self.tokens.get(self.current + 1)
                .is_some_and(|next| next.kind == TokenKind::String)
        {
            return Err(SemauriError::parse(
                "S416",
                "File imports must be resolved from a .suri source file",
                self.peek().span,
                Some("Use 'semauri check/build/run FILE' or 'semauri test PATH'.".into()),
            ));
        }
        if self.check(TokenKind::Word)
            && self.peek().lexeme.eq_ignore_ascii_case("define")
            && self.tokens.get(self.current + 1)
                .is_some_and(|next| next.lexeme.eq_ignore_ascii_case("function"))
        {
            let start = self.advance().clone();
            return self.define_function_statement(&start);
        }
        if self.matches(&[TokenKind::Create]) {
            let start = self.previous().clone();
            return self.create_statement(&start);
        }
        if self.matches(&[TokenKind::Make]) {
            let start = self.previous().clone();
            return self.make_statement(&start);
        }
        if self.matches(&[TokenKind::Add]) {
            let start = self.previous().clone();
            return self.add_statement(&start);
        }
        if self.matches(&[TokenKind::Let]) {
            let start = self.previous().clone();
            return self.let_statement(&start);
        }
        if self.matches(&[TokenKind::Set]) {
            let start = self.previous().clone();
            return self.set_statement(&start);
        }
        if self.matches(&[TokenKind::If, TokenKind::Unless]) {
            let start = self.previous().clone();
            return self.if_statement(&start);
        }
        if self.matches(&[TokenKind::For]) {
            let start = self.previous().clone();
            return self.for_statement(&start);
        }
        if self.matches(&[TokenKind::Within]) {
            let start = self.previous().clone();
            return self.domain_scope_statement(&start);
        }
        if self.matches(&[TokenKind::DomainAction]) {
            let start = self.previous().clone();
            let expr = self.parse_registered_domain_operation(&start)?;
            self.consume_optional_dot();
            if let Expr::DomainOperation {
                domain,
                operation,
                arguments,
                span,
            } = expr
            {
                return Ok(Stmt::DomainOperation {
                    domain,
                    operation,
                    arguments,
                    span: SourceSpan::new(
                        start.span.line(),
                        start.span.column(),
                        span.end_line(),
                        if self.previous().kind == TokenKind::Dot {
                            self.previous().span.end_column()
                        } else {
                            span.end_column()
                        },
                    ),
                });
            }
        }

        Err(self.parse_error(
            self.peek(),
            "Expected a core statement or registered domain action",
            "S201",
            None,
        ))
    }

    fn define_function_statement(&mut self, start: &Token) -> Result<Stmt> {
        self.consume_surface("function", "Expected 'function' after define", "S246")?;
        let name = self.consume(TokenKind::Word, "Expected function name", "S246")?;
        self.consume_surface("taking", "Expected 'taking' after function name", "S246")?;
        let mut parameters = Vec::new();
        loop {
            let parameter = self.consume(TokenKind::Word, "Expected parameter name", "S246")?;
            self.consume_surface("as", "Expected 'as' after parameter name", "S246")?;
            let parameter_type = self.function_type()?;
            parameters.push((parameter.lexeme.to_lowercase(), parameter_type));
            if parameters.len() > 16 {
                return Err(self.parse_error(&parameter,
                    "Functions support at most 16 parameters", "S246", None));
            }
            if !self.matches(&[TokenKind::Comma]) {
                break;
            }
        }
        self.consume(TokenKind::To, "Expected 'to' before function body", "S246")?;
        let body = self.expression()?;
        self.consume_optional_dot();
        Ok(Stmt::DefineFunction {
            name: name.lexeme.to_lowercase(),
            parameters,
            body,
            span: self.span_from(start),
        })
    }

    fn function_type(&mut self) -> Result<Type> {
        if self.matches(&[TokenKind::List]) {
            self.consume(TokenKind::Of, "Expected 'of' after list type", "S246")?;
            return Ok(Type::List(Box::new(self.function_type()?)));
        }
        let token = self.peek().clone();
        let ty = match token.lexeme.to_ascii_lowercase().as_str() {
            "number" => Type::Number,
            "string" => Type::String,
            "boolean" => Type::Boolean,
            "color" => Type::Color,
            _ => return Err(self.parse_error(&token,
                "Expected parameter type number, string, boolean, color or list of TYPE", "S246", None)),
        };
        self.advance();
        Ok(ty)
    }

    fn create_statement(&mut self, start: &Token) -> Result<Stmt> {
        self.matches(&[TokenKind::Article]);
        let artifact = self.consume(
            TokenKind::DomainArtifact,
            &format!("Expected a registered artifact after '{}'", start.lexeme),
            "S202",
        )?;
        let term = self.domain_term(&artifact)?;
        let mut subject = None;
        let mut title = None;

        if self.matches(&[TokenKind::Called]) {
            title = Some(normalize_phrase(&self.phrase_until(&[
                TokenKind::Dot,
                TokenKind::Eof,
            ])?));
        } else if self.matches(&[TokenKind::For]) {
            self.matches(&[TokenKind::Article]);
            subject = Some(normalize_phrase(&self.phrase_until(&[
                TokenKind::Dot,
                TokenKind::Eof,
            ])?));
        }

        self.consume_optional_dot();
        Ok(Stmt::CreateArtifact {
            domain: term.domain,
            kind: term.kind,
            subject,
            title,
            span: self.span_from(start),
        })
    }

    fn domain_scope_statement(&mut self, start: &Token) -> Result<Stmt> {
        let domain_token = self.peek().clone();
        let domain_name = domain_token.lexeme.to_lowercase();
        if !self.domains.names().contains(&domain_name) {
            return Err(self.parse_error(
                &domain_token,
                format!("Unknown semantic domain '{}'", domain_token.lexeme),
                "S240",
                Some(format!(
                    "Available domains: {}.",
                    self.domains.names().join(", ")
                )),
            ));
        }
        self.advance();
        self.consume(
            TokenKind::Colon,
            "Expected ':' after the semantic domain name",
            "S240",
        )?;

        self.domain_scope_stack.push(domain_name.clone());
        let body_result = self.block_until(&[TokenKind::End]);
        self.domain_scope_stack.pop();
        let body = body_result?;

        self.consume(
            TokenKind::End,
            "Expected 'End' to close the semantic domain scope",
            "S240",
        )?;
        self.consume_optional_dot();
        Ok(Stmt::DomainScope {
            domain: domain_name,
            body,
            span: self.span_from(start),
        })
    }

    fn make_statement(&mut self, start: &Token) -> Result<Stmt> {
        if self.check(TokenKind::Pronoun) {
            return self.pronoun_property_statement(start);
        }
        if self.named_reference_ahead() {
            return self.named_property_statement(start);
        }
        self.create_statement(start)
    }

    fn pronoun_property_statement(&mut self, start: &Token) -> Result<Stmt> {
        let pronoun = self.advance().clone();
        let value_token = self.consume(
            TokenKind::Color,
            &format!("Expected a supported value after '{}'", pronoun.lexeme),
            "S206",
        )?;
        let (domain, property) = self.implicit_property_for(&value_token, &Type::Color)?;
        self.consume_optional_dot();
        let value = Expr::Literal {
            ty: Type::Color,
            value: LiteralValue::Color(token_string_literal(&value_token)),
            span: value_token.span,
        };
        Ok(Stmt::SetProperty {
            domain,
            target: Reference::Pronoun {
                pronoun: pronoun.lexeme,
                span: pronoun.span,
            },
            property,
            value,
            span: self.span_from(start),
        })
    }

    fn named_property_statement(&mut self, start: &Token) -> Result<Stmt> {
        self.matches(&[TokenKind::Article]);
        let element = self.consume(
            TokenKind::DomainElement,
            "Expected a registered element",
            "S207",
        )?;
        let element_term = self.domain_term(&element)?;
        self.consume(
            TokenKind::Called,
            "Expected 'called' or 'named' in an explicit reference",
            "S207",
        )?;
        let reference_tokens = self.tokens_until(&[TokenKind::Dot, TokenKind::Eof]);
        if reference_tokens.len() < 2 {
            return Err(self.parse_error(
                self.peek(),
                "Expected a name and a value",
                "S208",
                None,
            ));
        }
        let value_token = reference_tokens.last().unwrap().clone();
        if value_token.kind != TokenKind::Color {
            return Err(self.parse_error(
                &value_token,
                "Expected a supported value after the referenced element",
                "S208",
                None,
            ));
        }
        let (domain, property) = self.implicit_property_for(&value_token, &Type::Color)?;
        if domain != element_term.domain {
            return Err(self.parse_error(
                &value_token,
                format!(
                    "Implicit property belongs to domain '{}', but the target belongs to '{}'",
                    domain, element_term.domain
                ),
                "S235",
                None,
            ));
        }
        let label_tokens = &reference_tokens[..reference_tokens.len() - 1];
        let label = normalize_phrase(&phrase_from_tokens(label_tokens));
        self.consume_optional_dot();
        let reference_span = span_between_tokens(&element, label_tokens.last().unwrap());
        Ok(Stmt::SetProperty {
            domain: domain.clone(),
            target: Reference::Named {
                domain,
                kind: element_term.kind,
                label,
                span: reference_span,
            },
            property,
            value: Expr::Literal {
                ty: Type::Color,
                value: LiteralValue::Color(token_string_literal(&value_token)),
                span: value_token.span,
            },
            span: self.span_from(start),
        })
    }

    fn let_statement(&mut self, start: &Token) -> Result<Stmt> {
        // 'first' and 'last' are contextual prefix operators, so existing
        // programs may still use them as immutable variable names.
        let name = if self.matches(&[TokenKind::First, TokenKind::Last]) {
            self.previous().clone()
        } else {
            self.consume(
                TokenKind::Word,
                "Expected a variable name after 'Let'",
                "S209",
            )?
        };
        self.consume(
            TokenKind::Be,
            "Expected 'be' after the variable name",
            "S210",
        )?;
        let value = self.expression()?;
        self.consume_optional_dot();
        Ok(Stmt::Let {
            name: name.lexeme.to_lowercase(),
            value,
            span: self.span_from(start),
        })
    }

    fn set_statement(&mut self, start: &Token) -> Result<Stmt> {
        self.matches(&[TokenKind::Article]);
        let property_token = self.consume(
            TokenKind::DomainProperty,
            "Expected a registered property after 'Set'",
            "S211",
        )?;
        let property_term = self.domain_term(&property_token)?;
        self.consume(
            TokenKind::Of,
            "Expected 'of' after the property name",
            "S212",
        )?;
        let target = self.canonical_reference(&property_term.domain)?;
        self.consume(
            TokenKind::To,
            "Expected 'to' before the new value",
            "S213",
        )?;
        let value = self.expression()?;
        self.consume_optional_dot();
        Ok(Stmt::SetProperty {
            domain: property_term.domain,
            target,
            property: property_term.kind,
            value,
            span: self.span_from(start),
        })
    }

    fn if_statement(&mut self, start: &Token) -> Result<Stmt> {
        let condition = self.expression()?;
        // "Unless X" is "If not X"; keep a single typed HIR and runtime path.
        let condition = if start.kind == TokenKind::Unless {
            let span = span_between_spans(start.span, condition.span());
            Expr::Unary {
                operator: UnaryOperator::Not,
                operand: Box::new(condition),
                span,
            }
        } else {
            condition
        };
        // In plain-English conditionals, "then" is an optional delimiter.
        self.matches_surface("then");
        self.consume(
            TokenKind::Colon,
            "Expected ':' after the if condition",
            "S218",
        )?;
        let consequence = self.block_until(&[TokenKind::Otherwise, TokenKind::End])?;
        let mut branches = vec![(start.span, condition, consequence)];
        let mut otherwise = None;

        // Chained conditionals share one End. Lower the chain into nested
        // If nodes so existing type checks, scopes and optimizer rules apply.
        while self.matches(&[TokenKind::Otherwise]) {
            // 'elif' is a single-token spelling of 'otherwise if'.
            let is_elif = self.previous().lexeme.eq_ignore_ascii_case("elif");
            if is_elif || self.matches(&[TokenKind::If]) {
                let branch_start = self.previous().span;
                let condition = self.expression()?;
                self.matches_surface("then");
                self.consume(
                    TokenKind::Colon,
                    "Expected ':' after the if condition",
                    "S218",
                )?;
                let consequence = self.block_until(&[TokenKind::Otherwise, TokenKind::End])?;
                branches.push((branch_start, condition, consequence));
            } else {
                self.consume(
                    TokenKind::Colon,
                    "Expected ':' after 'Otherwise'",
                    "S219",
                )?;
                otherwise = Some(self.block_until(&[TokenKind::End])?);
                break;
            }
        }

        self.consume(
            TokenKind::End,
            "Expected 'End' to close the If block",
            "S220",
        )?;
        self.consume_optional_dot();

        let end = self.previous().span;
        let mut nested: Option<(Stmt, SourceSpan)> = None;
        for (start_span, condition, consequence) in branches.into_iter().rev() {
            let span = SourceSpan::new(
                start_span.line(),
                start_span.column(),
                end.end_line(),
                end.end_column(),
            );
            let alternative = if let Some((statement, nested_span)) = nested.take() {
                Some(Block {
                    statements: vec![statement],
                    span: nested_span,
                })
            } else {
                otherwise.take()
            };
            nested = Some((
                Stmt::If {
                    condition,
                    consequence,
                    alternative,
                    span,
                },
                span,
            ));
        }
        // The initial If guarantees at least one branch.
        Ok(nested.expect("an If has at least one branch").0)
    }

    fn for_statement(&mut self, start: &Token) -> Result<Stmt> {
        // Natural shorthand: both 'for every item in items' and
        // Python-like 'for item in items' bind the same typed iterator.
        self.matches(&[TokenKind::Every]);
        let variable = self.consume(
            TokenKind::Word,
            "Expected an iteration variable after 'For every'",
            "S229",
        )?;
        self.consume(
            TokenKind::In,
            "Expected 'in' after the iteration variable",
            "S230",
        )?;
        let iterable = self.expression()?;
        // Optional 'do' makes the loop read as an English sentence.
        self.matches_surface("do");
        self.consume(
            TokenKind::Colon,
            "Expected ':' after the iterable expression",
            "S231",
        )?;
        let body = self.block_until(&[TokenKind::End])?;
        self.consume(
            TokenKind::End,
            "Expected 'End' to close the For block",
            "S232",
        )?;
        self.consume_optional_dot();
        Ok(Stmt::ForEach {
            variable_name: variable.lexeme.to_lowercase(),
            binding_span: variable.span,
            iterable,
            body,
            span: self.span_from(start),
        })
    }

    fn block_until(&mut self, terminators: &[TokenKind]) -> Result<Block> {
        let mut statements = Vec::new();
        while !terminators.contains(&self.peek().kind) && !self.check(TokenKind::Eof) {
            statements.push(self.statement()?);
        }
        if statements.is_empty() {
            return Err(self.parse_error(
                self.peek(),
                "Expected at least one statement in the block",
                "S221",
                None,
            ));
        }
        let span = program_span(&statements);
        Ok(Block { statements, span })
    }

    fn expression(&mut self) -> Result<Expr> {
        self.logical_or()
    }

    fn logical_or(&mut self) -> Result<Expr> {
        let mut expression = self.logical_and()?;
        while self.matches(&[TokenKind::Or]) {
            let right = self.logical_and()?;
            let span = span_between_spans(expression.span(), right.span());
            expression = Expr::Binary {
                left: Box::new(expression),
                operator: BinaryOperator::Or,
                right: Box::new(right),
                span,
            };
        }
        Ok(expression)
    }

    fn logical_and(&mut self) -> Result<Expr> {
        let mut expression = self.logical_not()?;
        while self.matches(&[TokenKind::And]) {
            let right = self.logical_not()?;
            let span = span_between_spans(expression.span(), right.span());
            expression = Expr::Binary {
                left: Box::new(expression),
                operator: BinaryOperator::And,
                right: Box::new(right),
                span,
            };
        }
        Ok(expression)
    }

    fn logical_not(&mut self) -> Result<Expr> {
        if self.matches(&[TokenKind::Not]) {
            let operator = self.previous().clone();
            let operand = self.logical_not()?;
            let span = span_between_spans(operator.span, operand.span());
            return Ok(Expr::Unary {
                operator: UnaryOperator::Not,
                operand: Box::new(operand),
                span,
            });
        }
        self.comparison()
    }

    fn comparison(&mut self) -> Result<Expr> {
        let left = self.concatenative()?;
        // Conversational negation: 'does not start/end with' for text and
        // 'does not contain/include' for strings or typed lists. Reuse the
        // existing typed binary operators and boolean Not, without re-running
        // either operand or introducing new runtime operations.
        if self.check(TokenKind::Word)
            && self.peek().lexeme.eq_ignore_ascii_case("does")
            && self.tokens.get(self.current + 1)
                .is_some_and(|next| next.kind == TokenKind::Not)
            && self.tokens.get(self.current + 2).is_some_and(|next|
                matches!(next.lexeme.to_ascii_lowercase().as_str(),
                    "start" | "end" | "begin" | "finish" | "contain" | "include"))
        {
            self.advance(); // does
            self.advance(); // not
            let verb = self.advance().lexeme.to_ascii_lowercase();
            let operator = match verb.as_str() {
                "start" | "begin" => BinaryOperator::StartsWith,
                "end" | "finish" => BinaryOperator::EndsWith,
                _ => BinaryOperator::Contains,
            };
            if matches!(verb.as_str(), "start" | "end" | "begin" | "finish") {
                if !self.check(TokenKind::Word)
                    || !self.peek().lexeme.eq_ignore_ascii_case("with")
                {
                    return Err(self.parse_error(self.peek(),
                        "Expected 'with' after 'does not start/end/begin/finish'", "S225", None));
                }
                self.advance(); // with
            }
            let right = self.concatenative()?;
            let span = span_between_spans(left.span(), right.span());
            let predicate = Expr::Binary {
                left: Box::new(left), operator, right: Box::new(right), span,
            };
            return Ok(Expr::Unary {
                operator: UnaryOperator::Not,
                operand: Box::new(predicate), span,
            });
        }
        // 'includes' and 'excludes' are typed contains/not-contains aliases.
        if self.check(TokenKind::Word)
            && matches!(self.peek().lexeme.to_ascii_lowercase().as_str(),
                "includes" | "excludes")
        {
            let negated = self.advance().lexeme.eq_ignore_ascii_case("excludes");
            let right = self.concatenative()?;
            let span = span_between_spans(left.span(), right.span());
            let predicate = Expr::Binary {
                left: Box::new(left), operator: BinaryOperator::Contains,
                right: Box::new(right), span,
            };
            return Ok(if negated {
                Expr::Unary { operator: UnaryOperator::Not,
                    operand: Box::new(predicate), span }
            } else { predicate });
        }
        // Natural-language predicates are contextual pairs, not reserved words.
        if self.check(TokenKind::Word)
            && matches!(self.peek().lexeme.to_ascii_lowercase().as_str(),
                "starts" | "ends" | "begins" | "finishes")
            && self.tokens.get(self.current + 1).is_some_and(|next|
                next.kind == TokenKind::Word && next.lexeme.eq_ignore_ascii_case("with"))
        {
            let operator = if matches!(
                self.advance().lexeme.to_ascii_lowercase().as_str(),
                "starts" | "begins") {
                BinaryOperator::StartsWith
            } else {
                BinaryOperator::EndsWith
            };
            self.advance(); // contextual 'with'
            let right = self.concatenative()?;
            return Ok(Expr::Binary {
                span: span_between_spans(left.span(), right.span()),
                left: Box::new(left),
                operator,
                right: Box::new(right),
            });
        }
        if self.matches(&[TokenKind::Contains, TokenKind::At]) {
            let operator = if self.previous().kind == TokenKind::Contains {
                BinaryOperator::Contains
            } else {
                BinaryOperator::AtIndex
            };
            let right = self.concatenative()?;
            let span = span_between_spans(left.span(), right.span());
            return Ok(Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                span,
            });
        }
        // Python-style membership, with an English "is in" variant.
        // Keep the existing 'contains' semantic and optimizer paths by
        // reversing operands: item in collection => collection contains item.
        if self.matches(&[TokenKind::In]) {
            return self.membership_expression(left, false);
        }
        if self.check(TokenKind::Not)
            && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::In)
        {
            self.advance(); // not
            self.advance(); // in
            return self.membership_expression(left, true);
        }
        if !self.matches(&[TokenKind::Is]) {
            return Ok(left);
        }
        if self.matches(&[TokenKind::In]) {
            return self.membership_expression(left, false);
        }
        if self.check(TokenKind::Not)
            && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::In)
        {
            self.advance(); // not
            self.advance(); // in
            return self.membership_expression(left, true);
        }

        // "item is one of collection" is a natural spelling of the typed
        // membership operator. Keep 'one' contextual instead of reserving it.
        let one_of = self.check(TokenKind::Word)
            && self.peek().lexeme.eq_ignore_ascii_case("one")
            && self.tokens.get(self.current + 1)
                .is_some_and(|next| next.kind == TokenKind::Of);
        let not_one_of = self.check(TokenKind::Not)
            && self.tokens.get(self.current + 1).is_some_and(|next|
                next.kind == TokenKind::Word && next.lexeme.eq_ignore_ascii_case("one"))
            && self.tokens.get(self.current + 2)
                .is_some_and(|next| next.kind == TokenKind::Of);
        if one_of || not_one_of {
            if not_one_of { self.advance(); } // not
            self.advance(); // one
            self.advance(); // of
            return self.membership_expression(left, not_one_of);
        }

        // Boolean words after 'is' are checked equality predicates, and
        // only accept bool values. Ordinary identifiers are unaffected.
        let not_boolean = self.check(TokenKind::Not)
            && self.tokens.get(self.current + 1)
                .is_some_and(|next| next.kind == TokenKind::Boolean);
        if self.check(TokenKind::Boolean) || not_boolean {
            if not_boolean { self.advance(); }
            let token = self.advance().clone();
            let span = span_between_spans(left.span(), token.span);
            return Ok(Expr::Binary {
                left: Box::new(left),
                operator: if not_boolean { BinaryOperator::NotEqual } else { BinaryOperator::Equal },
                right: Box::new(Expr::Literal {
                    ty: Type::Boolean,
                    value: LiteralValue::Boolean(token.lexeme.eq_ignore_ascii_case("true")),
                    span: token.span,
                }),
                span,
            });
        }

        // Natural mathematical divisibility predicates, which reuse checked
        // Modulo and typed numeric equality. Keep the words contextual.
        let negated_divisibility = self.check(TokenKind::Not);
        let offset = usize::from(negated_divisibility);
        let divisible = self.tokens.get(self.current + offset).is_some_and(|next|
            next.kind == TokenKind::Word && next.lexeme.eq_ignore_ascii_case("divisible"))
            && self.tokens.get(self.current + offset + 1)
                .is_some_and(|next| next.kind == TokenKind::By);
        let multiple = self.tokens.get(self.current + offset)
            .is_some_and(|next| next.kind == TokenKind::Article)
            && self.tokens.get(self.current + offset + 1).is_some_and(|next|
                next.kind == TokenKind::Word && next.lexeme.eq_ignore_ascii_case("multiple"))
            && self.tokens.get(self.current + offset + 2)
                .is_some_and(|next| next.kind == TokenKind::Of);
        if divisible || multiple {
            if negated_divisibility { self.advance(); } // not
            if divisible { self.advance(); self.advance(); } // divisible by
            else { self.advance(); self.advance(); self.advance(); } // a multiple of
            let divisor = self.concatenative()?;
            let span = span_between_spans(left.span(), divisor.span());
            let remainder = Expr::Binary {
                left: Box::new(left),
                operator: BinaryOperator::Modulo,
                right: Box::new(divisor),
                span,
            };
            return Ok(Expr::Binary {
                left: Box::new(remainder),
                operator: if negated_divisibility { BinaryOperator::NotEqual } else { BinaryOperator::Equal },
                right: Box::new(Expr::Literal {
                    ty: Type::Number,
                    value: LiteralValue::Number(NumberValue::Int(0)),
                    span,
                }),
                span,
            });
        }

        // Natural emptiness and whitespace predicates. Lower them to
        // existing typed Trim, Length, Equal and NotEqual operators so
        // their type safety, effects and optimizer remain unchanged.
        let negated = self.check(TokenKind::Not)
            && self.tokens.get(self.current + 1).is_some_and(|next|
                next.kind == TokenKind::Word
                && matches!(next.lexeme.to_ascii_lowercase().as_str(), "empty" | "blank"));
        let positive = self.check(TokenKind::Word)
            && matches!(self.peek().lexeme.to_ascii_lowercase().as_str(), "empty" | "blank");
        if negated || positive {
            if negated { self.advance(); }
            let predicate = self.advance().clone();
            return Ok(self.empty_predicate_expression(
                left, predicate.lexeme.eq_ignore_ascii_case("blank"), negated,
                predicate.span,
            ));
        }

        // Numeric and parity predicates are read like English, and desugar
        // to the already checked Sign/Even/Odd and comparison expressions.
        // In particular Sign gives an exact integer zero for decimal 0.0.
        let is_not_predicate = self.check(TokenKind::Not)
            && self.tokens.get(self.current + 1).is_some_and(|next|
                next.kind == TokenKind::Word && matches!(
                    next.lexeme.to_ascii_lowercase().as_str(),
                    "zero" | "positive" | "negative" | "nonzero" | "nonnegative" | "nonpositive" | "even" | "odd"));
        let is_predicate = self.check(TokenKind::Word) && matches!(
            self.peek().lexeme.to_ascii_lowercase().as_str(),
            "zero" | "positive" | "negative" | "nonzero" | "nonnegative" | "nonpositive" | "even" | "odd");
        if is_not_predicate || is_predicate {
            if is_not_predicate { self.advance(); }
            let keyword = self.advance().clone();
            return Ok(self.numeric_predicate_expression(
                left, keyword.lexeme.to_ascii_lowercase().as_str(),
                is_not_predicate, keyword.span,
            ));
        }

        // Natural prime predicates delegate to the existing checked pure
        // prime operator. In particular, invalid fractional inputs still
        // report the same S436 diagnostic as 'prime of'.
        let is_not_prime = self.check(TokenKind::Not)
            && self.tokens.get(self.current + 1).is_some_and(|next|
                next.kind == TokenKind::Word && next.lexeme.eq_ignore_ascii_case("prime"));
        let is_prime = self.check(TokenKind::Word)
            && self.peek().lexeme.eq_ignore_ascii_case("prime");
        if is_not_prime || is_prime {
            if is_not_prime { self.advance(); }
            let word = self.advance().clone();
            let span = span_between_spans(left.span(), word.span);
            let checked = Expr::Unary {
                operator: UnaryOperator::Prime,
                operand: Box::new(left), span,
            };
            return Ok(if is_not_prime {
                Expr::Unary { operator: UnaryOperator::Not,
                    operand: Box::new(checked), span }
            } else { checked });
        }

        // Negated English bounds map to the complementary checked numeric
        // ordering operator. This evaluates each operand exactly once and
        // preserves precise signed-i64 comparisons and S314 type errors.
        let not_at_bound = self.check(TokenKind::Not)
            && self.tokens.get(self.current + 1)
                .is_some_and(|next| next.kind == TokenKind::At)
            && self.tokens.get(self.current + 2).is_some_and(|next|
                next.kind == TokenKind::Word && matches!(
                    next.lexeme.to_ascii_lowercase().as_str(), "least" | "most"));
        let not_simple_bound = self.check(TokenKind::Not)
            && self.tokens.get(self.current + 1).is_some_and(|next|
                next.kind == TokenKind::Word && matches!(
                    next.lexeme.to_ascii_lowercase().as_str(), "above" | "below"));
        // Natural negated comparisons are complements of the checked
        // ordering operators. This evaluates both operands exactly once and
        // preserves precise integer comparisons and numeric-only typing.
        // "no" stays contextual, so it remains a usable identifier elsewhere.
        let is_negated_order = (self.check(TokenKind::Not)
            || (self.check(TokenKind::Word)
                && self.peek().lexeme.eq_ignore_ascii_case("no")))
            && self.tokens.get(self.current + 1).is_some_and(|next|
                matches!(next.kind, TokenKind::Greater | TokenKind::Less));
        let operator = if is_negated_order {
            self.advance(); // not / contextual no
            let greater = self.matches(&[TokenKind::Greater]);
            if !greater {
                self.consume(TokenKind::Less, "Expected 'less' after negation", "S223")?;
            }
            self.consume(TokenKind::Than, "Expected 'than' in negated comparison",
                if greater { "S222" } else { "S223" })?;
            let inclusive = if self.matches(&[TokenKind::Or]) {
                self.consume(TokenKind::Equal,
                    "Expected 'equal' after 'or' in a negated comparison",
                    if greater { "S222" } else { "S223" })?;
                self.consume(TokenKind::To,
                    "Expected 'to' after 'equal' in a negated comparison",
                    if greater { "S222" } else { "S223" })?;
                true
            } else { false };
            match (greater, inclusive) {
                (true, false) => BinaryOperator::LessThanOrEqual,
                (true, true) => BinaryOperator::LessThan,
                (false, false) => BinaryOperator::GreaterThanOrEqual,
                (false, true) => BinaryOperator::GreaterThan,
            }
        } else if not_at_bound || not_simple_bound {
            self.advance(); // not
            if not_at_bound {
                self.advance(); // at
                let bound = self.advance().lexeme.to_ascii_lowercase();
                if bound == "least" { BinaryOperator::LessThan }
                else { BinaryOperator::GreaterThan }
            } else {
                let bound = self.advance().lexeme.to_ascii_lowercase();
                if bound == "above" { BinaryOperator::LessThanOrEqual }
                else { BinaryOperator::GreaterThanOrEqual }
            }
        } else if self.matches(&[TokenKind::Greater]) {
            self.consume(
                TokenKind::Than,
                "Expected 'than' after 'greater'",
                "S222",
            )?;
            if self.matches(&[TokenKind::Or]) {
                self.consume(
                    TokenKind::Equal,
                    "Expected 'equal' after 'or' in an inclusive comparison",
                    "S222",
                )?;
                self.consume(
                    TokenKind::To,
                    "Expected 'to' after 'equal' in an inclusive comparison",
                    "S222",
                )?;
                BinaryOperator::GreaterThanOrEqual
            } else {
                BinaryOperator::GreaterThan
            }
        } else if self.matches(&[TokenKind::Less]) {
            self.consume(
                TokenKind::Than,
                "Expected 'than' after 'less'",
                "S223",
            )?;
            if self.matches(&[TokenKind::Or]) {
                self.consume(
                    TokenKind::Equal,
                    "Expected 'equal' after 'or' in an inclusive comparison",
                    "S223",
                )?;
                self.consume(
                    TokenKind::To,
                    "Expected 'to' after 'equal' in an inclusive comparison",
                    "S223",
                )?;
                BinaryOperator::LessThanOrEqual
            } else {
                BinaryOperator::LessThan
            }
        } else if self.check(TokenKind::At) && self.tokens.get(self.current + 1)
            .is_some_and(|next| next.kind == TokenKind::Word && matches!(
                next.lexeme.to_ascii_lowercase().as_str(), "least" | "most")) {
            // 'is at least' / 'is at most' map directly to checked inclusive
            // ordering comparisons, without evaluating the left side twice.
            self.advance(); // at
            let bound = self.advance().lexeme.to_ascii_lowercase();
            if bound == "least" { BinaryOperator::GreaterThanOrEqual }
            else { BinaryOperator::LessThanOrEqual }
        } else if self.check(TokenKind::Word) && matches!(
            self.peek().lexeme.to_ascii_lowercase().as_str(), "above" | "below") {
            if self.advance().lexeme.eq_ignore_ascii_case("above") {
                BinaryOperator::GreaterThan
            } else { BinaryOperator::LessThan }
        } else if self.matches(&[TokenKind::Equal]) {
            self.consume(
                TokenKind::To,
                "Expected 'to' after 'equal'",
                "S224",
            )?;
            BinaryOperator::Equal
        } else if self.check(TokenKind::Word)
            && self.peek().lexeme.eq_ignore_ascii_case("different")
        {
            self.advance();
            self.consume(TokenKind::From, "Expected 'from' after 'is different'", "S224")?;
            BinaryOperator::NotEqual
        } else if self.matches(&[TokenKind::Not]) {
            self.consume(
                TokenKind::Equal,
                "Expected 'equal' after 'not'",
                "S224",
            )?;
            self.consume(
                TokenKind::To,
                "Expected 'to' after 'not equal'",
                "S224",
            )?;
            BinaryOperator::NotEqual
        } else {
            return Err(self.parse_error(
                self.peek(),
                "Expected a comparison after 'is' (greater than, less than, at least, at most, above, below, equal to, or not equal to)",
                "S225",
                Some(
                    "Ordering comparisons may add 'or equal to', for example 'is greater than or equal to'."
                        .to_string(),
                ),
            ));
        };

        let right = self.concatenative()?;
        let span = span_between_spans(left.span(), right.span());
        Ok(Expr::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
            span,
        })
    }

    // Contextual natural-language concatenation, left associative.
    fn concatenative(&mut self) -> Result<Expr> {
        let mut expression = self.additive()?;
        while self.check(TokenKind::Word)
            && self.peek().lexeme.eq_ignore_ascii_case("joined")
            && self.tokens.get(self.current + 1).is_some_and(|next|
                next.kind == TokenKind::Word && next.lexeme.eq_ignore_ascii_case("with"))
        {
            self.advance(); // joined
            self.advance(); // with
            let right = self.additive()?;
            let span = span_between_spans(expression.span(), right.span());
            expression = Expr::Binary {
                left: Box::new(expression),
                operator: BinaryOperator::Concat,
                right: Box::new(right),
                span,
            };
        }
        Ok(expression)
    }

    // Reuse the typed sign and parity operations; this also avoids
    // loss of precision and Int-vs-Float equality differences at zero.
    fn numeric_predicate_expression(
        &self,
        value: Expr,
        predicate: &str,
        negated: bool,
        end: SourceSpan,
    ) -> Expr {
        let span = span_between_spans(value.span(), end);
        if matches!(predicate, "even" | "odd") {
            let parity = Expr::Unary {
                operator: if predicate == "even" { UnaryOperator::Even } else { UnaryOperator::Odd },
                operand: Box::new(value),
                span,
            };
            return if negated {
                Expr::Unary { operator: UnaryOperator::Not, operand: Box::new(parity), span }
            } else { parity };
        }
        let sign = Expr::Unary {
            operator: UnaryOperator::Sign,
            operand: Box::new(value),
            span,
        };
        let operator = match (predicate, negated) {
            ("zero", false) => BinaryOperator::Equal,
            ("zero", true) => BinaryOperator::NotEqual,
            ("positive", false) => BinaryOperator::GreaterThan,
            ("positive", true) => BinaryOperator::LessThanOrEqual,
            ("negative", false) => BinaryOperator::LessThan,
            ("negative", true) => BinaryOperator::GreaterThanOrEqual,
            ("nonzero", false) => BinaryOperator::NotEqual,
            ("nonzero", true) => BinaryOperator::Equal,
            ("nonnegative", false) => BinaryOperator::GreaterThanOrEqual,
            ("nonnegative", true) => BinaryOperator::LessThan,
            ("nonpositive", false) => BinaryOperator::LessThanOrEqual,
            ("nonpositive", true) => BinaryOperator::GreaterThan,
            _ => unreachable!("guard selects numeric predicates"),
        };
        Expr::Binary {
            left: Box::new(sign),
            operator,
            right: Box::new(Expr::Literal {
                ty: Type::Number,
                value: LiteralValue::Number(NumberValue::Int(0)),
                span: end,
            }),
            span,
        }
    }

    // Desugar natural predicates through the existing typed operations.
    // 'empty' accepts strings/lists; 'blank' accepts only strings and ignores
    // surrounding Unicode whitespace through the existing trim operator.
    fn empty_predicate_expression(
        &self,
        value: Expr,
        trim_first: bool,
        negated: bool,
        end: SourceSpan,
    ) -> Expr {
        let span = span_between_spans(value.span(), end);
        let operand = if trim_first {
            Expr::Unary {
                operator: UnaryOperator::Trim,
                operand: Box::new(value),
                span,
            }
        } else {
            value
        };
        let length = Expr::Unary {
            operator: UnaryOperator::Length,
            operand: Box::new(operand),
            span,
        };
        Expr::Binary {
            left: Box::new(length),
            operator: if negated { BinaryOperator::NotEqual } else { BinaryOperator::Equal },
            right: Box::new(Expr::Literal {
                ty: Type::Number,
                value: LiteralValue::Number(NumberValue::Int(0)),
                span: end,
            }),
            span,
        }
    }

    // No new runtime operator: membership reuses typed Contains and Not.
    fn membership_expression(&mut self, item: Expr, negated: bool) -> Result<Expr> {
        let collection = self.concatenative()?;
        let span = span_between_spans(item.span(), collection.span());
        let contains = Expr::Binary {
            left: Box::new(collection),
            operator: BinaryOperator::Contains,
            right: Box::new(item),
            span,
        };
        if negated {
            Ok(Expr::Unary { operator: UnaryOperator::Not, operand: Box::new(contains), span })
        } else {
            Ok(contains)
        }
    }

    fn additive(&mut self) -> Result<Expr> {
        let mut expression = self.multiplicative()?;
        while self.matches(&[TokenKind::Plus, TokenKind::Minus]) {
            let operator = if self.previous().kind == TokenKind::Plus {
                BinaryOperator::Add
            } else {
                BinaryOperator::Subtract
            };
            let right = self.multiplicative()?;
            let span = span_between_spans(expression.span(), right.span());
            expression = Expr::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
                span,
            };
        }
        Ok(expression)
    }

    fn multiplicative(&mut self) -> Result<Expr> {
        let mut expression = self.numeric_unary()?;
        loop {
            if self.matches(&[TokenKind::Times]) {
                let right = self.numeric_unary()?;
                let span = span_between_spans(expression.span(), right.span());
                expression = Expr::Binary {
                    left: Box::new(expression),
                    operator: BinaryOperator::Multiply,
                    right: Box::new(right),
                    span,
                };
            } else if self.matches(&[TokenKind::Divided]) {
                self.consume(
                    TokenKind::By,
                    "Expected 'by' after 'divided'",
                    "S226",
                )?;
                let right = self.numeric_unary()?;
                let span = span_between_spans(expression.span(), right.span());
                expression = Expr::Binary {
                    left: Box::new(expression),
                    operator: BinaryOperator::Divide,
                    right: Box::new(right),
                    span,
                };
            } else if self.matches(&[TokenKind::Modulo]) {
                let right = self.numeric_unary()?;
                let span = span_between_spans(expression.span(), right.span());
                expression = Expr::Binary {
                    left: Box::new(expression),
                    operator: BinaryOperator::Modulo,
                    right: Box::new(right),
                    span,
                };
            } else {
                break;
            }
        }
        Ok(expression)
    }

    fn numeric_unary(&mut self) -> Result<Expr> {
        if self.matches(&[TokenKind::Minus]) {
            let operator = self.previous().clone();
            let operand = self.numeric_unary()?;
            let span = span_between_spans(operator.span, operand.span());
            return Ok(Expr::Unary {
                operator: UnaryOperator::Negate,
                operand: Box::new(operand),
                span,
            });
        }
        self.primary()
    }

    fn numeric_series_expression(&mut self) -> Result<Expr> {
        let start = self.advance().clone();
        let operation = self.advance().clone();
        self.consume(TokenKind::Of,
            "Expected 'of' after the numeric series operation", "S242")?;
        let operand = self.primary()?;
        let span = span_between_spans(start.span, operand.span());
        let operator = if start.lexeme.eq_ignore_ascii_case("adjacent")
            || start.lexeme.eq_ignore_ascii_case("successive") {
            UnaryOperator::AdjacentDifferences
        } else if operation.kind == TokenKind::Average {
            UnaryOperator::RunningAverage
        } else if operation.kind == TokenKind::Minimum
            || operation.lexeme.eq_ignore_ascii_case("min") {
            UnaryOperator::RunningMinimum
        } else if operation.kind == TokenKind::Maximum
            || operation.lexeme.eq_ignore_ascii_case("max") {
            UnaryOperator::RunningMaximum
        } else if operation.lexeme.eq_ignore_ascii_case("product") {
            UnaryOperator::RunningProduct
        } else {
            UnaryOperator::RunningTotal
        };
        Ok(Expr::Unary { operator, operand: Box::new(operand), span })
    }

    fn primary(&mut self) -> Result<Expr> {
        let token = self.peek().clone();
        match token.kind {
            // Deduplicate and sort a typed list in one expression. Composing
            // existing operators retains storage-aware distinct semantics,
            // exact numeric ordering and one-time evaluation of the source.
            kind if (kind == TokenKind::Sorted
                || (kind == TokenKind::Word
                    && token.lexeme.eq_ignore_ascii_case("sort")))
                && self.tokens.get(self.current + 1).is_some_and(|next|
                    next.kind == TokenKind::Distinct
                    || (next.kind == TokenKind::Word
                        && next.lexeme.eq_ignore_ascii_case("unique")))
                && self.tokens.get(self.current + 2)
                    .is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.advance(); // distinct / unique
                self.consume(TokenKind::Of,
                    "Expected 'of' after sorted distinct/unique", "S242")?;
                let operand = self.primary()?;
                let span = span_between_spans(start.span, operand.span());
                let deduplicated = Expr::Unary {
                    operator: UnaryOperator::Distinct,
                    operand: Box::new(operand),
                    span,
                };
                Ok(Expr::Unary {
                    operator: UnaryOperator::Sorted,
                    operand: Box::new(deduplicated),
                    span,
                })
            }
            // Optional sort order keeps all of Sorted's static checks and exact
            // numeric comparisons. Descending wraps Sorted in Reverse, so the
            // list operand is evaluated exactly once (even if effectful).
            kind if (kind == TokenKind::Sorted
                || (kind == TokenKind::Word
                    && token.lexeme.eq_ignore_ascii_case("sort")))
                && self.tokens.get(self.current + 1).is_some_and(|next|
                    next.kind == TokenKind::Word
                    && matches!(next.lexeme.to_ascii_lowercase().as_str(),
                        "ascending" | "descending"))
                && self.tokens.get(self.current + 2)
                    .is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                let descending = self.advance().lexeme.eq_ignore_ascii_case("descending");
                self.consume(TokenKind::Of, "Expected 'of' after sort order", "S242")?;
                let operand = self.primary()?;
                let span = span_between_spans(start.span, operand.span());
                let sorted = Expr::Unary {
                    operator: UnaryOperator::Sorted,
                    operand: Box::new(operand),
                    span,
                };
                Ok(if descending {
                    Expr::Unary {
                        operator: UnaryOperator::Reverse,
                        operand: Box::new(sorted),
                        span,
                    }
                } else {
                    sorted
                })
            }
            // Count unique typed values without allocating a second copy of
            // the source expression. Both operators retain their HIR checks,
            // distinct-value semantics, and constant-folding behavior.
            TokenKind::Word
                if (token.lexeme.eq_ignore_ascii_case("cardinality")
                    && self.tokens.get(self.current + 1)
                        .is_some_and(|next| next.kind == TokenKind::Of))
                    || (token.lexeme.eq_ignore_ascii_case("count")
                        && self.tokens.get(self.current + 1).is_some_and(|next|
                            next.kind == TokenKind::Distinct
                            || (next.kind == TokenKind::Word
                                && next.lexeme.eq_ignore_ascii_case("unique")))
                        && self.tokens.get(self.current + 2)
                            .is_some_and(|next| next.kind == TokenKind::Of)) => {
                let start = self.advance().clone();
                if start.lexeme.eq_ignore_ascii_case("count") {
                    self.advance(); // contextual distinct / unique
                }
                self.consume(TokenKind::Of, "Expected 'of' after unique count", "S242")?;
                let operand = self.primary()?;
                let span = span_between_spans(start.span, operand.span());
                let unique = Expr::Unary {
                    operator: UnaryOperator::Distinct,
                    operand: Box::new(operand),
                    span,
                };
                Ok(Expr::Unary {
                    operator: UnaryOperator::Length,
                    operand: Box::new(unique),
                    span,
                })
            }
            // Numeric distance variants are contextual natural-language forms:
            // difference between A and B, absolute difference between A and B,
            // and squared difference between A and B. Lower to the existing
            // checked operators so both inputs are evaluated exactly once.
            TokenKind::Word
                if (token.lexeme.eq_ignore_ascii_case("difference")
                    && self.tokens.get(self.current + 1).is_some_and(|next|
                        next.lexeme.eq_ignore_ascii_case("between")))
                    || (matches!(token.lexeme.to_ascii_lowercase().as_str(),
                        "absolute" | "squared")
                        && self.tokens.get(self.current + 1).is_some_and(|next|
                            next.kind == TokenKind::Word
                            && next.lexeme.eq_ignore_ascii_case("difference"))
                        && self.tokens.get(self.current + 2).is_some_and(|next|
                            next.lexeme.eq_ignore_ascii_case("between"))) => {
                let start = self.advance().clone();
                let mode = start.lexeme.to_ascii_lowercase();
                if mode != "difference" {
                    self.advance(); // contextual "difference" after absolute/squared
                }
                self.consume_surface(
                    "between", "Expected 'between' after difference", "S252"
                )?;
                let left = self.numeric_unary()?;
                self.consume(
                    TokenKind::And, "Expected 'and' between numeric operands", "S252"
                )?;
                let right = self.numeric_unary()?;
                let span = span_between_spans(start.span, right.span());
                let difference = Expr::Binary {
                    left: Box::new(left),
                    operator: BinaryOperator::Subtract,
                    right: Box::new(right),
                    span,
                };
                Ok(match mode.as_str() {
                    "absolute" => Expr::Unary {
                        operator: UnaryOperator::Absolute,
                        operand: Box::new(difference),
                        span,
                    },
                    "squared" => Expr::Unary {
                        operator: UnaryOperator::Square,
                        operand: Box::new(difference),
                        span,
                    },
                    _ => difference,
                })
            }
            // Checked squared-error metrics for residual lists. These
            // contextual names reuse the exact same map and aggregation as
            // 'mean square' and 'root mean square'. No separate arithmetic
            // or second evaluation of the input list is introduced.
            kind if (kind == TokenKind::Sum
                    && self.tokens.get(self.current + 1).is_some_and(|next|
                        next.kind == TokenKind::Of)
                    && self.tokens.get(self.current + 2).is_some_and(|next|
                        next.kind == TokenKind::Word
                        && next.lexeme.eq_ignore_ascii_case("squares"))
                    && self.tokens.get(self.current + 3).is_some_and(|next|
                        next.kind == TokenKind::Of))
                || (kind == TokenKind::Word
                    && ((token.lexeme.eq_ignore_ascii_case("mean")
                        && ((self.tokens.get(self.current + 1).is_some_and(|next|
                            next.lexeme.eq_ignore_ascii_case("square"))
                            && self.tokens.get(self.current + 2).is_some_and(|next|
                                next.kind == TokenKind::Of))
                            || (self.tokens.get(self.current + 1).is_some_and(|next|
                                next.lexeme.eq_ignore_ascii_case("squared"))
                                && self.tokens.get(self.current + 2).is_some_and(|next|
                                    next.lexeme.eq_ignore_ascii_case("error"))
                                && self.tokens.get(self.current + 3).is_some_and(|next|
                                    next.kind == TokenKind::Of))))
                        || (token.lexeme.eq_ignore_ascii_case("root")
                            && self.tokens.get(self.current + 1).is_some_and(|next|
                                next.lexeme.eq_ignore_ascii_case("mean"))
                            && ((self.tokens.get(self.current + 2).is_some_and(|next|
                                next.lexeme.eq_ignore_ascii_case("square"))
                                && self.tokens.get(self.current + 3).is_some_and(|next|
                                    next.kind == TokenKind::Of))
                                || (self.tokens.get(self.current + 2).is_some_and(|next|
                                    next.lexeme.eq_ignore_ascii_case("squared"))
                                    && self.tokens.get(self.current + 3).is_some_and(|next|
                                        next.lexeme.eq_ignore_ascii_case("error"))
                                    && self.tokens.get(self.current + 4).is_some_and(|next|
                                        next.kind == TokenKind::Of))))
                        || (matches!(token.lexeme.to_ascii_lowercase().as_str(),
                            "rms" | "mse" | "rmse")
                            && self.tokens.get(self.current + 1).is_some_and(|next|
                                next.kind == TokenKind::Of)))) => {
                let start = self.advance().clone();
                let (sum, root) = if start.kind == TokenKind::Sum {
                    self.consume(TokenKind::Of,
                        "Expected 'of' before squares", "S242")?;
                    self.consume_surface("squares",
                        "Expected 'squares' after sum of", "S242")?;
                    (true, false)
                } else if start.lexeme.eq_ignore_ascii_case("root") {
                    self.consume_surface("mean",
                        "Expected 'mean' after root", "S242")?;
                    if self.peek().lexeme.eq_ignore_ascii_case("squared") {
                        self.consume_surface("squared",
                            "Expected 'squared' after root mean", "S242")?;
                        self.consume_surface("error",
                            "Expected 'error' after root mean squared", "S242")?;
                    } else {
                        self.consume_surface("square",
                            "Expected 'square' after root mean", "S242")?;
                    }
                    (false, true)
                } else if start.lexeme.eq_ignore_ascii_case("mean") {
                    if self.peek().lexeme.eq_ignore_ascii_case("squared") {
                        self.consume_surface("squared",
                            "Expected 'squared' after mean", "S242")?;
                        self.consume_surface("error",
                            "Expected 'error' after mean squared", "S242")?;
                    } else {
                        self.consume_surface("square",
                            "Expected 'square' after mean", "S242")?;
                    }
                    (false, false)
                } else {
                    (false, matches!(start.lexeme.to_ascii_lowercase().as_str(),
                        "rms" | "rmse"))
                };
                self.consume(TokenKind::Of,
                    "Expected 'of' before the list of numeric residuals", "S242")?;
                let source = self.primary()?;
                let span = span_between_spans(start.span, source.span());
                let iterator = "__semauri_square_aggregate_item".to_owned();
                let squared = Expr::Unary {
                    operator: UnaryOperator::Square,
                    operand: Box::new(Expr::Variable {
                        name: iterator.clone(),
                        span: start.span,
                    }),
                    span,
                };
                let mapped = Expr::ListTransform {
                    kind: ListTransformKind::Map,
                    iterator,
                    binding_span: start.span,
                    iterable: Box::new(source),
                    body: Box::new(squared),
                    span,
                };
                let aggregated = Expr::Unary {
                    operator: if sum { UnaryOperator::Sum } else { UnaryOperator::Average },
                    operand: Box::new(mapped),
                    span,
                };
                Ok(if root {
                    Expr::Unary {
                        operator: UnaryOperator::SquareRoot,
                        operand: Box::new(aggregated),
                        span,
                    }
                } else {
                    aggregated
                })
            }
            // Signed model bias is the arithmetic mean of residuals.
            // Mean bias error is distinct from mean absolute/squared error:
            // negative residuals are intentionally preserved.
            TokenKind::Word
                if ((token.lexeme.eq_ignore_ascii_case("mbe")
                    && self.tokens.get(self.current + 1).is_some_and(|next|
                        next.kind == TokenKind::Of))
                    || (token.lexeme.eq_ignore_ascii_case("mean")
                        && ((self.tokens.get(self.current + 1).is_some_and(|next|
                            next.lexeme.eq_ignore_ascii_case("error"))
                            && self.tokens.get(self.current + 2).is_some_and(|next|
                                next.kind == TokenKind::Of))
                            || (self.tokens.get(self.current + 1).is_some_and(|next|
                                next.lexeme.eq_ignore_ascii_case("bias"))
                                && self.tokens.get(self.current + 2).is_some_and(|next|
                                    next.lexeme.eq_ignore_ascii_case("error"))
                                && self.tokens.get(self.current + 3).is_some_and(|next|
                                    next.kind == TokenKind::Of))))) => {
                let start = self.advance().clone();
                if start.lexeme.eq_ignore_ascii_case("mean") {
                    if self.peek().lexeme.eq_ignore_ascii_case("bias") {
                        self.consume_surface("bias",
                            "Expected 'bias' after mean", "S242")?;
                    }
                    self.consume_surface("error",
                        "Expected 'error' after mean/bias", "S242")?;
                }
                self.consume(TokenKind::Of,
                    "Expected 'of' before the list of signed residuals", "S242")?;
                let operand = self.primary()?;
                let span = span_between_spans(start.span, operand.span());
                Ok(Expr::Unary {
                    operator: UnaryOperator::Average,
                    operand: Box::new(operand),
                    span,
                })
            }
            // Absolute-error statistics over a list of signed residuals.
            // Reuse one typed map of checked absolute values, then the
            // existing sum, average or maximum. This preserves single
            // evaluation of the source list and exact integer arithmetic.
            kind if (kind == TokenKind::Sum
                    && self.tokens.get(self.current + 1).is_some_and(|next|
                        next.kind == TokenKind::Of)
                    && self.tokens.get(self.current + 2).is_some_and(|next|
                        next.kind == TokenKind::Word
                        && next.lexeme.eq_ignore_ascii_case("magnitudes"))
                    && self.tokens.get(self.current + 3).is_some_and(|next|
                        next.kind == TokenKind::Of))
                || (kind == TokenKind::Word
                    && ((matches!(token.lexeme.to_ascii_lowercase().as_str(),
                        "mean" | "max")
                        && self.tokens.get(self.current + 1).is_some_and(|next|
                            next.kind == TokenKind::Word
                            && next.lexeme.eq_ignore_ascii_case("magnitude"))
                        && self.tokens.get(self.current + 2).is_some_and(|next|
                            next.kind == TokenKind::Of))
                        || (token.lexeme.eq_ignore_ascii_case("mean")
                            && self.tokens.get(self.current + 1).is_some_and(|next|
                                next.kind == TokenKind::Word
                                && next.lexeme.eq_ignore_ascii_case("absolute"))
                            && self.tokens.get(self.current + 2).is_some_and(|next|
                                next.kind == TokenKind::Word
                                && next.lexeme.eq_ignore_ascii_case("error"))
                            && self.tokens.get(self.current + 3).is_some_and(|next|
                                next.kind == TokenKind::Of))
                        || (token.lexeme.eq_ignore_ascii_case("mae")
                            && self.tokens.get(self.current + 1).is_some_and(|next|
                                next.kind == TokenKind::Of)))) => {
                let start = self.advance().clone();
                let operation = if start.kind == TokenKind::Sum {
                    self.consume(TokenKind::Of,
                        "Expected 'of' before magnitudes", "S242")?;
                    self.consume_surface("magnitudes",
                        "Expected 'magnitudes' after sum of", "S242")?;
                    UnaryOperator::Sum
                } else if start.lexeme.eq_ignore_ascii_case("mae") {
                    UnaryOperator::Average
                } else {
                    if start.lexeme.eq_ignore_ascii_case("mean")
                        && self.peek().lexeme.eq_ignore_ascii_case("absolute")
                    {
                        self.consume_surface("absolute",
                            "Expected 'absolute' after mean", "S242")?;
                        self.consume_surface("error",
                            "Expected 'error' after mean absolute", "S242")?;
                    } else {
                        self.consume_surface("magnitude",
                            "Expected 'magnitude' after mean/max", "S242")?;
                    }
                    if start.lexeme.eq_ignore_ascii_case("max") {
                        UnaryOperator::Maximum
                    } else {
                        UnaryOperator::Average
                    }
                };
                self.consume(TokenKind::Of,
                    "Expected 'of' before the list of numeric residuals", "S242")?;
                let source = self.primary()?;
                let span = span_between_spans(start.span, source.span());
                // An internal symbol in the map scope, never a user binding.
                let iterator = "__semauri_magnitude_aggregate_item".to_owned();
                let absolute = Expr::Unary {
                    operator: UnaryOperator::Absolute,
                    operand: Box::new(Expr::Variable {
                        name: iterator.clone(),
                        span: start.span,
                    }),
                    span,
                };
                let mapped = Expr::ListTransform {
                    kind: ListTransformKind::Map,
                    iterator,
                    binding_span: start.span,
                    iterable: Box::new(source),
                    body: Box::new(absolute),
                    span,
                };
                Ok(Expr::Unary {
                    operator: operation,
                    operand: Box::new(mapped),
                    span,
                })
            }
            // Keep the parser's recursive primary() frame small by
            // implementing the numeric scan body in its own method.
            TokenKind::Word
                if (matches!(token.lexeme.to_ascii_lowercase().as_str(),
                    "running" | "cumulative")
                    && self.tokens.get(self.current + 1).is_some_and(|next|
                        matches!(next.kind, TokenKind::Sum | TokenKind::Average
                            | TokenKind::Minimum | TokenKind::Maximum)
                            || (next.kind == TokenKind::Word
                                && matches!(next.lexeme.to_ascii_lowercase().as_str(),
                                    "total" | "product" | "min" | "max")))
                    && self.tokens.get(self.current + 2).is_some_and(|next|
                        next.kind == TokenKind::Of))
                    || (matches!(token.lexeme.to_ascii_lowercase().as_str(),
                        "adjacent" | "successive")
                        && self.tokens.get(self.current + 1).is_some_and(|next|
                            next.kind == TokenKind::Word
                            && next.lexeme.eq_ignore_ascii_case("differences"))
                        && self.tokens.get(self.current + 2).is_some_and(|next|
                            next.kind == TokenKind::Of)) => self.numeric_series_expression(),
            // Python-friendly contextual functions; keep the strict existing HIR operators.
            TokenKind::Word
                if matches!(token.lexeme.to_ascii_lowercase().as_str(),
                    "len" | "abs" | "mean" | "min" | "max" | "strip" | "upper" | "lower"
                    | "avg" | "var" | "stddev" | "total" | "size" | "unique" | "head" | "final" | "med" | "prod"
                    | "rev" | "sort" | "flat")
                    && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after function alias", "S242")?;
                let operand = self.primary()?;
                let operator = match start.lexeme.to_ascii_lowercase().as_str() {
                    "len" => UnaryOperator::Length,
                    "abs" => UnaryOperator::Absolute,
                    "mean" => UnaryOperator::Average,
                    "min" => UnaryOperator::Minimum,
                    "max" => UnaryOperator::Maximum,
                    "strip" => UnaryOperator::Trim,
                    "upper" => UnaryOperator::Uppercase,
                    "lower" => UnaryOperator::Lowercase,
                    "avg" => UnaryOperator::Average,
                    "var" => UnaryOperator::Variance,
                    "stddev" => UnaryOperator::StandardDeviation,
                    "total" => UnaryOperator::Sum,
                    "size" => UnaryOperator::Length,
                    "unique" => UnaryOperator::Distinct,
                    "head" => UnaryOperator::First,
                    "final" => UnaryOperator::Last,
                    "med" => UnaryOperator::Median,
                    "prod" => UnaryOperator::Product,
                    "rev" => UnaryOperator::Reverse,
                    "sort" => UnaryOperator::Sorted,
                    "flat" => UnaryOperator::Flatten,
                    _ => unreachable!("guard restricts aliases"),
                };
                Ok(Expr::Unary {
                    operator,
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            TokenKind::Word
                if matches!(token.lexeme.to_ascii_lowercase().as_str(), "variance" | "stdev")
                    && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after statistical operator", "S242")?;
                let operand = self.primary()?;
                Ok(Expr::Unary {
                    operator: if start.lexeme.eq_ignore_ascii_case("variance") {
                        UnaryOperator::Variance
                    } else {
                        UnaryOperator::StandardDeviation
                    },
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            TokenKind::Word if token.lexeme.eq_ignore_ascii_case("power")
                && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after power", "S252")?;
                let base = self.numeric_unary()?;
                self.consume(TokenKind::To, "Expected 'to' after power base", "S252")?;
                let exponent = self.numeric_unary()?;
                Ok(Expr::Binary {
                    left: Box::new(base),
                    right: Box::new(exponent.clone()),
                    operator: BinaryOperator::Power,
                    span: span_between_spans(start.span, exponent.span()),
                })
            }
            TokenKind::Word if token.lexeme.eq_ignore_ascii_case("repeat")
                && self.tokens.get(self.current + 1).is_some_and(|next| matches!(
                    next.kind, TokenKind::Number | TokenKind::Boolean | TokenKind::String | TokenKind::Color | TokenKind::Word | TokenKind::Minus | TokenKind::LParen
                )) => {
                let start = self.advance().clone();
                let count = self.numeric_unary()?;
                self.consume(TokenKind::Of, "Expected 'of' after repeat count", "S249")?;
                let value = self.primary()?;
                Ok(Expr::Repeat {
                    span: span_between_spans(start.span, value.span()),
                    count: Box::new(count),
                    value: Box::new(value),
                })
            }
            TokenKind::If => {
                // Expression-only branch form: if EXPR then EXPR otherwise EXPR.
                // Existing statement 'if EXPR: ... end.' remains unchanged.
                let start = self.advance().clone();
                let condition = self.expression()?;
                self.consume_surface("then", "Expected 'then' after expression condition", "S248")?;
                let consequence = self.expression()?;
                self.consume(TokenKind::Otherwise,
                    "Expected 'otherwise' between conditional expression values", "S248")?;
                let alternative = self.expression()?;
                Ok(Expr::Conditional {
                    span: span_between_spans(start.span, alternative.span()),
                    condition: Box::new(condition),
                    consequence: Box::new(consequence),
                    alternative: Box::new(alternative),
                })
            }
            TokenKind::Color => {
                self.advance();
                Ok(Expr::Literal {
                    ty: Type::Color,
                    value: LiteralValue::Color(token_string_literal(&token)),
                    span: token.span,
                })
            }
            TokenKind::String => {
                self.advance();
                Ok(Expr::Literal {
                    ty: Type::String,
                    value: LiteralValue::String(token_string_literal(&token)),
                    span: token.span,
                })
            }
            TokenKind::Number => {
                self.advance();
                let value = match token.literal {
                    TokenLiteral::Number(value) => value,
                    _ => unreachable!(),
                };
                Ok(Expr::Literal {
                    ty: Type::Number,
                    value: LiteralValue::Number(value),
                    span: token.span,
                })
            }
            TokenKind::Boolean => {
                self.advance();
                Ok(Expr::Literal {
                    ty: Type::Boolean,
                    value: LiteralValue::Boolean(token.lexeme.eq_ignore_ascii_case("true")),
                    span: token.span,
                })
            }
            TokenKind::Article => {
                let start = self.advance().clone();
                self.list_literal(&start, false)
            }
            TokenKind::List => {
                let start = self.advance().clone();
                self.list_literal(&start, true)
            }
            // Select highest/lowest distinct values without repeating input evaluation.
            TokenKind::Word
                if matches!(token.lexeme.to_ascii_lowercase().as_str(), "top" | "bottom")
                    && self.tokens.get(self.current + 1).is_some_and(|next|
                        next.kind == TokenKind::Distinct
                        || (next.kind == TokenKind::Word
                            && next.lexeme.eq_ignore_ascii_case("unique")))
                    && self.tokens.get(self.current + 2).is_some_and(|next|
                        matches!(next.kind, TokenKind::Number | TokenKind::Word
                            | TokenKind::Boolean | TokenKind::String
                            | TokenKind::Minus | TokenKind::LParen)) => {
                let start = self.advance().clone();
                let top = start.lexeme.eq_ignore_ascii_case("top");
                self.advance(); // distinct / unique
                let count = self.numeric_unary()?;
                self.consume(TokenKind::From,
                    "Expected 'from' after top/bottom distinct count", "S243")?;
                let source = self.primary()?;
                let span = span_between_spans(start.span, source.span());
                let distinct = Expr::Unary {
                    operator: UnaryOperator::Distinct,
                    operand: Box::new(source),
                    span,
                };
                let sorted = Expr::Unary {
                    operator: UnaryOperator::Sorted,
                    operand: Box::new(distinct),
                    span,
                };
                let sorted = if top {
                    Expr::Unary {
                        operator: UnaryOperator::Reverse,
                        operand: Box::new(sorted),
                        span,
                    }
                } else { sorted };
                Ok(Expr::Binary {
                    left: Box::new(count),
                    operator: BinaryOperator::Take,
                    right: Box::new(sorted),
                    span,
                })
            }
            // Select the largest or smallest N elements without evaluating the
            // source list twice. Existing Sorted and Take enforce exact
            // ordering, list types, nonnegative integral counts and bounds.
            TokenKind::Word
                if matches!(token.lexeme.to_ascii_lowercase().as_str(), "top" | "bottom")
                    && self.tokens.get(self.current + 1).is_some_and(|next|
                        matches!(next.kind, TokenKind::Number | TokenKind::Word
                            | TokenKind::Boolean | TokenKind::String
                            | TokenKind::Minus | TokenKind::LParen)) => {
                let start = self.advance().clone();
                let largest_first = start.lexeme.eq_ignore_ascii_case("top");
                let count = self.numeric_unary()?;
                self.consume(TokenKind::From,
                    "Expected 'from' after top/bottom count", "S243")?;
                let input = self.primary()?;
                let span = span_between_spans(start.span, input.span());
                let ordered = Expr::Unary {
                    operator: UnaryOperator::Sorted,
                    operand: Box::new(input),
                    span,
                };
                let ordered = if largest_first {
                    Expr::Unary {
                        operator: UnaryOperator::Reverse,
                        operand: Box::new(ordered),
                        span,
                    }
                } else {
                    ordered
                };
                Ok(Expr::Binary {
                    left: Box::new(count),
                    operator: BinaryOperator::Take,
                    right: Box::new(ordered),
                    span,
                })
            }
            // Suffix selection is equivalent to reversing the input, applying
            // the checked Take/Drop, and reversing the result. This preserves
            // Unicode scalar order, the list's original order, and one-time
            // evaluation of both operands. Require a count token after 'last':
            // 'let last be 2. take last from values' remains valid.
            kind if matches!(kind, TokenKind::Take | TokenKind::Drop)
                && self.tokens.get(self.current + 1)
                    .is_some_and(|next| next.kind == TokenKind::Last)
                && self.tokens.get(self.current + 2).is_some_and(|next|
                    matches!(next.kind, TokenKind::Number | TokenKind::Word
                        | TokenKind::Boolean | TokenKind::String
                        | TokenKind::Minus | TokenKind::LParen)) => {
                let start = self.advance().clone();
                self.advance(); // last
                let count = self.numeric_unary()?;
                self.consume(TokenKind::From,
                    "Expected 'from' after take/drop last count", "S243")?;
                let source = self.primary()?;
                let span = span_between_spans(start.span, source.span());
                let reversed = Expr::Unary {
                    operator: UnaryOperator::Reverse,
                    operand: Box::new(source),
                    span,
                };
                let sliced = Expr::Binary {
                    left: Box::new(count),
                    operator: if start.kind == TokenKind::Take {
                        BinaryOperator::Take
                    } else {
                        BinaryOperator::Drop
                    },
                    right: Box::new(reversed),
                    span,
                };
                Ok(Expr::Unary {
                    operator: UnaryOperator::Reverse,
                    operand: Box::new(sliced),
                    span,
                })
            }
            // Explicit first-N variants reuse the checked Take/Drop operator.
            // Only treat first as syntax when followed by a count expression:
            // take first from values still accepts a variable called first.
            kind if matches!(kind, TokenKind::Take | TokenKind::Drop)
                && self.tokens.get(self.current + 1)
                    .is_some_and(|next| next.kind == TokenKind::First)
                && self.tokens.get(self.current + 2).is_some_and(|next|
                    matches!(next.kind, TokenKind::Number | TokenKind::Word
                        | TokenKind::Boolean | TokenKind::String
                        | TokenKind::Minus | TokenKind::LParen)) => {
                let start = self.advance().clone();
                self.advance(); // first
                let count = self.numeric_unary()?;
                self.consume(TokenKind::From,
                    "Expected 'from' after first-count slicing", "S243")?;
                let source = self.primary()?;
                let span = span_between_spans(start.span, source.span());
                Ok(Expr::Binary {
                    left: Box::new(count),
                    operator: if start.kind == TokenKind::Take {
                        BinaryOperator::Take
                    } else {
                        BinaryOperator::Drop
                    },
                    right: Box::new(source),
                    span,
                })
            }
            TokenKind::Take | TokenKind::Drop => {
                let start = self.advance().clone();
                let count = self.numeric_unary()?;
                self.consume(
                    TokenKind::From,
                    "Expected 'from' between a count and a list or string",
                    "S243",
                )?;
                let list = self.primary()?;
                let span = span_between_spans(start.span, list.span());
                Ok(Expr::Binary {
                    left: Box::new(count),
                    operator: if start.kind == TokenKind::Take {
                        BinaryOperator::Take
                    } else {
                        BinaryOperator::Drop
                    },
                    right: Box::new(list),
                    span,
                })
            }
            TokenKind::Last
                if self.tokens.get(self.current + 1)
                    .is_some_and(|next| next.lexeme.eq_ignore_ascii_case("position"))
                    && self.tokens.get(self.current + 2)
                        .is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.advance(); // contextual 'position'
                self.consume(TokenKind::Of, "Expected 'of' after last position", "S242")?;
                let needle = self.primary()?;
                self.consume(TokenKind::In, "Expected 'in' before searched text", "S242")?;
                let searched = self.primary()?;
                Ok(Expr::Binary {
                    left: Box::new(needle),
                    right: Box::new(searched.clone()),
                    operator: BinaryOperator::TextLastPosition,
                    span: span_between_spans(start.span, searched.span()),
                })
            }
            TokenKind::First | TokenKind::Last
                if self.tokens.get(self.current + 1)
                    .is_some_and(|next| next.kind != TokenKind::Of) => {
                self.advance();
                Ok(Expr::Variable {
                    name: token.lexeme.to_lowercase(),
                    span: token.span,
                })
            }
            TokenKind::Any | TokenKind::All
                if self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Every) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Every, "Expected 'every' after any/all", "S251")?;
                let iterator = self.consume(TokenKind::Word, "Expected predicate iterator name", "S251")?;
                self.consume(TokenKind::In, "Expected 'in' after predicate iterator", "S251")?;
                let iterable = self.primary()?;
                self.consume(TokenKind::If, "Expected 'if' before quantified predicate", "S251")?;
                let body = self.expression()?;
                Ok(Expr::ListTransform {
                    kind: if start.kind == TokenKind::Any { ListTransformKind::Any } else { ListTransformKind::All },
                    iterator: iterator.lexeme.to_lowercase(),
                    binding_span: iterator.span,
                    iterable: Box::new(iterable),
                    span: span_between_spans(start.span, body.span()),
                    body: Box::new(body),
                })
            }
            TokenKind::Word if token.lexeme.eq_ignore_ascii_case("product")
                && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after product", "S242")?;
                let operand = self.primary()?;
                Ok(Expr::Unary {
                    operator: UnaryOperator::Product,
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            TokenKind::Word if token.lexeme.eq_ignore_ascii_case("absolute")
                && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after absolute", "S242")?;
                let operand = self.primary()?;
                Ok(Expr::Unary {
                    operator: UnaryOperator::Absolute,
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            TokenKind::Word if token.lexeme.eq_ignore_ascii_case("sign")
                && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after sign", "S242")?;
                let operand = self.primary()?;
                Ok(Expr::Unary {
                    operator: UnaryOperator::Sign,
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            TokenKind::Word
                if matches!(token.lexeme.to_ascii_lowercase().as_str(), "floor" | "ceiling")
                    && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after numeric rounding operator", "S242")?;
                let operand = self.primary()?;
                Ok(Expr::Unary {
                    operator: if start.lexeme.eq_ignore_ascii_case("floor") {
                        UnaryOperator::Floor
                    } else {
                        UnaryOperator::Ceiling
                    },
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            TokenKind::Word if token.lexeme.eq_ignore_ascii_case("round")
                && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after round", "S242")?;
                let operand = self.primary()?;
                Ok(Expr::Unary {
                    operator: UnaryOperator::Round,
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            TokenKind::Word if token.lexeme.eq_ignore_ascii_case("truncate")
                && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after truncate", "S242")?;
                let operand = self.primary()?;
                Ok(Expr::Unary {
                    operator: UnaryOperator::Truncate,
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            TokenKind::Word if token.lexeme.eq_ignore_ascii_case("factorial")
                && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after factorial", "S242")?;
                let operand = self.primary()?;
                Ok(Expr::Unary {
                    operator: UnaryOperator::Factorial,
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            TokenKind::Word if token.lexeme.eq_ignore_ascii_case("prime")
                && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after prime", "S242")?;
                let operand = self.primary()?;
                Ok(Expr::Unary {
                    operator: UnaryOperator::Prime,
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            TokenKind::Word if token.lexeme.eq_ignore_ascii_case("cube")
                && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after cube", "S242")?;
                let operand = self.primary()?;
                Ok(Expr::Unary {
                    operator: UnaryOperator::Cube,
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            TokenKind::Word if token.lexeme.eq_ignore_ascii_case("square")
                && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after square", "S242")?;
                let operand = self.primary()?;
                Ok(Expr::Unary {
                    operator: UnaryOperator::Square,
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            // Checked integer-to-text conversions for binary, octal and hex.
            // The names remain ordinary identifiers without a following 'of'.
            TokenKind::Word
                if matches!(token.lexeme.to_ascii_lowercase().as_str(),
                    "binary" | "octal" | "hex" | "hexadecimal")
                    && self.tokens.get(self.current + 1)
                        .is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after radix formatter", "S242")?;
                let operand = self.primary()?;
                let operator = match start.lexeme.to_ascii_lowercase().as_str() {
                    "binary" => UnaryOperator::BinaryDigits,
                    "octal" => UnaryOperator::OctalDigits,
                    "hex" | "hexadecimal" => UnaryOperator::HexadecimalDigits,
                    _ => unreachable!("radix guard"),
                };
                Ok(Expr::Unary {
                    operator,
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            TokenKind::Word if token.lexeme.eq_ignore_ascii_case("digits")
                && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after digits", "S242")?;
                let operand = self.primary()?;
                Ok(Expr::Unary {
                    operator: UnaryOperator::Digits,
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            TokenKind::Word if token.lexeme.eq_ignore_ascii_case("reciprocal")
                && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after reciprocal", "S242")?;
                let operand = self.primary()?;
                Ok(Expr::Unary {
                    operator: UnaryOperator::Reciprocal,
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            TokenKind::Word if token.lexeme.eq_ignore_ascii_case("cbrt")
                && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after cbrt", "S242")?;
                let operand = self.primary()?;
                Ok(Expr::Unary {
                    operator: UnaryOperator::CubeRoot,
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            TokenKind::Word
                if matches!(token.lexeme.to_ascii_lowercase().as_str(), "exp" | "ln" | "log10" | "log2" | "sin" | "cos" | "asin" | "acos" | "atan" | "sinh" | "cosh" | "tanh" | "asinh" | "acosh" | "atanh" | "log1p" | "expm1")
                    && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after scientific math operator", "S242")?;
                let operand = self.primary()?;
                let operator = match start.lexeme.to_ascii_lowercase().as_str() {
                    "exp" => UnaryOperator::Exponential,
                    "ln" => UnaryOperator::NaturalLog,
                    "log10" => UnaryOperator::CommonLog,
                    "log2" => UnaryOperator::BinaryLog,
                    "sin" => UnaryOperator::Sine,
                    "cos" => UnaryOperator::Cosine,
                    "asin" => UnaryOperator::ArcSine,
                    "acos" => UnaryOperator::ArcCosine,
                    "atan" => UnaryOperator::ArcTangent,
                    "sinh" => UnaryOperator::HyperbolicSine,
                    "cosh" => UnaryOperator::HyperbolicCosine,
                    "tanh" => UnaryOperator::HyperbolicTangent,
                    "asinh" => UnaryOperator::HyperbolicArcSine,
                    "acosh" => UnaryOperator::HyperbolicArcCosine,
                    "atanh" => UnaryOperator::HyperbolicArcTangent,
                    "log1p" => UnaryOperator::LogOnePlus,
                    "expm1" => UnaryOperator::ExpMinusOne,
                    _ => unreachable!("guard guarantees a scientific function"),

                };
                Ok(Expr::Unary {
                    operator,
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            TokenKind::Word if token.lexeme.eq_ignore_ascii_case("sqrt")
                && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after sqrt", "S242")?;
                let operand = self.primary()?;
                Ok(Expr::Unary {
                    operator: UnaryOperator::SquareRoot,
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            TokenKind::Word
                if matches!(token.lexeme.to_ascii_lowercase().as_str(), "even" | "odd")
                    && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after parity operator", "S242")?;
                let operand = self.primary()?;
                Ok(Expr::Unary {
                    operator: if start.lexeme.eq_ignore_ascii_case("even") {
                        UnaryOperator::Even
                    } else {
                        UnaryOperator::Odd
                    },
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            TokenKind::Word if token.lexeme.eq_ignore_ascii_case("flatten")
                && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after flatten", "S242")?;
                let operand = self.primary()?;
                Ok(Expr::Unary {
                    operator: UnaryOperator::Flatten,
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            // Split is also a registered ML action; the following 'of'\n            // distinguishes this pure expression without changing ML syntax.
            // Pure Unicode text search retains contextual identifier names.
            TokenKind::Word
                if matches!(token.lexeme.to_ascii_lowercase().as_str(), "occurrences" | "position" | "positions")
                    && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after text search operation", "S242")?;
                let needle = self.primary()?;
                self.consume(TokenKind::In, "Expected 'in' before searched text", "S242")?;
                let text = self.primary()?;
                let span = span_between_spans(start.span, text.span());
                let operator = match start.lexeme.to_ascii_lowercase().as_str() {
                    "occurrences" => BinaryOperator::TextOccurrences,
                    "positions" => BinaryOperator::TextPositions,
                    _ => BinaryOperator::TextPosition,
                };
                Ok(Expr::Binary {
                    left: Box::new(needle),
                    right: Box::new(text),
                    operator,
                    span,
                })
            }
            // RFC 6901 JSON navigation composes with strict typed decoding.
            TokenKind::Word
                if token.lexeme.eq_ignore_ascii_case("json")
                    && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::At)
                    && self.tokens.get(self.current + 2).is_some_and(|next|
                        next.lexeme.eq_ignore_ascii_case("pointer")) => {
                let start = self.advance().clone();
                self.consume(TokenKind::At, "Expected 'at' after json", "S242")?;
                self.advance(); // pointer
                let pointer = self.primary()?;
                self.consume(TokenKind::In, "Expected 'in' before JSON document", "S242")?;
                let document = self.primary()?;
                let span = span_between_spans(start.span, document.span());
                Ok(Expr::Binary {
                    left: Box::new(pointer),
                    operator: BinaryOperator::JsonPointer,
                    right: Box::new(document),
                    span,
                })
            }
            // Typed field projection requires a complete contextual phrase,
            // so existing 'json of' and identifiers are unaffected.
            TokenKind::Word
                if token.lexeme.eq_ignore_ascii_case("json")
                    && self.tokens.get(self.current + 1).is_some_and(|next| matches!(
                        next.lexeme.to_ascii_lowercase().as_str(),
                        "string" | "number" | "boolean" | "strings" | "numbers" | "booleans"
                    ))
                    && self.tokens.get(self.current + 2).is_some_and(|next| next.kind == TokenKind::At) => {
                let start = self.advance().clone();
                let target = self.advance().clone();
                self.consume(TokenKind::At, "Expected 'at' after JSON field type", "S242")?;
                let key = self.primary()?;
                self.consume(TokenKind::In, "Expected 'in' before JSON document", "S242")?;
                let document = self.primary()?;
                let span = span_between_spans(start.span, document.span());
                let operator = match target.lexeme.to_ascii_lowercase().as_str() {
                    "string" => BinaryOperator::JsonFieldString,
                    "number" => BinaryOperator::JsonFieldNumber,
                    "boolean" => BinaryOperator::JsonFieldBoolean,
                    "strings" => BinaryOperator::JsonFieldStrings,
                    "numbers" => BinaryOperator::JsonFieldNumbers,
                    "booleans" => BinaryOperator::JsonFieldBooleans,
                    _ => unreachable!(),
                };
                Ok(Expr::Binary {
                    left: Box::new(key),
                    right: Box::new(document),
                    operator,
                    span,
                })
            }
            // Inspect JSON object keys without materializing untyped records.
            TokenKind::Word
                if token.lexeme.eq_ignore_ascii_case("keys")
                    && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of)
                    && self.tokens.get(self.current + 2)
                        .is_some_and(|next| next.lexeme.eq_ignore_ascii_case("json")) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after keys", "S242")?;
                self.advance(); // contextual 'json'
                let operand = self.primary()?;
                Ok(Expr::Unary {
                    operator: UnaryOperator::JsonKeys,
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            TokenKind::Word
                if token.lexeme.eq_ignore_ascii_case("has")
                    && self.tokens.get(self.current + 1)
                        .is_some_and(|next| next.lexeme.eq_ignore_ascii_case("json"))
                    && self.tokens.get(self.current + 2)
                        .is_some_and(|next| next.lexeme.eq_ignore_ascii_case("key")) => {
                let start = self.advance().clone();
                self.advance(); // json
                self.advance(); // key
                let key = self.primary()?;
                self.consume(TokenKind::In, "Expected 'in' before JSON document", "S242")?;
                let document = self.primary()?;
                let span = span_between_spans(start.span, document.span());
                Ok(Expr::Binary {
                    operator: BinaryOperator::JsonHasKey,
                    left: Box::new(key),
                    right: Box::new(document),
                    span,
                })
            }
            // Type-directed JSON decoding is a complete contextual phrase.
            // No new reserved keywords are required.
            TokenKind::Word
                if token.lexeme.eq_ignore_ascii_case("decode")
                    && self.tokens.get(self.current + 1)
                        .is_some_and(|next| next.lexeme.eq_ignore_ascii_case("json"))
                    && self.tokens.get(self.current + 2)
                        .is_some_and(|next| next.lexeme.eq_ignore_ascii_case("as"))
                    && self.tokens.get(self.current + 3).is_some_and(|next| matches!(
                        next.lexeme.to_ascii_lowercase().as_str(),
                        "string" | "number" | "boolean" | "strings" | "numbers" | "booleans"
                    ))
                    && self.tokens.get(self.current + 4)
                        .is_some_and(|next| next.kind == TokenKind::From) => {
                let start = self.advance().clone();
                self.advance(); // json
                self.advance(); // as
                let target = self.advance().clone();
                self.consume(TokenKind::From, "Expected 'from' after JSON type", "S242")?;
                let operand = self.primary()?;
                let operator = match target.lexeme.to_ascii_lowercase().as_str() {
                    "string" => UnaryOperator::DecodeJsonString,
                    "number" => UnaryOperator::DecodeJsonNumber,
                    "boolean" => UnaryOperator::DecodeJsonBoolean,
                    "strings" => UnaryOperator::DecodeJsonStrings,
                    "numbers" => UnaryOperator::DecodeJsonNumbers,
                    "booleans" => UnaryOperator::DecodeJsonBooleans,
                    _ => unreachable!(),
                };
                Ok(Expr::Unary {
                    operator,
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            TokenKind::Word
                if token.lexeme.eq_ignore_ascii_case("parse")
                    && self.tokens.get(self.current + 1).is_some_and(|next|
                        matches!(next.kind, TokenKind::Word | TokenKind::DomainAction)
                            && matches!(next.lexeme.to_ascii_lowercase().as_str(), "number" | "boolean"))
                    && self.tokens.get(self.current + 2).is_some_and(|next| next.kind == TokenKind::From) => {
                let start = self.advance().clone();
                let target = self.advance().clone();
                self.consume(TokenKind::From, "Expected 'from' after parse type", "S242")?;
                let operand = self.primary()?;
                Ok(Expr::Unary {
                    operator: if target.lexeme.eq_ignore_ascii_case("number") {
                        UnaryOperator::ParseNumber
                    } else {
                        UnaryOperator::ParseBoolean
                    },
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            TokenKind::Word | TokenKind::DomainAction
                if matches!(token.lexeme.to_ascii_lowercase().as_str(), "split" | "join")
                    && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after split/join", "S242")?;
                let left = self.primary()?;
                let operator = if start.lexeme.eq_ignore_ascii_case("split") {
                    self.consume(TokenKind::By, "Expected 'by' after text to split", "S242")?;
                    BinaryOperator::Split
                } else {
                    self.consume_surface("with", "Expected 'with' after list to join", "S242")?;
                    BinaryOperator::Join
                };
                let right = self.primary()?;
                Ok(Expr::Binary {
                    span: span_between_spans(start.span, right.span()),
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                })
            }
            // Pure literal substring removal is a full contextual phrase,
            // preserving domain actions and ordinary identifier names.
            TokenKind::Word | TokenKind::DomainAction
                if token.lexeme.eq_ignore_ascii_case("remove")
                    && self.tokens.get(self.current + 1).is_some_and(|next|
                        next.lexeme.eq_ignore_ascii_case("all"))
                    && self.tokens.get(self.current + 2).is_some_and(|next|
                        next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.advance(); // all
                self.consume(TokenKind::Of, "Expected 'of' after 'remove all'", "S242")?;
                let needle = self.primary()?;
                self.consume(TokenKind::From, "Expected 'from' after search text", "S242")?;
                let text = self.primary()?;
                let span = span_between_spans(start.span, text.span());
                Ok(Expr::Binary {
                    left: Box::new(needle),
                    operator: BinaryOperator::TextRemoveAll,
                    right: Box::new(text),
                    span,
                })
            }
            // Character indexing uses Unicode scalar positions and is contextual,
            // just like 'character from' in the scalar conversion family.
            TokenKind::Word
                if token.lexeme.eq_ignore_ascii_case("character")
                    && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::At) => {
                let start = self.advance().clone();
                self.consume(TokenKind::At, "Expected 'at' after character", "S242")?;
                let index = self.numeric_unary()?;
                self.consume(TokenKind::In, "Expected 'in' before text", "S242")?;
                let text = self.primary()?;
                let span = span_between_spans(start.span, text.span());
                Ok(Expr::Binary {
                    left: Box::new(index),
                    operator: BinaryOperator::CharacterAt,
                    right: Box::new(text),
                    span,
                })
            }
            // Construct one Unicode scalar from a checked integer code point.
            TokenKind::Word
                if token.lexeme.eq_ignore_ascii_case("character")
                    && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::From) => {
                let start = self.advance().clone();
                self.consume(TokenKind::From, "Expected 'from' after character", "S242")?;
                let operand = self.numeric_unary()?;
                Ok(Expr::Unary {
                    operator: UnaryOperator::CharacterFrom,
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            // Contextual text operators do not reserve identifiers such as 'words'.
            TokenKind::Word
                if matches!(token.lexeme.to_ascii_lowercase().as_str(), "trim" | "words" | "lines" | "capitalize" | "swapcase" | "uncapitalize" | "titlecase" | "json" | "characters" | "codepoints" | "codepoint")
                    && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after a text operation", "S242")?;
                let operand = self.primary()?;
                Ok(Expr::Unary {
                    operator: match start.lexeme.to_ascii_lowercase().as_str() {
                        "trim" => UnaryOperator::Trim,
                        "capitalize" => UnaryOperator::Capitalize,
                        "swapcase" => UnaryOperator::Swapcase,
                        "uncapitalize" => UnaryOperator::Uncapitalize,
                        "titlecase" => UnaryOperator::Titlecase,
                        "words" => UnaryOperator::Words,
                        "lines" => UnaryOperator::Lines,
                        "json" => UnaryOperator::Json,
                        "characters" => UnaryOperator::Characters,
                        "codepoints" => UnaryOperator::Codepoints,
                        "codepoint" => UnaryOperator::Codepoint,
                        _ => unreachable!(),
                    },
                    span: span_between_spans(start.span, operand.span()),
                    operand: Box::new(operand),
                })
            }
            TokenKind::Length | TokenKind::Reverse | TokenKind::Sum | TokenKind::Average
            | TokenKind::Minimum | TokenKind::Maximum | TokenKind::Median | TokenKind::Sorted | TokenKind::Distinct | TokenKind::First | TokenKind::Last | TokenKind::Any | TokenKind::All
            | TokenKind::Uppercase | TokenKind::Lowercase => {
                let start = self.advance().clone();
                self.consume(
                    TokenKind::Of,
                    "Expected 'of' after a prefix operation",
                    "S242",
                )?;
                // A primary operand keeps prefix precedence unambiguous.
                // Use parentheses to measure a larger expression.
                let operand = self.primary()?;
                let span = span_between_spans(start.span, operand.span());
                Ok(Expr::Unary {
                    operator: match start.kind {
                        TokenKind::Length => UnaryOperator::Length,
                        TokenKind::Reverse => UnaryOperator::Reverse,
                        TokenKind::Sum => UnaryOperator::Sum,
                        TokenKind::Average => UnaryOperator::Average,
                        TokenKind::Minimum => UnaryOperator::Minimum,
                        TokenKind::Maximum => UnaryOperator::Maximum,
                        TokenKind::Median => UnaryOperator::Median,
                        TokenKind::Sorted => UnaryOperator::Sorted,
                        TokenKind::Distinct => UnaryOperator::Distinct,
                        TokenKind::First => UnaryOperator::First,
                        TokenKind::Last => UnaryOperator::Last,
                        TokenKind::Any => UnaryOperator::Any,
                        TokenKind::All => UnaryOperator::All,
                        TokenKind::Uppercase => UnaryOperator::Uppercase,
                        TokenKind::Lowercase => UnaryOperator::Lowercase,
                        _ => unreachable!(),
                    },
                    operand: Box::new(operand),
                    span,
                })
            }
            TokenKind::Word if token.lexeme.eq_ignore_ascii_case("invoke")
                && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Word)
                && self.tokens.get(self.current + 2).is_some_and(|next| next.kind == TokenKind::To) => {
                let start = self.advance().clone();
                let name = self.consume(TokenKind::Word, "Expected a function name after invoke", "S247")?;
                self.consume(TokenKind::To, "Expected 'to' after function name", "S247")?;
                // Parenthesized argument tuples support arity >1.
                // A bare primary keeps all pre-0.26.0 single-argument syntax.
                let (arguments, end_span) = if self.check(TokenKind::LParen) {
                    let open = self.advance().clone();
                    let mut args = Vec::new();
                    if !self.check(TokenKind::RParen) {
                        loop {
                            args.push(self.expression()?);
                            if args.len() > 16 {
                                return Err(self.parse_error(self.peek(),
                                    "Function calls support at most 16 arguments", "S247", None));
                            }
                            if !self.matches(&[TokenKind::Comma]) { break; }
                            // A trailing comma is allowed inside an argument
                            // tuple, but never creates an extra argument.
                            if self.check(TokenKind::RParen) { break; }
                        }
                    }
                    let close = self.consume(TokenKind::RParen,
                        "Expected ')' to close function arguments", "S247")?;
                    let _ = open;
                    (args, close.span)
                } else {
                    let arg = self.primary()?;
                    let end = arg.span();
                    (vec![arg], end)
                };
                Ok(Expr::FunctionCall {
                    name: name.lexeme.to_lowercase(),
                    span: span_between_spans(start.span, end_span),
                    arguments,
                })
            }
            TokenKind::Word if token.lexeme.eq_ignore_ascii_case("range")
                && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::From) => {
                let start = self.advance().clone();
                self.consume(TokenKind::From, "Expected 'from' after range", "S245")?;
                let lower = self.numeric_unary()?;
                // "through" is a contextual inclusive alias for "to".
                if self.check(TokenKind::Word)
                    && self.peek().lexeme.eq_ignore_ascii_case("through") {
                    self.advance();
                } else {
                    self.consume(TokenKind::To,
                        "Expected 'to' or 'through' between range endpoints", "S245")?;
                }
                let upper = self.numeric_unary()?;
                Ok(Expr::Binary {
                    span: span_between_spans(start.span, upper.span()),
                    operator: BinaryOperator::Range,
                    left: Box::new(lower),
                    right: Box::new(upper),
                })
            }
            TokenKind::Word if token.lexeme.eq_ignore_ascii_case("reduce")
                && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Every) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Every, "Expected 'every' after reduce", "S250")?;
                let iterator = self.consume(TokenKind::Word, "Expected reduce element name", "S250")?;
                self.consume(TokenKind::In, "Expected 'in' after element name", "S250")?;
                let iterable = self.primary()?;
                self.consume_surface("starting", "Expected 'starting' after reduce input", "S250")?;
                let accumulator = self.consume(TokenKind::Word, "Expected accumulator name", "S250")?;
                self.consume_surface("as", "Expected 'as' before reduce seed", "S250")?;
                let initial = self.expression()?;
                self.consume(TokenKind::To, "Expected 'to' before reduce body", "S250")?;
                let body = self.expression()?;
                Ok(Expr::Reduce {
                    iterator: iterator.lexeme.to_lowercase(),
                    iterator_span: iterator.span,
                    accumulator: accumulator.lexeme.to_lowercase(),
                    accumulator_span: accumulator.span,
                    iterable: Box::new(iterable),
                    initial: Box::new(initial),
                    span: span_between_spans(start.span, body.span()),
                    body: Box::new(body),
                })
            }
            TokenKind::Word if token.lexeme.eq_ignore_ascii_case("zip")
                && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Every) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Every, "Expected 'every' after zip", "S251")?;
                let left = self.consume(TokenKind::Word, "Expected first zip iterator", "S251")?;
                self.consume(TokenKind::In, "Expected 'in' before first list", "S251")?;
                let left_list = self.primary()?;
                self.consume_surface("with", "Expected 'with' between lists", "S251")?;
                let right = self.consume(TokenKind::Word, "Expected second zip iterator", "S251")?;
                self.consume(TokenKind::In, "Expected 'in' before second list", "S251")?;
                let right_list = self.primary()?;
                self.consume(TokenKind::To, "Expected 'to' before zip body", "S251")?;
                let body = self.expression()?;
                Ok(Expr::ZipTransform {
                    left_name: left.lexeme.to_lowercase(), left_span: left.span,
                    left_list: Box::new(left_list),
                    right_name: right.lexeme.to_lowercase(), right_span: right.span,
                    right_list: Box::new(right_list), span: span_between_spans(start.span, body.span()),
                    body: Box::new(body),
                })
            }
            TokenKind::Word if matches!(token.lexeme.to_ascii_lowercase().as_str(), "map" | "filter" | "count" | "partition")
                && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Every) => {
                let start = self.advance().clone();
                let kind = match start.lexeme.to_ascii_lowercase().as_str() {
                    "map" => ListTransformKind::Map,
                    "filter" => ListTransformKind::Filter,
                    "partition" => ListTransformKind::Partition,
                    _ => ListTransformKind::Count,
                };
                self.consume(
                    TokenKind::Every,
                    "Expected 'every' after map/filter/count/partition",
                    "S244",
                )?;
                let variable = self.consume(
                    TokenKind::Word,
                    "Expected a list-element variable name after 'every'",
                    "S244",
                )?;
                self.consume(
                    TokenKind::In,
                    "Expected 'in' before the input list",
                    "S244",
                )?;
                let iterable = self.primary()?;
                match kind {
                    ListTransformKind::Map => {
                        self.consume(TokenKind::To, "Expected 'to' after the map input", "S244")?;
                    }
                    ListTransformKind::Filter | ListTransformKind::Partition | ListTransformKind::Count | ListTransformKind::Any | ListTransformKind::All => {
                        self.consume(TokenKind::If, "Expected 'if' after the filter/count/partition input", "S244")?;
                    }
                }
                let body = self.expression()?;
                let span = span_between_spans(start.span, body.span());
                Ok(Expr::ListTransform {
                    kind,
                    iterator: variable.lexeme.to_lowercase(),
                    binding_span: variable.span,
                    iterable: Box::new(iterable),
                    body: Box::new(body),
                    span,
                })
            }
            TokenKind::Word => {
                self.advance();
                Ok(Expr::Variable {
                    name: token.lexeme.to_lowercase(),
                    span: token.span,
                })
            }
            TokenKind::DomainAction => {
                let start = self.advance().clone();
                let expression = self.parse_registered_domain_operation(&start)?;
                let operation = match &expression {
                    Expr::DomainOperation {
                        domain, operation, ..
                    } => self.domains.fetch(domain)?.operation(operation).cloned(),
                    _ => None,
                }
                .ok_or_else(|| {
                    self.parse_error(
                        &start,
                        "Unknown registered domain operation",
                        "S239",
                        None,
                    )
                })?;
                if operation.return_type == Type::Unit {
                    return Err(self.parse_error(
                        &start,
                        format!("Operation '{}' does not produce a value", operation.name),
                        "S241",
                        Some(format!(
                            "Use '{}' as a statement instead of inside an expression.",
                            start.lexeme
                        )),
                    ));
                }
                Ok(expression)
            }
            TokenKind::LParen => {
                let opening = self.advance().clone();
                let inner = self.expression()?;
                let closing = self.consume(
                    TokenKind::RParen,
                    "Expected ')' after expression",
                    "S227",
                )?;
                Ok(inner.with_span(span_between_tokens(&opening, &closing)))
            }
            _ => Err(self.parse_error(
                &token,
                "Expected a color, string, number, boolean, list, variable or parenthesized expression",
                "S217",
                None,
            )),
        }
    }

    fn list_literal(&mut self, start: &Token, list_consumed: bool) -> Result<Expr> {
        if !list_consumed {
            self.consume(
                TokenKind::List,
                "Expected 'list' after the article in a list literal",
                "S233",
            )?;
        }
        self.consume(
            TokenKind::Of,
            "Expected 'of' after 'list'",
            "S234",
        )?;
        let mut items = vec![self.expression()?];
        while self.matches(&[TokenKind::Comma]) {
            // The final comma may precede the statement terminator.
            // Reject missing elements in the middle of a list as before.
            if self.check(TokenKind::Dot) { break; }
            items.push(self.expression()?);
        }
        let span = span_between_spans(start.span, items.last().unwrap().span());
        Ok(Expr::List { items, span })
    }

    fn canonical_reference(&mut self, expected_domain: &str) -> Result<Reference> {
        if self.check(TokenKind::Pronoun) {
            let token = self.advance().clone();
            return Ok(Reference::Pronoun {
                pronoun: token.lexeme,
                span: token.span,
            });
        }

        self.matches(&[TokenKind::Article]);
        let element = self.consume(
            TokenKind::DomainElement,
            "Expected a registered element or 'it' after 'of'",
            "S214",
        )?;
        let term = self.domain_term(&element)?;
        if term.domain != expected_domain {
            return Err(self.parse_error(
                &element,
                format!(
                    "Element belongs to domain '{}', expected '{}'",
                    term.domain, expected_domain
                ),
                "S235",
                None,
            ));
        }
        self.consume(
            TokenKind::Called,
            "Canonical references must name the target with 'called' or 'named'",
            "S215",
        )?;
        let label_tokens = self.tokens_until(&[TokenKind::To, TokenKind::Eof]);
        if label_tokens.is_empty() {
            return Err(self.parse_error(
                self.peek(),
                "Expected the referenced element name",
                "S216",
                None,
            ));
        }
        let label = normalize_phrase(&phrase_from_tokens(&label_tokens));
        let span = span_between_tokens(&element, label_tokens.last().unwrap());
        Ok(Reference::Named {
            domain: term.domain,
            kind: term.kind,
            label,
            span,
        })
    }

    fn add_statement(&mut self, start: &Token) -> Result<Stmt> {
        self.matches(&[TokenKind::Article]);
        if self.matches(&[TokenKind::Title]) {
            self.consume(
                TokenKind::Called,
                "Expected 'called' or 'named' after 'title'",
                "S204",
            )?;
            let title = normalize_phrase(&self.phrase_until(&[
                TokenKind::Dot,
                TokenKind::Eof,
            ])?);
            self.consume_optional_dot();
            return Ok(Stmt::SetTitle {
                title,
                span: self.span_from(start),
            });
        }
        if self.check(TokenKind::DomainElement) {
            let element = self.advance().clone();
            let term = self.domain_term(&element)?;
            let label = if self.matches(&[TokenKind::Called]) {
                Some(normalize_phrase(&self.phrase_until(&[
                    TokenKind::Dot,
                    TokenKind::Eof,
                ])?))
            } else {
                None
            };
            self.consume_optional_dot();
            return Ok(Stmt::AddElement {
                domain: term.domain,
                kind: term.kind,
                label,
                span: self.span_from(start),
            });
        }
        Err(self.parse_error(
            self.peek(),
            "Expected 'title' or a registered domain element after 'Add'",
            "S203",
            None,
        ))
    }

    fn parse_registered_domain_operation(&mut self, start: &Token) -> Result<Expr> {
        let term = self.action_term(start)?;
        let domain = self.domains.fetch(&term.domain)?;
        let operation = domain.operation(&term.kind).cloned().ok_or_else(|| {
            self.parse_error(start, "Unknown domain operation", "S239", None)
        })?;
        let mut arguments = BTreeMap::new();
        for segment in &operation.pattern {
            match segment {
                PatternSegment::Literal(word) => {
                    self.consume_surface(
                        word,
                        &format!(
                            "Expected '{}' in '{}' operation",
                            word, operation.name
                        ),
                        "S238",
                    )?;
                }
                PatternSegment::Slot { name, .. } => {
                    arguments.insert(name.clone(), self.expression()?);
                }
            }
        }
        Ok(Expr::DomainOperation {
            domain: term.domain,
            operation: operation.name,
            arguments,
            span: self.span_from(start),
        })
    }

    fn action_term(&self, token: &Token) -> Result<DomainTerm> {
        let candidates = match &token.literal {
            TokenLiteral::ActionCandidates(values) => values.clone(),
            TokenLiteral::DomainTerm(value) => vec![value.clone()],
            _ => {
                return Err(self.parse_error(
                    token,
                    "Domain token is missing semantic metadata",
                    "S237",
                    None,
                ))
            }
        };

        if let Some(scope) = self.domain_scope_stack.last() {
            if let Some(selected) = candidates
                .iter()
                .find(|candidate| &candidate.domain == scope)
            {
                return Ok(selected.clone());
            }
            let available = candidates
                .iter()
                .map(|candidate| candidate.domain.clone())
                .collect::<BTreeSet<_>>();
            return Err(self.parse_error(
                token,
                format!(
                    "Action '{}' is not available in semantic domain '{}'",
                    token.lexeme, scope
                ),
                "S240",
                Some(format!(
                    "This action is available in: {}.",
                    available.into_iter().collect::<Vec<_>>().join(", ")
                )),
            ));
        }

        if candidates.len() == 1 {
            return Ok(candidates[0].clone());
        }

        let domains = candidates
            .iter()
            .map(|candidate| candidate.domain.clone())
            .collect::<BTreeSet<_>>();
        Err(self.parse_error(
            token,
            format!("Ambiguous domain action '{}'", token.lexeme),
            "S240",
            Some(format!(
                "Qualify it with 'Within <domain>:'; candidates: {}.",
                domains.into_iter().collect::<Vec<_>>().join(", ")
            )),
        ))
    }

    fn implicit_property_for(&self, token: &Token, value_type: &Type) -> Result<(String, String)> {
        self.domains
            .infer_property_for_type(value_type)
            .ok_or_else(|| {
                self.parse_error(
                    token,
                    format!(
                        "Cannot infer a unique property for {}; use 'Set <property> of ... to ...'",
                        value_type
                    ),
                    "S236",
                    None,
                )
            })
    }

    fn domain_term(&self, token: &Token) -> Result<DomainTerm> {
        match &token.literal {
            TokenLiteral::DomainTerm(term) => Ok(term.clone()),
            _ => Err(self.parse_error(
                token,
                "Domain token is missing semantic metadata",
                "S237",
                None,
            )),
        }
    }

    fn named_reference_ahead(&self) -> bool {
        let offset = if self.check(TokenKind::Article) { 1 } else { 0 };
        self.tokens
            .get(self.current + offset)
            .is_some_and(|token| token.kind == TokenKind::DomainElement)
    }

    fn phrase_until(&mut self, terminators: &[TokenKind]) -> Result<String> {
        let tokens = self.tokens_until(terminators);
        if tokens.is_empty() {
            return Err(self.parse_error(
                self.peek(),
                "Expected a name or description",
                "S205",
                None,
            ));
        }
        Ok(phrase_from_tokens(&tokens))
    }

    fn tokens_until(&mut self, terminators: &[TokenKind]) -> Vec<Token> {
        let mut tokens = Vec::new();
        while !terminators.contains(&self.peek().kind) {
            tokens.push(self.advance().clone());
        }
        tokens
    }

    fn consume_optional_dot(&mut self) {
        if self.check(TokenKind::Dot) {
            self.advance();
        }
    }

    fn consume(&mut self, kind: TokenKind, message: &str, code: &str) -> Result<Token> {
        if self.check(kind) {
            return Ok(self.advance().clone());
        }
        Err(self.parse_error(self.peek(), message, code, None))
    }

    fn consume_surface(&mut self, word: &str, message: &str, code: &str) -> Result<Token> {
        if self.peek().lexeme.eq_ignore_ascii_case(word) {
            return Ok(self.advance().clone());
        }
        Err(self.parse_error(self.peek(), message, code, None))
    }

    // Consume a contextual English delimiter without reserving its name.
    fn matches_surface(&mut self, word: &str) -> bool {
        if self.peek().kind == TokenKind::Word && self.peek().lexeme.eq_ignore_ascii_case(word) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn matches(&mut self, kinds: &[TokenKind]) -> bool {
        if kinds.iter().any(|kind| self.check(*kind)) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    fn advance(&mut self) -> &Token {
        if !self.check(TokenKind::Eof) {
            self.current += 1;
        }
        self.previous()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current.saturating_sub(1)]
    }

    fn span_from(&self, start: &Token) -> SourceSpan {
        span_between_tokens(start, self.previous())
    }

    fn parse_error(
        &self,
        token: &Token,
        message: impl Into<String>,
        code: &str,
        hint: Option<String>,
    ) -> SemauriError {
        SemauriError::parse(code, message, token.span, hint)
    }
}

fn token_string_literal(token: &Token) -> String {
    match &token.literal {
        TokenLiteral::String(value) => value.clone(),
        _ => token.lexeme.clone(),
    }
}

fn phrase_from_tokens(tokens: &[Token]) -> String {
    tokens
        .iter()
        .map(|token| match &token.literal {
            TokenLiteral::String(value) => value.clone(),
            _ => token.lexeme.clone(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_phrase(value: &str) -> String {
    value
        .split_whitespace()
        .map(|word| {
            if word.chars().all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit()) {
                word.to_string()
            } else {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                    }
                    None => String::new(),
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn span_between_tokens(first: &Token, last: &Token) -> SourceSpan {
    SourceSpan::new(
        first.span.line(),
        first.span.column(),
        last.span.end_line(),
        last.span.end_column(),
    )
}

fn span_between_spans(first: SourceSpan, last: SourceSpan) -> SourceSpan {
    SourceSpan::new(
        first.line(),
        first.column(),
        last.end_line(),
        last.end_column(),
    )
}

fn program_span(statements: &[Stmt]) -> SourceSpan {
    if statements.is_empty() {
        return SourceSpan::point(1, 1);
    }
    span_between_spans(
        statements.first().unwrap().span(),
        statements.last().unwrap().span(),
    )
}

