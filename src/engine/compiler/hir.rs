#[derive(Clone, Debug)]
pub struct Symbol {
    pub id: u64,
    pub name: String,
    pub kind: String,
    pub ty: Type,
    pub definition_span: SourceSpan,
}

impl Symbol {
    pub fn to_json(&self) -> JsonValue {
        json!({
            "id": self.id,
            "name": self.name,
            "kind": self.kind,
            "type": self.ty.to_json(),
            "definition_span": self.definition_span
        })
    }
}

#[derive(Clone, Debug)]
pub enum HirExpr {
    Literal {
        ty: Type,
        value: LiteralValue,
        span: SourceSpan,
    },
    List {
        ty: Type,
        items: Vec<HirExpr>,
        span: SourceSpan,
    },
    SymbolRef {
        ty: Type,
        symbol_id: u64,
        name: String,
        source_name: String,
        span: SourceSpan,
    },
    Promote {
        ty: Type,
        from_type: Type,
        value: Box<HirExpr>,
        span: SourceSpan,
    },
    Unary {
        ty: Type,
        operator: UnaryOperator,
        operand: Box<HirExpr>,
        span: SourceSpan,
    },
    Binary {
        ty: Type,
        left: Box<HirExpr>,
        operator: BinaryOperator,
        right: Box<HirExpr>,
        span: SourceSpan,
    },
    DomainOperation {
        ty: Type,
        domain: String,
        operation: String,
        arguments: BTreeMap<String, HirExpr>,
        effects: Vec<String>,
        span: SourceSpan,
    },
}

impl HirExpr {
    fn ty(&self) -> Type {
        match self {
            HirExpr::Literal { ty, .. }
            | HirExpr::List { ty, .. }
            | HirExpr::SymbolRef { ty, .. }
            | HirExpr::Promote { ty, .. }
            | HirExpr::Unary { ty, .. }
            | HirExpr::Binary { ty, .. }
            | HirExpr::DomainOperation { ty, .. } => ty.clone(),
        }
    }

    fn span(&self) -> SourceSpan {
        match self {
            HirExpr::Literal { span, .. }
            | HirExpr::List { span, .. }
            | HirExpr::SymbolRef { span, .. }
            | HirExpr::Promote { span, .. }
            | HirExpr::Unary { span, .. }
            | HirExpr::Binary { span, .. }
            | HirExpr::DomainOperation { span, .. } => *span,
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            HirExpr::Literal { .. } => "literal",
            HirExpr::List { .. } => "list",
            HirExpr::SymbolRef { .. } => "symbol_ref",
            HirExpr::Promote { .. } => "promote",
            HirExpr::Unary { .. } => "unary",
            HirExpr::Binary { .. } => "binary",
            HirExpr::DomainOperation { .. } => "domain_operation",
        }
    }

    fn to_json(&self) -> JsonValue {
        let mut fields = JsonMap::new();
        match self {
            HirExpr::Literal { value, .. } => {
                fields.insert("value".into(), value.to_json());
            }
            HirExpr::List { items, .. } => {
                fields.insert(
                    "items".into(),
                    JsonValue::Array(items.iter().map(HirExpr::to_json).collect()),
                );
            }
            HirExpr::SymbolRef {
                symbol_id,
                name,
                source_name,
                ..
            } => {
                fields.insert("symbol_id".into(), json!(symbol_id));
                fields.insert("name".into(), json!(name));
                fields.insert("source_name".into(), json!(source_name));
            }
            HirExpr::Promote {
                from_type, value, ..
            } => {
                fields.insert("value".into(), value.to_json());
                fields.insert("from_type".into(), from_type.to_json());
            }
            HirExpr::Unary {
                operator, operand, ..
            } => {
                fields.insert("operator".into(), json!(operator.as_str()));
                fields.insert("operand".into(), operand.to_json());
            }
            HirExpr::Binary {
                left,
                operator,
                right,
                ..
            } => {
                fields.insert("operator".into(), json!(operator.as_str()));
                fields.insert("left".into(), left.to_json());
                fields.insert("right".into(), right.to_json());
            }
            HirExpr::DomainOperation {
                domain,
                operation,
                arguments,
                effects,
                ..
            } => {
                fields.insert("domain".into(), json!(domain));
                fields.insert("operation".into(), json!(operation));
                fields.insert(
                    "arguments".into(),
                    JsonValue::Object(
                        arguments
                            .iter()
                            .map(|(name, value)| (name.clone(), value.to_json()))
                            .collect(),
                    ),
                );
                fields.insert("effects".into(), json!(effects));
            }
        }
        json!({
            "kind": self.kind(),
            "type": self.ty().to_json(),
            "fields": fields,
            "span": self.span()
        })
    }
}

