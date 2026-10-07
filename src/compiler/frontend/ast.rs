#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    Not,
    Negate,
}

impl UnaryOperator {
    fn as_str(&self) -> &'static str {
        match self {
            UnaryOperator::Not => "not",
            UnaryOperator::Negate => "negate",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    NotEqual,
    And,
    Or,
}

impl BinaryOperator {
    fn as_str(&self) -> &'static str {
        match self {
            BinaryOperator::Add => "add",
            BinaryOperator::Subtract => "subtract",
            BinaryOperator::Multiply => "multiply",
            BinaryOperator::Divide => "divide",
            BinaryOperator::Modulo => "modulo",
            BinaryOperator::GreaterThan => "greater_than",
            BinaryOperator::LessThan => "less_than",
            BinaryOperator::GreaterThanOrEqual => "greater_than_or_equal",
            BinaryOperator::LessThanOrEqual => "less_than_or_equal",
            BinaryOperator::Equal => "equal",
            BinaryOperator::NotEqual => "not_equal",
            BinaryOperator::And => "and",
            BinaryOperator::Or => "or",
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    Literal {
        ty: Type,
        value: LiteralValue,
        span: SourceSpan,
    },
    List {
        items: Vec<Expr>,
        span: SourceSpan,
    },
    Variable {
        name: String,
        span: SourceSpan,
    },
    Unary {
        operator: UnaryOperator,
        operand: Box<Expr>,
        span: SourceSpan,
    },
    Binary {
        left: Box<Expr>,
        operator: BinaryOperator,
        right: Box<Expr>,
        span: SourceSpan,
    },
    DomainOperation {
        domain: String,
        operation: String,
        arguments: BTreeMap<String, Expr>,
        span: SourceSpan,
    },
}

impl Expr {
    fn span(&self) -> SourceSpan {
        match self {
            Expr::Literal { span, .. }
            | Expr::List { span, .. }
            | Expr::Variable { span, .. }
            | Expr::Unary { span, .. }
            | Expr::Binary { span, .. }
            | Expr::DomainOperation { span, .. } => *span,
        }
    }

    fn with_span(self, span: SourceSpan) -> Self {
        match self {
            Expr::Literal { ty, value, .. } => Expr::Literal { ty, value, span },
            Expr::List { items, .. } => Expr::List { items, span },
            Expr::Variable { name, .. } => Expr::Variable { name, span },
            Expr::Unary {
                operator, operand, ..
            } => Expr::Unary {
                operator,
                operand,
                span,
            },
            Expr::Binary {
                left,
                operator,
                right,
                ..
            } => Expr::Binary {
                left,
                operator,
                right,
                span,
            },
            Expr::DomainOperation {
                domain,
                operation,
                arguments,
                ..
            } => Expr::DomainOperation {
                domain,
                operation,
                arguments,
                span,
            },
        }
    }

    fn to_json(&self) -> JsonValue {
        let span = self.span();
        let mut base = JsonMap::new();
        base.insert("line".into(), json!(span.line()));
        base.insert("column".into(), json!(span.column()));
        base.insert("end_line".into(), json!(span.end_line()));
        base.insert("end_column".into(), json!(span.end_column()));
        base.insert("span".into(), json!(span));

        match self {
            Expr::Literal { ty, value, .. } => {
                base.insert("type".into(), json!("literal"));
                base.insert("value_type".into(), ty.to_json());
                base.insert("value".into(), value.to_json());
            }
            Expr::List { items, .. } => {
                base.insert("type".into(), json!("list_literal"));
                base.insert(
                    "items".into(),
                    JsonValue::Array(items.iter().map(Expr::to_json).collect()),
                );
            }
            Expr::Variable { name, .. } => {
                base.insert("type".into(), json!("variable_reference"));
                base.insert("name".into(), json!(name));
            }
            Expr::Unary {
                operator, operand, ..
            } => {
                base.insert("type".into(), json!("unary_expression"));
                base.insert("operator".into(), json!(operator.as_str()));
                base.insert("operand".into(), operand.to_json());
            }
            Expr::Binary {
                left,
                operator,
                right,
                ..
            } => {
                base.insert("type".into(), json!("binary_expression"));
                base.insert("operator".into(), json!(operator.as_str()));
                base.insert("left".into(), left.to_json());
                base.insert("right".into(), right.to_json());
            }
            Expr::DomainOperation {
                domain,
                operation,
                arguments,
                ..
            } => {
                base.insert("type".into(), json!("domain_operation"));
                base.insert("domain".into(), json!(domain));
                base.insert("operation".into(), json!(operation));
                base.insert(
                    "arguments".into(),
                    JsonValue::Object(
                        arguments
                            .iter()
                            .map(|(name, value)| (name.clone(), value.to_json()))
                            .collect(),
                    ),
                );
            }
        }
        JsonValue::Object(base)
    }
}

#[derive(Clone, Debug)]
pub enum Reference {
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

impl Reference {
    fn span(&self) -> SourceSpan {
        match self {
            Reference::Pronoun { span, .. } | Reference::Named { span, .. } => *span,
        }
    }

