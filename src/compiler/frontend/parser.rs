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
            if self.matches(&[TokenKind::If]) {
                let branch_start = self.previous().span;
                let condition = self.expression()?;
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
        self.consume(
            TokenKind::Every,
            "Expected 'every' after 'For'",
            "S228",
        )?;
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
        let left = self.additive()?;
        // Natural-language predicates are contextual pairs, not reserved words.
        if self.check(TokenKind::Word)
            && matches!(self.peek().lexeme.to_ascii_lowercase().as_str(), "starts" | "ends")
            && self.tokens.get(self.current + 1).is_some_and(|next|
                next.kind == TokenKind::Word && next.lexeme.eq_ignore_ascii_case("with"))
        {
            let operator = if self.advance().lexeme.eq_ignore_ascii_case("starts") {
                BinaryOperator::StartsWith
            } else {
                BinaryOperator::EndsWith
            };
            self.advance(); // contextual 'with'
            let right = self.additive()?;
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
            let right = self.additive()?;
            let span = span_between_spans(left.span(), right.span());
            return Ok(Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                span,
            });
        }
        if !self.matches(&[TokenKind::Is]) {
            return Ok(left);
        }

        let operator = if self.matches(&[TokenKind::Greater]) {
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
        } else if self.matches(&[TokenKind::Equal]) {
            self.consume(
                TokenKind::To,
                "Expected 'to' after 'equal'",
                "S224",
            )?;
            BinaryOperator::Equal
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
                "Expected 'greater than', 'less than', 'equal to', or 'not equal to' after 'is'",
                "S225",
                Some(
                    "Ordering comparisons may add 'or equal to', for example 'is greater than or equal to'."
                        .to_string(),
                ),
            ));
        };

        let right = self.additive()?;
        let span = span_between_spans(left.span(), right.span());
        Ok(Expr::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
            span,
        })
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

    fn primary(&mut self) -> Result<Expr> {
        let token = self.peek().clone();
        match token.kind {
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
                if matches!(token.lexeme.to_ascii_lowercase().as_str(), "trim" | "words" | "lines" | "capitalize" | "swapcase" | "json" | "characters" | "codepoints" | "codepoint")
                    && self.tokens.get(self.current + 1).is_some_and(|next| next.kind == TokenKind::Of) => {
                let start = self.advance().clone();
                self.consume(TokenKind::Of, "Expected 'of' after a text operation", "S242")?;
                let operand = self.primary()?;
                Ok(Expr::Unary {
                    operator: match start.lexeme.to_ascii_lowercase().as_str() {
                        "trim" => UnaryOperator::Trim,
                        "capitalize" => UnaryOperator::Capitalize,
                        "swapcase" => UnaryOperator::Swapcase,
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
                self.consume(TokenKind::To, "Expected 'to' between range endpoints", "S245")?;
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

