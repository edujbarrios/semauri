struct HirBuilder<'a> {
    domains: &'a DomainRegistry,
    symbols: Vec<Symbol>,
    scopes: Vec<HashMap<String, u64>>,
}

impl<'a> HirBuilder<'a> {
    fn new(domains: &'a DomainRegistry) -> Self {
        Self {
            domains,
            symbols: Vec::new(),
            scopes: vec![HashMap::new()],
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
        statements.iter().map(|statement| self.build_stmt(statement)).collect()
    }

    fn build_stmt(&mut self, statement: &Stmt) -> Result<HirStmt> {
        match statement {
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
        BinaryOperator::Add
        | BinaryOperator::Subtract
        | BinaryOperator::Multiply
        | BinaryOperator::Divide => {
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

