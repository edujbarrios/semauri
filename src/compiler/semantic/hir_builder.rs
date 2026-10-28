struct HirBuilder<'a> {
    domains: &'a DomainRegistry,
    symbols: Vec<Symbol>,
    scopes: Vec<HashMap<String, u64>>,
    functions: HashMap<String, (Vec<(u64, Type)>, HirExpr)>,
}

impl<'a> HirBuilder<'a> {
    fn new(domains: &'a DomainRegistry) -> Self {
        Self {
            domains,
            symbols: Vec::new(),
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
        }
    }

    fn build(mut self, program: &Program) -> Result<HirResult> {
        let statements = self.build_statements(&program.statements)?;
        Ok(HirResult {
            statements,
            symbols: self.symbols,
            span: program.span,
        })
    }

    fn build_statements(&mut self, statements: &[Stmt]) -> Result<Vec<HirStmt>> {
        let mut result = Vec::new();
        for statement in statements {
            if let Stmt::DefineFunction { name, parameters, body, span } = statement {
                if self.scopes.len() != 1 {
                    return Err(SemauriError::semantic(
                        "S372", "Functions must be defined at top level",
                        Some(*span), None));
                }
                if self.functions.contains_key(name) {
                    return Err(SemauriError::semantic(
                        "S371", format!("Function '{name}' is already defined"),
                        Some(*span), None));
                }
                self.push_scope();
                let built = (|| {
                    let mut bindings = Vec::new();
                    for (parameter_name, parameter_type) in parameters {
                        let binding = self.create_symbol(
                            parameter_name, "function_parameter", parameter_type.clone(), *span)?;
                        bindings.push((binding.id, parameter_type.clone()));
                    }
                    let body = self.build_expr(body)?;
                    if !pure_function_expression(&body) {
                        return Err(SemauriError::semantic(
                            "S372", "Function bodies must be pure core expressions",
                            Some(body.span()), None));
                    }
                    Ok((bindings, body))
                })();
                self.pop_scope();
                let (bindings, body) = built?;
                self.functions.insert(name.clone(), (bindings, body));
            } else {
                result.push(self.build_stmt(statement)?);
            }
        }
        Ok(result)
    }

    fn build_stmt(&mut self, statement: &Stmt) -> Result<HirStmt> {
        match statement {
            Stmt::DefineFunction { .. } => unreachable!("function declarations handled separately"),
            Stmt::DomainScope { domain, body, span } => {
                self.domains.fetch(domain)?;
                self.push_scope();
                let built = self.build_statements(&body.statements);
                self.pop_scope();
                Ok(HirStmt::DomainScope {
                    domain: domain.clone(),
                    body: built?,
                    span: *span,
                })
            }
            Stmt::Let { name, value, span } => {
                let built_value = self.build_expr(value)?;
                let symbol = self.create_symbol(name, "variable", built_value.ty(), *span)?;
                Ok(HirStmt::Let {
                    symbol_id: symbol.id,
                    name: symbol.name.clone(),
                    source_name: name.clone(),
                    value: built_value,
                    span: *span,
                })
            }
            Stmt::If {
                condition,
                consequence,
                alternative,
                span,
            } => {
                let condition = self.build_expr(condition)?;
                if condition.ty() != Type::Boolean {
                    return Err(SemauriError::semantic(
                        "S316",
                        format!(
                            "If condition must evaluate to boolean, received {}",
                            condition.ty()
                        ),
                        Some(condition.span()),
                        None,
                    ));
                }

                self.push_scope();
                let consequence_built = self.build_statements(&consequence.statements);
                self.pop_scope();
                let consequence_built = consequence_built?;

                let alternative_built = if let Some(alternative) = alternative {
                    self.push_scope();
                    let built = self.build_statements(&alternative.statements);
                    self.pop_scope();
                    Some(built?)
                } else {
                    None
                };

                Ok(HirStmt::If {
                    condition,
                    consequence: consequence_built,
                    alternative: alternative_built,
                    consequence_span: consequence.span,
                    alternative_span: alternative.as_ref().map(|block| block.span),
                    span: *span,
                })
            }
            Stmt::ForEach {
                variable_name,
                binding_span,
                iterable,
                body,
                span,
            } => {
                let iterable = self.build_expr(iterable)?;
                let element_type = if let Type::List(element_type) = iterable.ty() {
                    *element_type
                } else {
                    return Err(SemauriError::semantic(
                        "S322",
                        format!("For every requires a list, received {}", iterable.ty()),
                        Some(iterable.span()),
                        None,
                    ));
                };
                self.push_scope();
                let iterator =
                    self.create_symbol(variable_name, "iterator", element_type, *binding_span)?;
                let body_built = self.build_statements(&body.statements);
                self.pop_scope();
                Ok(HirStmt::ForEach {
                    iterator_symbol_id: iterator.id,
                    iterator_name: iterator.name.clone(),
                    iterator_source_name: variable_name.clone(),
                    iterable,
                    body: body_built?,
                    body_span: body.span,
                    span: *span,
                })
            }
            Stmt::CreateArtifact {
                domain,
                kind,
                subject,
                title,
                span,
            } => {
                self.domains.fetch(domain)?;
                Ok(HirStmt::CreateArtifact {
                    domain: domain.clone(),
                    artifact_kind: kind.clone(),
                    subject: subject.clone(),
                    title: title.clone(),
                    span: *span,
                })
            }
            Stmt::DomainOperation {
                domain,
                operation,
                arguments,
                span,
            } => {
                let expression = Expr::DomainOperation {
                    domain: domain.clone(),
                    operation: operation.clone(),
                    arguments: arguments.clone(),
                    span: *span,
                };
                let hir = self.build_expr(&expression)?;
                if let HirExpr::DomainOperation {
                    domain,
                    operation,
                    arguments,
                    effects,
                    ty,
                    span,
                } = hir
                {
                    Ok(HirStmt::DomainOperation {
                        domain,
                        operation,
                        arguments,
                        effects,
                        return_type: ty,
                        span,
                    })
                } else {
                    unreachable!()
                }
            }
            Stmt::SetTitle { title, span } => Ok(HirStmt::SetTitle {
                title: title.clone(),
                span: *span,
            }),
            Stmt::AddElement {
                domain,
                kind,
                label,
                span,
            } => {
                self.domains.fetch(domain)?;
                Ok(HirStmt::AddElement {
                    domain: domain.clone(),
                    element_kind: kind.clone(),
                    label: label.clone(),
                    span: *span,
                })
            }
            Stmt::SetProperty {
                domain,
                target,
                property,
                value,
                span,
            } => {
                let value = self.build_expr(value)?;
                if let Some(expected) = self.domains.property_type(domain, property) {
                    if expected != value.ty() {
                        return Err(SemauriError::semantic(
                            "S313",
                            format!(
                                "Property '{}' in domain '{}' expects {}, but received {}",
                                property,
                                domain,
                                expected,
                                value.ty()
                            ),
                            Some(value.span()),
                            Some(format!(
                                "Use a {} literal or a variable containing a {}.",
                                expected, expected
                            )),
                        ));
                    }
                }
                let target = match target {
                    Reference::Pronoun { pronoun, span } => HirReference::Pronoun {
                        pronoun: pronoun.clone(),
                        span: *span,
                    },
                    Reference::Named {
                        domain,
                        kind,
                        label,
                        span,
                    } => HirReference::Named {
                        domain: domain.clone(),
                        kind: kind.clone(),
                        label: label.clone(),
                        span: *span,
                    },
                };
                Ok(HirStmt::SetProperty {
                    domain: domain.clone(),
                    target,
                    property: property.clone(),
                    value,
                    span: *span,
                })
            }
        }
    }