    fn to_json(&self) -> JsonValue {
        let span = self.span();
        match self {
            Reference::Pronoun { pronoun, .. } => json!({
                "type": "pronoun_reference",
                "pronoun": pronoun,
                "line": span.line(),
                "column": span.column(),
                "end_line": span.end_line(),
                "end_column": span.end_column(),
                "span": span
            }),
            Reference::Named {
                domain,
                kind,
                label,
                ..
            } => json!({
                "type": "named_reference",
                "domain": domain,
                "kind": kind,
                "label": label,
                "line": span.line(),
                "column": span.column(),
                "end_line": span.end_line(),
                "end_column": span.end_column(),
                "span": span
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Block {
    pub statements: Vec<Stmt>,
    pub span: SourceSpan,
}

impl Block {
    fn to_json(&self) -> JsonValue {
        json!({
            "type": "block",
            "statements": self.statements.iter().map(Stmt::to_json).collect::<Vec<_>>(),
            "line": self.span.line(),
            "column": self.span.column(),
            "end_line": self.span.end_line(),
            "end_column": self.span.end_column(),
            "span": self.span
        })
    }
}

#[derive(Clone, Debug)]
pub enum Stmt {
    CreateArtifact {
        domain: String,
        kind: String,
        subject: Option<String>,
        title: Option<String>,
        span: SourceSpan,
    },
    DomainScope {
        domain: String,
        body: Block,
        span: SourceSpan,
    },
    DomainOperation {
        domain: String,
        operation: String,
        arguments: BTreeMap<String, Expr>,
        span: SourceSpan,
    },
    SetTitle {
        title: String,
        span: SourceSpan,
    },
    AddElement {
        domain: String,
        kind: String,
        label: Option<String>,
        span: SourceSpan,
    },
    SetProperty {
        domain: String,
        target: Reference,
        property: String,
        value: Expr,
        span: SourceSpan,
    },
    Let {
        name: String,
        value: Expr,
        span: SourceSpan,
    },
    If {
        condition: Expr,
        consequence: Block,
        alternative: Option<Block>,
        span: SourceSpan,
    },
    ForEach {
        variable_name: String,
        binding_span: SourceSpan,
        iterable: Expr,
        body: Block,
        span: SourceSpan,
    },
}

impl Stmt {
    fn span(&self) -> SourceSpan {
        match self {
            Stmt::CreateArtifact { span, .. }
            | Stmt::DomainScope { span, .. }
            | Stmt::DomainOperation { span, .. }
            | Stmt::SetTitle { span, .. }
            | Stmt::AddElement { span, .. }
            | Stmt::SetProperty { span, .. }
            | Stmt::Let { span, .. }
            | Stmt::If { span, .. }
            | Stmt::ForEach { span, .. } => *span,
        }
    }

    fn to_json(&self) -> JsonValue {
        let span = self.span();
        let mut object = JsonMap::new();
        object.insert("line".into(), json!(span.line()));
        object.insert("column".into(), json!(span.column()));
        object.insert("end_line".into(), json!(span.end_line()));
        object.insert("end_column".into(), json!(span.end_column()));
        object.insert("span".into(), json!(span));

        match self {
            Stmt::CreateArtifact {
                domain,
                kind,
                subject,
                title,
                ..
            } => {
                object.insert("type".into(), json!("create_artifact"));
                object.insert("domain".into(), json!(domain));
                object.insert("kind".into(), json!(kind));
                object.insert("subject".into(), json!(subject));
                object.insert("title".into(), json!(title));
            }
            Stmt::DomainScope { domain, body, .. } => {
                object.insert("type".into(), json!("domain_scope"));
                object.insert("domain".into(), json!(domain));
                object.insert("body".into(), body.to_json());
            }
            Stmt::DomainOperation {
                domain,
                operation,
                arguments,
                ..
            } => {
                object.insert("type".into(), json!("domain_operation"));
                object.insert("domain".into(), json!(domain));
                object.insert("operation".into(), json!(operation));
                object.insert(
                    "arguments".into(),
                    JsonValue::Object(
                        arguments
                            .iter()
                            .map(|(name, value)| (name.clone(), value.to_json()))
                            .collect(),
                    ),
                );
            }
            Stmt::SetTitle { title, .. } => {
                object.insert("type".into(), json!("set_title"));
                object.insert("title".into(), json!(title));
            }
            Stmt::AddElement {
                domain,
                kind,
                label,
                ..
            } => {
                object.insert("type".into(), json!("add_element"));
                object.insert("domain".into(), json!(domain));
                object.insert("kind".into(), json!(kind));
                object.insert("label".into(), json!(label));
            }
            Stmt::SetProperty {
                domain,
                target,
                property,
                value,
                ..
            } => {
                object.insert("type".into(), json!("set_property"));
                object.insert("domain".into(), json!(domain));
                object.insert("target".into(), target.to_json());
                object.insert("property".into(), json!(property));
                object.insert("value".into(), value.to_json());
            }
            Stmt::Let { name, value, .. } => {
                object.insert("type".into(), json!("let_binding"));
                object.insert("name".into(), json!(name));
                object.insert("value".into(), value.to_json());
            }
            Stmt::If {
                condition,
                consequence,
                alternative,
                ..
            } => {
                object.insert("type".into(), json!("if_statement"));
                object.insert("condition".into(), condition.to_json());
                object.insert("consequence".into(), consequence.to_json());
                object.insert(
                    "alternative".into(),
                    alternative
                        .as_ref()
                        .map(Block::to_json)
                        .unwrap_or(JsonValue::Null),
                );
            }
            Stmt::ForEach {
                variable_name,
                binding_span,
                iterable,
                body,
                ..
            } => {
                object.insert("type".into(), json!("for_each"));
                object.insert("variable_name".into(), json!(variable_name));
                object.insert("binding_span".into(), json!(binding_span));
                object.insert("iterable".into(), iterable.to_json());
                object.insert("body".into(), body.to_json());
            }
        }
        JsonValue::Object(object)
    }
}

#[derive(Clone, Debug)]
pub struct Program {
    pub statements: Vec<Stmt>,
    pub span: SourceSpan,
}

impl Program {
    pub fn to_json(&self) -> JsonValue {
        json!({
            "type": "program",
            "statements": self.statements.iter().map(Stmt::to_json).collect::<Vec<_>>(),
            "span": self.span
        })
    }
}