#[derive(Clone, Debug)]
pub enum HirReference {
    Pronoun {
        pronoun: String,
        span: SourceSpan,
    },
    Named {
        domain: String,
        kind: String,
        label: String,
        span: SourceSpan,
    },
}

impl HirReference {
    fn span(&self) -> SourceSpan {
        match self {
            HirReference::Pronoun { span, .. } | HirReference::Named { span, .. } => *span,
        }
    }

    fn to_json(&self) -> JsonValue {
        match self {
            HirReference::Pronoun { pronoun, span } => json!({
                "kind": "pronoun_reference",
                "type": "entity_ref",
                "fields": {"pronoun": pronoun},
                "span": span
            }),
            HirReference::Named {
                domain,
                kind,
                label,
                span,
            } => json!({
                "kind": "named_reference",
                "type": "entity_ref",
                "fields": {"domain": domain, "entity_kind": kind, "label": label},
                "span": span
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub enum HirStmt {
    DomainScope {
        domain: String,
        body: Vec<HirStmt>,
        span: SourceSpan,
    },
    Let {
        symbol_id: u64,
        name: String,
        source_name: String,
        value: HirExpr,
        span: SourceSpan,
    },
    If {
        condition: HirExpr,
        consequence: Vec<HirStmt>,
        alternative: Option<Vec<HirStmt>>,
        consequence_span: SourceSpan,
        alternative_span: Option<SourceSpan>,
        span: SourceSpan,
    },
    ForEach {
        iterator_symbol_id: u64,
        iterator_name: String,
        iterator_source_name: String,
        iterable: HirExpr,
        body: Vec<HirStmt>,
        body_span: SourceSpan,
        span: SourceSpan,
    },
    CreateArtifact {
        domain: String,
        artifact_kind: String,
        subject: Option<String>,
        title: Option<String>,
        span: SourceSpan,
    },
    DomainOperation {
        domain: String,
        operation: String,
        arguments: BTreeMap<String, HirExpr>,
        effects: Vec<String>,
        return_type: Type,
        span: SourceSpan,
    },
    SetTitle {
        title: String,
        span: SourceSpan,
    },
    AddElement {
        domain: String,
        element_kind: String,
        label: Option<String>,
        span: SourceSpan,
    },
    SetProperty {
        domain: String,
        target: HirReference,
        property: String,
        value: HirExpr,
        span: SourceSpan,
    },
}

impl HirStmt {
    fn span(&self) -> SourceSpan {
        match self {
            HirStmt::DomainScope { span, .. }
            | HirStmt::Let { span, .. }
            | HirStmt::If { span, .. }
            | HirStmt::ForEach { span, .. }
            | HirStmt::CreateArtifact { span, .. }
            | HirStmt::DomainOperation { span, .. }
            | HirStmt::SetTitle { span, .. }
            | HirStmt::AddElement { span, .. }
            | HirStmt::SetProperty { span, .. } => *span,
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            HirStmt::DomainScope { .. } => "domain_scope",
            HirStmt::Let { .. } => "let",
            HirStmt::If { .. } => "if",
            HirStmt::ForEach { .. } => "for_each",
            HirStmt::CreateArtifact { .. } => "create_artifact",
            HirStmt::DomainOperation { .. } => "domain_operation",
            HirStmt::SetTitle { .. } => "set_title",
            HirStmt::AddElement { .. } => "add_element",
            HirStmt::SetProperty { .. } => "set_property",
        }
    }

    fn to_json(&self) -> JsonValue {
        let mut fields = JsonMap::new();
        match self {
            HirStmt::DomainScope { domain, body, span } => {
                fields.insert("domain".into(), json!(domain));
                fields.insert("body".into(), hir_block_json(body, *span));
            }
            HirStmt::Let {
                symbol_id,
                name,
                source_name,
                value,
                ..
            } => {
                fields.insert("symbol_id".into(), json!(symbol_id));
                fields.insert("name".into(), json!(name));
                fields.insert("source_name".into(), json!(source_name));
                fields.insert("value".into(), value.to_json());
            }
            HirStmt::If {
                condition,
                consequence,
                alternative,
                consequence_span,
                alternative_span,
                ..
            } => {
                fields.insert("condition".into(), condition.to_json());
                fields.insert(
                    "consequence".into(),
                    hir_block_json(consequence, *consequence_span),
                );
                fields.insert(
                    "alternative".into(),
                    alternative
                        .as_ref()
                        .map(|body| hir_block_json(body, alternative_span.unwrap()))
                        .unwrap_or(JsonValue::Null),
                );
            }
            HirStmt::ForEach {
                iterator_symbol_id,
                iterator_name,
                iterator_source_name,
                iterable,
                body,
                body_span,
                ..
            } => {
                fields.insert("iterator_symbol_id".into(), json!(iterator_symbol_id));
                fields.insert("iterator_name".into(), json!(iterator_name));
                fields.insert(
                    "iterator_source_name".into(),
                    json!(iterator_source_name),
                );
                fields.insert("iterable".into(), iterable.to_json());
                fields.insert("body".into(), hir_block_json(body, *body_span));
            }
            HirStmt::CreateArtifact {
                domain,
                artifact_kind,
                subject,
                title,
                ..
            } => {
                fields.insert("domain".into(), json!(domain));
                fields.insert("artifact_kind".into(), json!(artifact_kind));
                fields.insert("subject".into(), json!(subject));
                fields.insert("title".into(), json!(title));
            }
            HirStmt::DomainOperation {
                domain,
                operation,
                arguments,
                effects,
                ..
            } => {
                fields.insert("domain".into(), json!(domain));
                fields.insert("operation".into(), json!(operation));
                fields.insert(
                    "arguments".into(),
                    JsonValue::Object(
                        arguments
                            .iter()
                            .map(|(name, value)| (name.clone(), value.to_json()))
                            .collect(),
                    ),
                );
                fields.insert("effects".into(), json!(effects));
            }
            HirStmt::SetTitle { title, .. } => {
                fields.insert("title".into(), json!(title));
            }
            HirStmt::AddElement {
                domain,
                element_kind,
                label,
                ..
            } => {
                fields.insert("domain".into(), json!(domain));
                fields.insert("element_kind".into(), json!(element_kind));
                fields.insert("label".into(), json!(label));
            }
            HirStmt::SetProperty {
                domain,
                target,
                property,
                value,
                ..
            } => {
                fields.insert("domain".into(), json!(domain));
                fields.insert("target".into(), target.to_json());
                fields.insert("property".into(), json!(property));
                fields.insert("value".into(), value.to_json());
            }
        }
        json!({
            "kind": self.kind(),
            "type": "unit",
            "fields": fields,
            "span": self.span()
        })
    }
}

fn hir_block_json(statements: &[HirStmt], span: SourceSpan) -> JsonValue {
    json!({
        "kind": "block",
        "type": "unit",
        "fields": {
            "statements": statements.iter().map(HirStmt::to_json).collect::<Vec<_>>()
        },
        "span": span
    })
}

#[derive(Clone, Debug)]
pub struct HirResult {
    pub statements: Vec<HirStmt>,
    pub symbols: Vec<Symbol>,
    pub span: SourceSpan,
}

impl HirResult {
    pub fn to_json(&self) -> JsonValue {
        json!({
            "program": {
                "kind": "program",
                "type": "unit",
                "fields": {
                    "statements": self.statements.iter().map(HirStmt::to_json).collect::<Vec<_>>()
                },
                "span": self.span
            },
            "symbols": self.symbols.iter().map(Symbol::to_json).collect::<Vec<_>>()
        })
    }
}

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
                Ok(HirExpr::Unary {
                    ty: Type::Boolean,
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
        BinaryOperator::Equal => {
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