    fn build_expr(&mut self, expression: &Expr) -> Result<HirExpr> {
        match expression {
            Expr::Literal { ty, value, span } => Ok(HirExpr::Literal {
                ty: ty.clone(),
                value: value.clone(),
                span: *span,
            }),
            Expr::List { items, span } => {
                let items = items
                    .iter()
                    .map(|item| self.build_expr(item))
                    .collect::<Result<Vec<_>>>()?;
                if items.is_empty() {
                    return Err(SemauriError::semantic(
                        "S320",
                        "A list needs at least one item so its type can be inferred",
                        Some(*span),
                        None,
                    ));
                }
                let element_type = items[0].ty();
                if let Some(mismatch) = items.iter().find(|item| item.ty() != element_type) {
                    return Err(SemauriError::semantic(
                        "S321",
                        format!(
                            "List items must have one type, got {} and {}",
                            element_type,
                            mismatch.ty()
                        ),
                        Some(*span),
                        None,
                    ));
                }
                Ok(HirExpr::List {
                    ty: Type::List(Box::new(element_type)),
                    items,
                    span: *span,
                })
            }
            Expr::Variable { name, span } => {
                let symbol = self.resolve_symbol(name, *span)?;
                Ok(HirExpr::SymbolRef {
                    ty: symbol.ty.clone(),
                    symbol_id: symbol.id,
                    name: symbol.name.clone(),
                    source_name: name.clone(),
                    span: *span,
                })
            }
            Expr::Repeat { count, value, span } => {
                let count = self.build_expr(count)?;
                if count.ty() != Type::Number {
                    return Err(SemauriError::semantic(
                        "S377",
                        format!("Repeat count requires number, received {}", count.ty()),
                        Some(count.span()), None));
                }
                let value = self.build_expr(value)?;
                if value.ty() == Type::Unit {
                    return Err(SemauriError::semantic(
                        "S377", "Repeat element must have a concrete value type",
                        Some(value.span()), None));
                }
                Ok(HirExpr::Repeat {
                    ty: Type::List(Box::new(value.ty())),
                    count: Box::new(count),
                    value: Box::new(value),
                    span: *span,
                })
            }
            Expr::Conditional { condition, consequence, alternative, span } => {
                let condition = self.build_expr(condition)?;
                if condition.ty() != Type::Boolean {
                    return Err(SemauriError::semantic(
                        "S375",
                        format!("Conditional expression requires boolean condition, received {}", condition.ty()),
                        Some(condition.span()), None));
                }
                let consequence = self.build_expr(consequence)?;
                let alternative = self.build_expr(alternative)?;
                if consequence.ty() != alternative.ty() {
                    return Err(SemauriError::semantic(
                        "S376",
                        format!("Conditional branches must have the same type, received {} and {}",
                            consequence.ty(), alternative.ty()),
                        Some(*span), None));
                }
                Ok(HirExpr::Conditional {
                    ty: consequence.ty(),
                    condition: Box::new(condition),
                    consequence: Box::new(consequence),
                    alternative: Box::new(alternative),
                    span: *span,
                })
            }
            Expr::FunctionCall { name, arguments, span } => {
                let (parameters, body) = self.functions.get(name)
                    .cloned().ok_or_else(|| SemauriError::semantic(
                        "S369", format!("Unknown function '{name}' (define it before calling it)"),
                        Some(*span), None))?;
                if arguments.len() != parameters.len() {
                    return Err(SemauriError::semantic(
                        "S374", format!("Function '{name}' expects {} arguments, received {}",
                            parameters.len(), arguments.len()),
                        Some(*span), None));
                }
                let arguments = arguments.iter().map(|arg| self.build_expr(arg))
                    .collect::<Result<Vec<_>>>()?;
                for (argument, (_, param_type)) in arguments.iter().zip(&parameters) {
                    if argument.ty() != *param_type {
                        return Err(SemauriError::semantic(
                            "S370", format!("Function '{name}' expects {param_type}, received {}", argument.ty()),
                            Some(argument.span()), None));
                    }
                }
                Ok(HirExpr::FunctionCall {
                    ty: body.ty(),
                    name: name.clone(),
                    parameters,
                    arguments,
                    body: Box::new(body),
                    span: *span,
                })
            }
            Expr::Reduce { iterator, iterator_span, accumulator, accumulator_span, iterable, initial, body, span } => {
                let iterable = self.build_expr(iterable)?;
                let Type::List(element) = iterable.ty() else {
                    return Err(SemauriError::semantic(
                        "S379", format!("'reduce every' requires a typed list, received {}", iterable.ty()),
                        Some(iterable.span()), None));
                };
                let initial = self.build_expr(initial)?;
                if initial.ty() == Type::Unit {
                    return Err(SemauriError::semantic(
                        "S380", "Reduce seed must produce a value", Some(initial.span()), None));
                }
                let ty = initial.ty();
                self.push_scope();
                let result = (|| {
                    let item = self.create_symbol(iterator, "reduce_iterator", *element, *iterator_span)?;
                    let acc = self.create_symbol(accumulator, "reduce_accumulator", ty.clone(), *accumulator_span)?;
                    let body = self.build_expr(body)?;
                    if body.ty() != ty {
                        return Err(SemauriError::semantic(
                            "S380", format!("Reduce body must return accumulator type {ty}, received {}", body.ty()),
                            Some(body.span()), None));
                    }
                    if !pure_function_expression(&body) {
                        return Err(SemauriError::semantic(
                            "S381", "Reduce body must be a pure expression",
                            Some(body.span()), None));
                    }
                    Ok(HirExpr::Reduce {
                        ty,
                        iterator_symbol_id: item.id,
                        accumulator_symbol_id: acc.id,
                        iterator_name: iterator.clone(),
                        accumulator_name: accumulator.clone(),
                        iterable: Box::new(iterable),
                        initial: Box::new(initial),
                        body: Box::new(body),
                        span: *span,
                    })
                })();
                self.pop_scope();
                result
            }
            Expr::ZipTransform {
                left_name, left_span, left_list,
                right_name, right_span, right_list, body, span
            } => {
                let left_list = self.build_expr(left_list)?;
                let Type::List(left_element) = left_list.ty() else {
                    return Err(SemauriError::semantic(
                        "S383", format!("Zip expects first list, received {}", left_list.ty()),
                        Some(left_list.span()), None));
                };
                let right_list = self.build_expr(right_list)?;
                let Type::List(right_element) = right_list.ty() else {
                    return Err(SemauriError::semantic(
                        "S383", format!("Zip expects second list, received {}", right_list.ty()),
                        Some(right_list.span()), None));
                };
                self.push_scope();
                let result = (|| {
                    let left_symbol = self.create_symbol(left_name, "zip_iterator", *left_element, *left_span)?;
                    let right_symbol = self.create_symbol(right_name, "zip_iterator", *right_element, *right_span)?;
                    let body = self.build_expr(body)?;
                    if body.ty() == Type::Unit {
                        return Err(SemauriError::semantic(
                            "S384", "Zip body must produce a concrete value",
                            Some(body.span()), None));
                    }
                    Ok(HirExpr::ZipTransform {
                        ty: Type::List(Box::new(body.ty())),
                        left_symbol_id: left_symbol.id, left_name: left_name.clone(),
                        left_list: Box::new(left_list), right_symbol_id: right_symbol.id,
                        right_name: right_name.clone(), right_list: Box::new(right_list),
                        body: Box::new(body), span: *span,
                    })
                })();
                self.pop_scope();
                result
            }
            Expr::ListTransform { kind, iterator, binding_span, iterable, body, span } => {
                let iterable = self.build_expr(iterable)?;
                let Type::List(element) = iterable.ty() else {
                    return Err(SemauriError::semantic(
                        "S365",
                        format!("'{} every' requires a list input, received {}", kind.as_str(), iterable.ty()),
                        Some(iterable.span()),
                        None,
                    ));
                };
                self.push_scope();
                let result = (|| {
                    let binding = self.create_symbol(iterator, "list_iterator", *element, *binding_span)?;
                    let body = self.build_expr(body)?;
                    if matches!(kind, ListTransformKind::Filter | ListTransformKind::Partition | ListTransformKind::Count | ListTransformKind::Any | ListTransformKind::All)
                        && body.ty() != Type::Boolean {
                        return Err(SemauriError::semantic(
                            "S366",
                            format!("Filter predicate must be boolean, received {}", body.ty()),
                            Some(body.span()),
                            None,
                        ));
                    }
                    if *kind == ListTransformKind::Map && body.ty() == Type::Unit {
                        return Err(SemauriError::semantic(
                            "S366",
                            "Map body must produce a value rather than unit",
                            Some(body.span()),
                            None,
                        ));
                    }
                    let ty = match kind {
                        ListTransformKind::Map => Type::List(Box::new(body.ty())),
                        ListTransformKind::Filter => iterable.ty(),
                        ListTransformKind::Partition => Type::List(Box::new(iterable.ty())),
                        ListTransformKind::Count => Type::Number,
                        ListTransformKind::Any | ListTransformKind::All => Type::Boolean,
                    };
                    Ok(HirExpr::ListTransform {
                        ty,
                        kind: *kind,
                        iterator_symbol_id: binding.id,
                        iterator_name: iterator.clone(),
                        iterable: Box::new(iterable),
                        body: Box::new(body),
                        span: *span,
                    })
                })();
                self.pop_scope();
                result
            }
            Expr::Unary {
                operator,
                operand,
                span,
            } => {
                let operand = self.build_expr(operand)?;
                let ty = match operator {
                    UnaryOperator::Not => {
                        if operand.ty() != Type::Boolean {
                            return Err(SemauriError::semantic(
                                "S319",
                                format!(
                                    "Logical 'not' requires a boolean operand, received {}",
                                    operand.ty()
                                ),
                                Some(*span),
                                None,
                            ));
                        }
                        Type::Boolean
                    }
                    UnaryOperator::Uppercase | UnaryOperator::Lowercase | UnaryOperator::Capitalize | UnaryOperator::Swapcase | UnaryOperator::Uncapitalize => {
                        if operand.ty() != Type::String {
                            return Err(SemauriError::semantic(
                                "S351",
                                format!(
                                    "String case transformation requires a string, received {}",
                                    operand.ty()
                                ),
                                Some(*span),
                                None,
                            ));
                        }
                        Type::String
                    }
                    UnaryOperator::Characters | UnaryOperator::Codepoints
                    | UnaryOperator::Codepoint | UnaryOperator::CharacterFrom => {
                        let expected = if *operator == UnaryOperator::CharacterFrom {
                            Type::Number
                        } else {
                            Type::String
                        };
                        if operand.ty() != expected {
                            return Err(SemauriError::semantic(
                                "S408",
                                format!("'{}' requires {}, received {}", operator.as_str(), expected, operand.ty()),
                                Some(*span), None,
                            ));
                        }
                        match operator {
                            UnaryOperator::Characters => Type::List(Box::new(Type::String)),
                            UnaryOperator::Codepoints => Type::List(Box::new(Type::Number)),
                            UnaryOperator::Codepoint => Type::Number,
                            UnaryOperator::CharacterFrom => Type::String,
                            _ => unreachable!(),
                        }
                    }
                    UnaryOperator::JsonKeys => {
                        if operand.ty() != Type::String {
                            return Err(SemauriError::semantic(
                                "S398",
                                format!("'keys of json' needs a string document, received {}", operand.ty()),
                                Some(*span), None,
                            ));
                        }
                        Type::List(Box::new(Type::String))
                    }
                    UnaryOperator::DecodeJsonString | UnaryOperator::DecodeJsonNumber
                    | UnaryOperator::DecodeJsonBoolean | UnaryOperator::DecodeJsonStrings
                    | UnaryOperator::DecodeJsonNumbers | UnaryOperator::DecodeJsonBooleans => {
                        if operand.ty() != Type::String {
                            return Err(SemauriError::semantic(
                                "S396",
                                format!("'{}' needs JSON string input, received {}", operator.as_str(), operand.ty()),
                                Some(*span),
                                Some("Encode values with 'json of' or provide a JSON string.".to_string()),
                            ));
                        }
                        json_decode_type(operator).expect("known JSON decode operator")
                    }
                    UnaryOperator::ParseNumber | UnaryOperator::ParseBoolean => {
                        if operand.ty() != Type::String {
                            return Err(SemauriError::semantic(
                                "S394",
                                format!("'{}' needs a string, received {}", operator.as_str(), operand.ty()),
                                Some(*span), None,
                            ));
                        }
                        if *operator == UnaryOperator::ParseNumber { Type::Number } else { Type::Boolean }
                    }
                    UnaryOperator::Json => {
                        if !is_json_serializable_type(&operand.ty()) {
                            return Err(SemauriError::semantic(
                                "S390",
                                format!("'json of' requires a serializable value, received {}", operand.ty()),
                                Some(*span),
                                Some("Use numbers, booleans, strings, colors, or lists of those values.".into()),
                            ));
                        }
                        Type::String
                    }
                    UnaryOperator::Trim | UnaryOperator::Words | UnaryOperator::Lines => {
                        if operand.ty() != Type::String {
                            return Err(SemauriError::semantic(
                                "S351",
                                format!("Text operation '{}' requires a string, received {}", operator.as_str(), operand.ty()),
                                Some(*span),
                                None,
                            ));
                        }
                        match operator {
                            UnaryOperator::Trim => Type::String,
                            UnaryOperator::Words | UnaryOperator::Lines => Type::List(Box::new(Type::String)),
                            _ => unreachable!(),
                        }
                    }
                    UnaryOperator::Any | UnaryOperator::All => {
                        if operand.ty() != Type::List(Box::new(Type::Boolean)) {
                            return Err(SemauriError::semantic(
                                "S356",
                                format!("Boolean reduction requires list<boolean>, received {}", operand.ty()),
                                Some(*span),
                                None,
                            ));
                        }
                        Type::Boolean
                    }
                    UnaryOperator::Sum | UnaryOperator::Average | UnaryOperator::Minimum | UnaryOperator::Maximum | UnaryOperator::Median => {
                        if operand.ty() != Type::List(Box::new(Type::Number)) {
                            return Err(SemauriError::semantic(
                                "S353",
                                format!("Numeric aggregation requires list<number>, received {}", operand.ty()),
                                Some(*span),
                                None,
                            ));
                        }
                        Type::Number
                    }
                    UnaryOperator::First | UnaryOperator::Last => {
                        match operand.ty() {
                            Type::List(element) => *element,
                            ty => return Err(SemauriError::semantic(
                                "S361",
                                format!("'first of' and 'last of' require a list, received {ty}"),
                                Some(*span),
                                None,
                            )),
                        }
                    }
                    UnaryOperator::Product => {
                        if operand.ty() != Type::List(Box::new(Type::Number)) {
                            return Err(SemauriError::semantic(
                                "S353",
                                format!("'product of' requires list<number>, received {}", operand.ty()),
                                Some(*span), None));
                        }
                        Type::Number
                    }
                    UnaryOperator::Absolute => {
                        if operand.ty() != Type::Number {
                            return Err(SemauriError::semantic(
                                "S386",
                                format!("'absolute of' requires number, received {}", operand.ty()),
                                Some(*span), None));
                        }
                        Type::Number
                    }
                    UnaryOperator::Sign => {
                        if operand.ty() != Type::Number {
                            return Err(SemauriError::semantic(
                                "S417",
                                format!("'sign of' requires number, received {}", operand.ty()),
                                Some(*span), None));
                        }
                        Type::Number
                    }
                    UnaryOperator::Floor | UnaryOperator::Ceiling => {
                        if operand.ty() != Type::Number {
                            return Err(SemauriError::semantic(
                                "S419",
                                format!("'floor of' and 'ceiling of' require number, received {}", operand.ty()),
                                Some(*span), None));
                        }
                        Type::Number
                    }
                    UnaryOperator::Round => {
                        if operand.ty() != Type::Number {
                            return Err(SemauriError::semantic(
                                "S427",
                                format!("'round of' requires number, received {}", operand.ty()),
                                Some(*span), None));
                        }
                        Type::Number
                    }
                    UnaryOperator::Truncate => {
                        if operand.ty() != Type::Number {
                            return Err(SemauriError::semantic(
                                "S429",
                                format!("'truncate of' requires number, received {}", operand.ty()),
                                Some(*span), None));
                        }
                        Type::Number
                    }
                    UnaryOperator::Factorial => {
                        if operand.ty() != Type::Number {
                            return Err(SemauriError::semantic(
                                "S431",
                                format!("'factorial of' requires number, received {}", operand.ty()),
                                Some(*span), None));
                        }
                        Type::Number
                    }
                    UnaryOperator::Prime => {
                        if operand.ty() != Type::Number {
                            return Err(SemauriError::semantic(
                                "S435",
                                format!("'prime of' requires number, received {}", operand.ty()),
                                Some(*span), None));
                        }
                        Type::Boolean
                    }
                    UnaryOperator::Cube => {
                        if operand.ty() != Type::Number {
                            return Err(SemauriError::semantic(
                                "S433",
                                format!("'cube of' requires number, received {}", operand.ty()),
                                Some(*span), None));
                        }
                        Type::Number
                    }
                    UnaryOperator::Square => {
                        if operand.ty() != Type::Number {
                            return Err(SemauriError::semantic(
                                "S421",
                                format!("'square of' requires number, received {}", operand.ty()),
                                Some(*span), None));
                        }
                        Type::Number
                    }
                    UnaryOperator::Digits => {
                        if operand.ty() != Type::Number {
                            return Err(SemauriError::semantic(
                                "S441",
                                format!("'digits of' requires number, received {}", operand.ty()),
                                Some(*span), None));
                        }
                        Type::Number
                    }
                    UnaryOperator::Reciprocal => {
                        if operand.ty() != Type::Number {
                            return Err(SemauriError::semantic(
                                "S439",
                                format!("'reciprocal of' requires number, received {}", operand.ty()),
                                Some(*span), None));
                        }
                        Type::Number
                    }
                    UnaryOperator::CubeRoot => {
                        if operand.ty() != Type::Number {
                            return Err(SemauriError::semantic(
                                "S437",
                                format!("'cbrt of' requires number, received {}", operand.ty()),
                                Some(*span), None));
                        }
                        Type::Number
                    }
                    UnaryOperator::SquareRoot => {
                        if operand.ty() != Type::Number {
                            return Err(SemauriError::semantic(
                                "S423",
                                format!("'sqrt of' requires number, received {}", operand.ty()),
                                Some(*span), None));
                        }
                        Type::Number
                    }
                    UnaryOperator::Even | UnaryOperator::Odd => {
                        if operand.ty() != Type::Number {
                            return Err(SemauriError::semantic(
                                "S425",
                                format!("'even of' and 'odd of' require number, received {}", operand.ty()),
                                Some(*span), None));
                        }
                        Type::Boolean
                    }
                    UnaryOperator::Distinct => {
                        match operand.ty() {
                            Type::List(_) => operand.ty(),
                            ty => return Err(SemauriError::semantic(
                                "S360",
                                format!("'distinct of' requires a list, received {ty}"),
                                Some(*span),
                                None,
                            )),
                        }
                    }
                    UnaryOperator::Sorted => {
                        match operand.ty() {
                            Type::List(element) if matches!(element.as_ref(), Type::Number | Type::String | Type::Boolean) => operand.ty(),
                            ty => return Err(SemauriError::semantic(
                                "S359",
                                format!("'sorted of' requires list<number>, list<string> or list<boolean>, received {ty}"),
                                Some(*span),
                                None,
                            )),
                        }
                    }
                    UnaryOperator::Flatten => match operand.ty() {
                        Type::List(outer) => match *outer {
                            Type::List(inner) => Type::List(inner),
                            other => return Err(SemauriError::semantic(
                                "S385",
                                format!("'flatten of' requires a list of lists, received list<{other}>"),
                                Some(*span), None,
                            )),
                        },
                        other => return Err(SemauriError::semantic(
                            "S385",
                            format!("'flatten of' requires a list of lists, received {other}"),
                            Some(*span), None,
                        )),
                    },
                    UnaryOperator::Reverse => {
                        match operand.ty() {
                            Type::String | Type::List(_) => operand.ty(),
                            ty => return Err(SemauriError::semantic(
                                "S352",
                                format!("'reverse of' requires a string or list, received {ty}"),
                                Some(*span),
                                None,
                            )),
                        }
                    }
                    UnaryOperator::Length => {
                        if !matches!(operand.ty(), Type::String | Type::List(_)) {
                            return Err(SemauriError::semantic(
                                "S343",
                                format!(
                                    "'Length of' requires a string or list, received {}",
                                    operand.ty()
                                ),
                                Some(*span),
                                None,
                            ));
                        }
                        Type::Number
                    }
                    UnaryOperator::Negate => {
                        if operand.ty() != Type::Number {
                            return Err(SemauriError::semantic(
                                "S314",
                                format!(
                                    "Unary 'minus' requires a number operand, received {}",
                                    operand.ty()
                                ),
                                Some(*span),
                                None,
                            ));
                        }
                        Type::Number
                    }
                };
                Ok(HirExpr::Unary {
                    ty,
                    operator: operator.clone(),
                    operand: Box::new(operand),
                    span: *span,
                })
            }
            Expr::Binary {
                left,
                operator,
                right,
                span,
            } => {
                let left = self.build_expr(left)?;
                let right = self.build_expr(right)?;
                let result_type = binary_type(operator, &left.ty(), &right.ty(), *span)?;
                Ok(HirExpr::Binary {
                    ty: result_type,
                    left: Box::new(left),
                    operator: operator.clone(),
                    right: Box::new(right),
                    span: *span,
                })
            }
            Expr::DomainOperation {
                domain,
                operation,
                arguments,
                span,
            } => {
                let domain_spec = self.domains.fetch(domain)?;
                let operation_spec = domain_spec.operation(operation).cloned().ok_or_else(|| {
                    SemauriError::semantic(
                        "S326",
                        format!(
                            "Domain '{}' does not define operation '{}'",
                            domain, operation
                        ),
                        Some(*span),
                        None,
                    )
                })?;
                let mut built_arguments = BTreeMap::new();
                for (name, value) in arguments {
                    built_arguments.insert(name.clone(), self.build_expr(value)?);
                }

                for segment in &operation_spec.pattern {
                    let PatternSegment::Slot { name, ty } = segment else {
                        continue;
                    };
                    let value = built_arguments.get(name).ok_or_else(|| {
                        SemauriError::semantic(
                            "S330",
                            format!(
                                "Operation '{}' is missing argument {:?}",
                                operation_spec.name, name
                            ),
                            Some(*span),
                            None,
                        )
                    })?;
                    if let Some(expected) = ty {
                        if Type::assignment_kind(&value.ty(), expected).is_none() {
                            return Err(SemauriError::semantic(
                                "S329",
                                format!(
                                    "Operation '{}' argument '{}' expects {}, but received {}",
                                    operation_spec.name,
                                    name,
                                    expected,
                                    value.ty()
                                ),
                                Some(value.span()),
                                Some(format!(
                                    "Provide a {} expression for '{}'.",
                                    expected, name
                                )),
                            ));
                        }
                    }
                }

                let mut promoted = BTreeMap::new();
                for (name, value) in built_arguments {
                    let expected = operation_spec.pattern.iter().find_map(|segment| match segment {
                        PatternSegment::Slot { name: slot_name, ty } if slot_name == &name => {
                            ty.clone()
                        }
                        _ => None,
                    });
                    if let Some(expected) = expected {
                        if Type::assignment_kind(&value.ty(), &expected)
                            == Some(AssignmentKind::Promote)
                        {
                            promoted.insert(
                                name,
                                HirExpr::Promote {
                                    ty: expected,
                                    from_type: value.ty(),
                                    span: value.span(),
                                    value: Box::new(value),
                                },
                            );
                            continue;
                        }
                    }
                    promoted.insert(name, value);
                }

                validate_domain_operation(
                    domain,
                    &operation_spec,
                    &promoted,
                    *span,
                )?;

                Ok(HirExpr::DomainOperation {
                    ty: operation_spec.return_type,
                    domain: domain.clone(),
                    operation: operation.clone(),
                    arguments: promoted,
                    effects: operation_spec.effects,
                    span: *span,
                })
            }
        }
    }

