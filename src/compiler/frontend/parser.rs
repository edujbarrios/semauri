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
        if self.matches(&[TokenKind::If]) {
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
        let name = self.consume(
            TokenKind::Word,
            "Expected a variable name after 'Let'",
            "S209",
        )?;
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
        self.consume(
            TokenKind::Colon,
            "Expected ':' after the if condition",
            "S218",
        )?;
        let consequence = self.block_until(&[TokenKind::Otherwise, TokenKind::End])?;
        let alternative = if self.matches(&[TokenKind::Otherwise]) {
            self.consume(
                TokenKind::Colon,
                "Expected ':' after 'Otherwise'",
                "S219",
            )?;
            Some(self.block_until(&[TokenKind::End])?)
        } else {
            None
        };
        self.consume(
            TokenKind::End,
            "Expected 'End' to close the If block",
            "S220",
        )?;
        self.consume_optional_dot();
        Ok(Stmt::If {
            condition,
            consequence,
            alternative,
            span: self.span_from(start),
        })
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
        } else {
            return Err(self.parse_error(
                self.peek(),
                "Expected 'greater than', 'less than', or 'equal to' after 'is'",
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
        let mut expression = self.primary()?;
        loop {
            if self.matches(&[TokenKind::Times]) {
                let right = self.primary()?;
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
                let right = self.primary()?;
                let span = span_between_spans(expression.span(), right.span());
                expression = Expr::Binary {
                    left: Box::new(expression),
                    operator: BinaryOperator::Divide,
                    right: Box::new(right),
                    span,
                };
            } else {
                break;
            }
        }
        Ok(expression)
    }

    fn primary(&mut self) -> Result<Expr> {
        let token = self.peek().clone();
        match token.kind {
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

