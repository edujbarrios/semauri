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