    fn create_symbol(
        &mut self,
        name: &str,
        kind: &str,
        ty: Type,
        definition_span: SourceSpan,
    ) -> Result<Symbol> {
        let key = name.to_lowercase();
        if self.scopes.last().unwrap().contains_key(&key) {
            return Err(SemauriError::semantic(
                "S311",
                format!("Variable '{name}' is already defined in this scope"),
                Some(definition_span),
                Some("Choose a different name or reuse the existing variable.".to_string()),
            ));
        }
        let symbol = Symbol {
            id: self.symbols.len() as u64 + 1,
            name: key.clone(),
            kind: kind.to_string(),
            ty,
            definition_span,
        };
        self.symbols.push(symbol.clone());
        self.scopes.last_mut().unwrap().insert(key, symbol.id);
        Ok(symbol)
    }

    fn resolve_symbol(&self, name: &str, span: SourceSpan) -> Result<Symbol> {
        let key = name.to_lowercase();
        for scope in self.scopes.iter().rev() {
            if let Some(id) = scope.get(&key) {
                return Ok(self.symbols[*id as usize - 1].clone());
            }
        }
        Err(SemauriError::semantic(
            "S312",
            format!("Unknown variable '{name}'"),
            Some(span),
            Some(format!("Declare it first with 'Let {name} be ...'.")),
        ))
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }
}

fn binary_type(
    operator: &BinaryOperator,
    left: &Type,
    right: &Type,
    span: SourceSpan,
) -> Result<Type> {
    match operator {
        BinaryOperator::Add => {
            if left == &Type::Number && right == &Type::Number {
                Ok(Type::Number)
            } else if left == &Type::String && right == &Type::String {
                Ok(Type::String)
            } else if matches!((left, right), (Type::List(a), Type::List(b)) if a == b) {
                Ok(left.clone())
            } else {
                Err(SemauriError::semantic(
                    "S314",
                    format!(
                        "Operator '{}' requires matching number, string, or list operands, got {} and {}",
                        operator.as_str(),
                        left,
                        right
                    ),
                    Some(span),
                    None,
                ))
            }
        }
        BinaryOperator::Subtract
        | BinaryOperator::Multiply
        | BinaryOperator::Divide
        | BinaryOperator::Modulo => {
            if left == &Type::Number && right == &Type::Number {
                Ok(Type::Number)
            } else {
                Err(SemauriError::semantic(
                    "S314",
                    format!(
                        "Operator '{}' requires number operands, got {} and {}",
                        operator.as_str(),
                        left,
                        right
                    ),
                    Some(span),
                    None,
                ))
            }
        }
        BinaryOperator::GreaterThan
        | BinaryOperator::LessThan
        | BinaryOperator::GreaterThanOrEqual
        | BinaryOperator::LessThanOrEqual => {
            if left == &Type::Number && right == &Type::Number {
                Ok(Type::Boolean)
            } else {
                Err(SemauriError::semantic(
                    "S314",
                    format!(
                        "Operator '{}' requires number operands, got {} and {}",
                        operator.as_str(),
                        left,
                        right
                    ),
                    Some(span),
                    None,
                ))
            }
        }
        BinaryOperator::Range => {
            if left == &Type::Number && right == &Type::Number {
                Ok(Type::List(Box::new(Type::Number)))
            } else {
                Err(SemauriError::semantic(
                    "S367",
                    format!("'range from ... to ...' requires two numeric endpoints, received {left} and {right}"),
                    Some(span),
                    None,
                ))
            }
        }
        BinaryOperator::Take | BinaryOperator::Drop => match (left, right) {
            (Type::Number, Type::List(_)) | (Type::Number, Type::String) => Ok(right.clone()),
            _ => Err(SemauriError::semantic(
                "S363",
                format!("'take' and 'drop' require a number and list or string, received {left} and {right}"),
                Some(span),
                None,
            )),
        },
        BinaryOperator::AtIndex => {
            match (left, right) {
                (Type::List(element), Type::Number) => Ok((**element).clone()),
                _ => Err(SemauriError::semantic(
                    "S345",
                    format!(
                        "'at' expects a list and a number index, received {} and {}",
                        left, right
                    ),
                    Some(span),
                    None,
                )),
            }
        }
        BinaryOperator::Split => {
            if left == &Type::String && right == &Type::String {
                Ok(Type::List(Box::new(Type::String)))
            } else {
                Err(SemauriError::semantic(
                    "S392",
                    format!("'split of ... by ...' needs two strings, received {left} and {right}"),
                    Some(span), None,
                ))
            }
        }
        BinaryOperator::Join => {
            if left == &Type::List(Box::new(Type::String)) && right == &Type::String {
                Ok(Type::String)
            } else {
                Err(SemauriError::semantic(
                    "S392",
                    format!("'join of ... with ...' needs list<string> and string, received {left} and {right}"),
                    Some(span), None,
                ))
            }
        }
        BinaryOperator::JsonFieldString | BinaryOperator::JsonFieldNumber
        | BinaryOperator::JsonFieldBoolean | BinaryOperator::JsonFieldStrings
        | BinaryOperator::JsonFieldNumbers | BinaryOperator::JsonFieldBooleans => {
            if left != &Type::String || right != &Type::String {
                return Err(SemauriError::semantic(
                    "S400",
                    format!("'json TYPE at KEY in DOCUMENT' needs string key and JSON document, received {left} and {right}"),
                    Some(span), None,
                ));
            }
            Ok(json_field_type(operator).expect("known JSON field type"))
        }
        BinaryOperator::TextRemoveAll => {
            if left == &Type::String && right == &Type::String {
                Ok(Type::String)
            } else {
                Err(SemauriError::semantic(
                    "S412",
                    format!("'remove all of NEEDLE from TEXT' requires two strings, received {left} and {right}"),
                    Some(span), None,
                ))
            }
        }
        BinaryOperator::CharacterAt => {
            if left == &Type::Number && right == &Type::String {
                Ok(Type::String)
            } else {
                Err(SemauriError::semantic(
                    "S410",
                    format!("'character at INDEX in TEXT' requires number index and string text, received {left} and {right}"),
                    Some(span), None,
                ))
            }
        }
        BinaryOperator::TextPositions => {
            if left == &Type::String && right == &Type::String {
                Ok(Type::List(Box::new(Type::Number)))
            } else {
                Err(SemauriError::semantic(
                    "S414",
                    format!("'positions of ... in ...' requires two strings, received {left} and {right}"),
                    Some(span), None,
                ))
            }
        }
        BinaryOperator::TextLastPosition => {
            if left == &Type::String && right == &Type::String {
                Ok(Type::Number)
            } else {
                Err(SemauriError::semantic(
                    "S414",
                    format!("'last position of ... in ...' requires two strings, received {left} and {right}"),
                    Some(span), None,
                ))
            }
        }
        BinaryOperator::TextOccurrences | BinaryOperator::TextPosition => {
            if left == &Type::String && right == &Type::String {
                Ok(Type::Number)
            } else {
                Err(SemauriError::semantic(
                    "S402",
                    format!("'{} of ... in ...' requires two strings, received {left} and {right}", operator.as_str()),
                    Some(span), None,
                ))
            }
        }
        BinaryOperator::JsonPointer => {
            if left == &Type::String && right == &Type::String {
                Ok(Type::String)
            } else {
                Err(SemauriError::semantic(
                    "S406",
                    format!("'json at pointer ... in ...' needs a string pointer and a JSON string document, received {left} and {right}"),
                    Some(span), None,
                ))
            }
        }
        BinaryOperator::JsonHasKey => {
            if left == &Type::String && right == &Type::String {
                Ok(Type::Boolean)
            } else {
                Err(SemauriError::semantic(
                    "S398",
                    format!("'has json key ... in ...' needs string key and document, received {left} and {right}"),
                    Some(span), None,
                ))
            }
        }
        BinaryOperator::StartsWith | BinaryOperator::EndsWith => {
            if left == &Type::String && right == &Type::String {
                Ok(Type::Boolean)
            } else {
                Err(SemauriError::semantic(
                    "S391",
                    format!("'{}' requires two strings, received {} and {}",
                        operator.as_str(), left, right),
                    Some(span),
                    None,
                ))
            }
        }
        BinaryOperator::Contains => {
            match left {
                Type::String if right == &Type::String => Ok(Type::Boolean),
                Type::List(element) if element.as_ref() == right => Ok(Type::Boolean),
                _ => Err(SemauriError::semantic(
                    "S344",
                    format!(
                        "'contains' requires a string and a string or list<T> and T, got {} and {}",
                        left, right
                    ),
                    Some(span),
                    None,
                )),
            }
        }
        BinaryOperator::Equal | BinaryOperator::NotEqual => {
            if left == right {
                Ok(Type::Boolean)
            } else {
                Err(SemauriError::semantic(
                    "S315",
                    format!(
                        "Equality requires operands of the same type, got {} and {}",
                        left, right
                    ),
                    Some(span),
                    None,
                ))
            }
        }
        BinaryOperator::And | BinaryOperator::Or => {
            if left == &Type::Boolean && right == &Type::Boolean {
                Ok(Type::Boolean)
            } else {
                Err(SemauriError::semantic(
                    "S319",
                    format!(
                        "Logical operators require boolean operands, got {} and {}",
                        left, right
                    ),
                    Some(span),
                    None,
                ))
            }
        }
    }
}

fn literal_data(expr: &HirExpr) -> Option<ValueData> {
    match expr {
        HirExpr::Literal { value, .. } => Some(match value {
            LiteralValue::Number(value) => ValueData::Number(value.clone()),
            LiteralValue::Boolean(value) => ValueData::Boolean(*value),
            LiteralValue::String(value) => ValueData::String(value.clone()),
            LiteralValue::Color(value) => ValueData::Color(value.clone()),
        }),
        HirExpr::Promote { value, .. } => literal_data(value),
        _ => None,
    }
}

fn validate_domain_operation(
    domain: &str,
    operation: &OperationSpec,
    arguments: &BTreeMap<String, HirExpr>,
    span: SourceSpan,
) -> Result<()> {
    if domain == "filesystem" {
        let path_names: &[&str] = match operation.name.as_str() {
            "copy" | "move" => &["source", "destination"],
            "write" | "append" | "make_directory" | "touch" | "delete" => &["path"],
            _ => &[],
        };
        for name in path_names {
            if let Some(ValueData::String(path)) = arguments.get(*name).and_then(literal_data) {
                if path.trim().is_empty() {
                    return Err(SemauriError::semantic(
                        "S340",
                        "filesystem path must not be empty",
                        Some(span),
                        Some("Use a non-empty filesystem path.".to_string()),
                    ));
                }
            }
        }
    }

    if domain != "ml" {
        return Ok(());
    }

    let number = |name: &str| -> Option<NumberValue> {
        arguments.get(name).and_then(literal_data).and_then(|value| match value {
            ValueData::Number(value) => Some(value),
            _ => None,
        })
    };
    let string = |name: &str| -> Option<String> {
        arguments.get(name).and_then(literal_data).and_then(|value| match value {
            ValueData::String(value) => Some(value),
            _ => None,
        })
    };

    let positive_integer = |name: &str, display: &str, code: &str| -> Result<()> {
        if let Some(value) = number(name) {
            if !(value.is_positive() && value.is_integer()) {
                return Err(SemauriError::semantic(
                    code,
                    format!("{display} must be a positive integer"),
                    Some(span),
                    None,
                ));
            }
        }
        Ok(())
    };
    let positive_number = |name: &str, display: &str, code: &str| -> Result<()> {
        if let Some(value) = number(name) {
            if !value.is_positive() {
                return Err(SemauriError::semantic(
                    code,
                    format!("{display} must be greater than zero"),
                    Some(span),
                    None,
                ));
            }
        }
        Ok(())
    };
    let non_negative_integer = |name: &str, display: &str, code: &str| -> Result<()> {
        if let Some(value) = number(name) {
            if value.is_negative() || !value.is_integer() {
                return Err(SemauriError::semantic(
                    code,
                    format!("{display} must be a non-negative integer"),
                    Some(span),
                    None,
                ));
            }
        }
        Ok(())
    };

    match operation.name.as_str() {
        "source_dataset_revision" | "resolve_model_revision" => {
            let id_arg = if operation.name == "source_dataset_revision" {
                "source"
            } else {
                "identifier"
            };
            for name in [id_arg, "revision"] {
                if let Some(value) = string(name) {
                    if value.trim().is_empty() {
                        return Err(SemauriError::semantic(
                            "S348",
                            format!(
                                "Versioned ML input '{}' must be a non-empty string",
                                name
                            ),
                            arguments.get(name).map(HirExpr::span).or(Some(span)),
                            Some(
                                "Provide an explicit non-empty source and revision identifier."
                                    .to_string(),
                            ),
                        ));
                    }
                }
            }
        }
        "configure_training" | "plan_training" => {
            positive_integer("epochs", "epochs", "S336")?;
            if let Some(optimizer) = string("optimizer") {
                if !matches!(optimizer.to_lowercase().as_str(), "adam" | "adamw" | "sgd") {
                    return Err(SemauriError::semantic(
                        "S336",
                        format!("Unsupported optimizer '{optimizer}'"),
                        Some(span),
                        Some("Supported optimizers: adam, adamw, sgd.".to_string()),
                    ));
                }
            }
            positive_number("learning_rate", "learning rate", "S336")?;
            positive_integer("batch_size", "batch size", "S336")?;
            non_negative_integer("seed", "seed", "S336")?;
            if operation.name == "plan_training" {
                if let Some(precision) = string("precision") {
                    if !matches!(precision.to_lowercase().as_str(), "fp32" | "fp16" | "bf16") {
                        return Err(SemauriError::semantic(
                            "S338",
                            format!("Unsupported training precision '{precision}'"),
                            Some(span),
                            Some("Supported precisions: fp32, fp16, bf16.".to_string()),
                        ));
                    }
                }
                positive_integer(
                    "gradient_accumulation",
                    "gradient accumulation",
                    "S338",
                )?;
                positive_integer("checkpoint_every", "checkpoint interval", "S338")?;
            }
        }
        "budget_resources" => {
            positive_number("memory_gb", "memory budget", "S342")?;
            positive_integer("workers", "worker count", "S342")?;
        }
        "transform_dataset" => {
            if let Some(transform) = string("transform") {
                if transform.trim().is_empty() {
                    return Err(SemauriError::semantic(
                        "S340",
                        "dataset transform must be a non-empty string",
                        Some(span),
                        None,
                    ));
                }
            }
        }
        "split_dataset" => {
            if let Some(ratio) = number("ratio") {
                if !(ratio.as_f64() > 0.0 && ratio.as_f64() < 1.0) {
                    return Err(SemauriError::semantic(
                        "S340",
                        "dataset split ratio must be greater than zero and less than one",
                        Some(span),
                        None,
                    ));
                }
            }
            non_negative_integer("seed", "dataset split seed", "S340")?;
        }
        "evaluate" => {
            if let Some(metric) = string("metric") {
                if !matches!(
                    metric.to_lowercase().as_str(),
                    "accuracy" | "precision" | "recall" | "f1" | "loss" | "perplexity"
                ) {
                    return Err(SemauriError::semantic(
                        "S341",
                        format!("Unsupported evaluation metric '{metric}'"),
                        Some(span),
                        Some(
                            "Supported metrics: accuracy, precision, recall, f1, loss, perplexity."
                                .to_string(),
                        ),
                    ));
                }
            }
        }
        "build_cnn" => {
            positive_integer("classes", "class count", "S337")?;
            positive_integer("input_channels", "input channel count", "S337")?;
        }
        "freeze_component" => {
            if let Some(component) = string("component") {
                if component.trim().is_empty() {
                    return Err(SemauriError::semantic(
                        "S337",
                        "component must be a non-empty string",
                        Some(span),
                        None,
                    ));
                }
            }
        }
        "apply_lora" => {
            positive_integer("rank", "LoRA rank", "S337")?;
            positive_number("alpha", "LoRA alpha", "S337")?;
        }
        "apply_qlora" => {
            positive_integer("rank", "QLoRA rank", "S339")?;
            positive_number("alpha", "QLoRA alpha", "S339")?;
            if let Some(bits) = number("quantization_bits") {
                if bits != NumberValue::Int(4) && bits != NumberValue::Int(8) {
                    return Err(SemauriError::semantic(
                        "S339",
                        format!("Unsupported QLoRA quantization '{}' bits", bits),
                        Some(span),
                        Some("Supported quantization widths: 4, 8 bits.".to_string()),
                    ));
                }
            }
            if let Some(targets) = string("targets") {
                if targets.trim().is_empty() {
                    return Err(SemauriError::semantic(
                        "S339",
                        "QLoRA targets must be a non-empty string",
                        Some(span),
                        None,
                    ));
                }
            }
        }
        _ => {}
    }

    Ok(())
}


/// Validate function definitions by walking the typed expression tree.
/// Domain operations can read or write runtime state and must not occur in
/// reusable compile-time functions. Previously checked calls are pure.
fn pure_function_expression(expr: &HirExpr) -> bool {
    match expr {
        HirExpr::DomainOperation { .. } => false,
        HirExpr::Literal { .. } | HirExpr::SymbolRef { .. } => true,
        HirExpr::List { items, .. } => items.iter().all(pure_function_expression),
        HirExpr::Unary { operand, .. } | HirExpr::Promote { value: operand, .. } =>
            pure_function_expression(operand),
        HirExpr::Binary { left, right, .. } =>
            pure_function_expression(left) && pure_function_expression(right),
        HirExpr::Reduce { iterable, initial, body, .. } =>
            pure_function_expression(iterable) && pure_function_expression(initial) && pure_function_expression(body),
        HirExpr::ZipTransform { left_list, right_list, body, .. } =>
            pure_function_expression(left_list) && pure_function_expression(right_list) &&
            pure_function_expression(body),
        HirExpr::ListTransform { iterable, body, .. } =>
            pure_function_expression(iterable) && pure_function_expression(body),
        HirExpr::Repeat { count, value, .. } =>
            pure_function_expression(count) && pure_function_expression(value),
        HirExpr::Conditional { condition, consequence, alternative, .. } =>
            pure_function_expression(condition)
            && pure_function_expression(consequence)
            && pure_function_expression(alternative),
        HirExpr::FunctionCall { arguments, body, .. } =>
            arguments.iter().all(pure_function_expression) && pure_function_expression(body),
    }
}

// JSON conversion excludes runtime handles and effects by construction.
fn is_json_serializable_type(ty: &Type) -> bool {
    match ty {
        Type::Number | Type::Boolean | Type::String | Type::Color => true,
        Type::List(element) => is_json_serializable_type(element),
        _ => false,
    }
}
