#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    Not,
    Negate,
    Length,
    Reverse,
    Flatten,
    Absolute,
    Sign,
    Floor,
    Ceiling,
    Product,
    Sum,
    Average,
    Minimum,
    Maximum,
    Any,
    All,
    Median,
    Sorted,
    Distinct,
    First,
    Last,
    Uppercase,
    Lowercase,
    Trim,
    Words,
    Lines,
    Json,
    ParseNumber,
    ParseBoolean,
    DecodeJsonString,
    DecodeJsonNumber,
    DecodeJsonBoolean,
    DecodeJsonStrings,
    DecodeJsonNumbers,
    DecodeJsonBooleans,
    JsonKeys,
    Characters,
    Codepoints,
    Codepoint,
    CharacterFrom,
}

impl UnaryOperator {
    fn as_str(&self) -> &'static str {
        match self {
            UnaryOperator::Not => "not",
            UnaryOperator::Negate => "negate",
            UnaryOperator::Length => "length",
            UnaryOperator::Reverse => "reverse",
            UnaryOperator::Flatten => "flatten",
            UnaryOperator::Absolute => "absolute",
            UnaryOperator::Sign => "sign",
            UnaryOperator::Floor => "floor",
            UnaryOperator::Ceiling => "ceiling",
            UnaryOperator::Product => "product",
            UnaryOperator::Sum => "sum",
            UnaryOperator::Average => "average",
            UnaryOperator::Minimum => "minimum",
            UnaryOperator::Maximum => "maximum",
            UnaryOperator::Any => "any",
            UnaryOperator::All => "all",
            UnaryOperator::Median => "median",
            UnaryOperator::Sorted => "sorted",
            UnaryOperator::Distinct => "distinct",
            UnaryOperator::First => "first",
            UnaryOperator::Last => "last",
            UnaryOperator::Uppercase => "uppercase",
            UnaryOperator::Lowercase => "lowercase",
            UnaryOperator::Trim => "trim",
            UnaryOperator::Words => "words",
            UnaryOperator::Lines => "lines",
            UnaryOperator::Json => "json",
            UnaryOperator::ParseNumber => "parse_number",
            UnaryOperator::ParseBoolean => "parse_boolean",
            UnaryOperator::DecodeJsonString => "decode_json_string",
            UnaryOperator::DecodeJsonNumber => "decode_json_number",
            UnaryOperator::DecodeJsonBoolean => "decode_json_boolean",
            UnaryOperator::DecodeJsonStrings => "decode_json_strings",
            UnaryOperator::DecodeJsonNumbers => "decode_json_numbers",
            UnaryOperator::DecodeJsonBooleans => "decode_json_booleans",
            UnaryOperator::JsonKeys => "json_keys",
            UnaryOperator::Characters => "characters",
            UnaryOperator::Codepoints => "codepoints",
            UnaryOperator::Codepoint => "codepoint",
            UnaryOperator::CharacterFrom => "character_from",
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
    Contains,
    JsonHasKey,
    JsonPointer,
    TextOccurrences,
    TextPosition,
    TextPositions,
    TextLastPosition,
    CharacterAt,
    TextRemoveAll,
    JsonFieldString,
    JsonFieldNumber,
    JsonFieldBoolean,
    JsonFieldStrings,
    JsonFieldNumbers,
    JsonFieldBooleans,
    Split,
    Join,
    StartsWith,
    EndsWith,
    AtIndex,
    Range,
    Take,
    Drop,
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
            BinaryOperator::Contains => "contains",
            BinaryOperator::JsonHasKey => "json_has_key",
            BinaryOperator::JsonPointer => "json_pointer",
            BinaryOperator::TextOccurrences => "text_occurrences",
            BinaryOperator::TextPosition => "text_position",
            BinaryOperator::TextPositions => "text_positions",
            BinaryOperator::TextLastPosition => "text_last_position",
            BinaryOperator::CharacterAt => "character_at",
            BinaryOperator::TextRemoveAll => "text_remove_all",
            BinaryOperator::JsonFieldString => "json_field_string",
            BinaryOperator::JsonFieldNumber => "json_field_number",
            BinaryOperator::JsonFieldBoolean => "json_field_boolean",
            BinaryOperator::JsonFieldStrings => "json_field_strings",
            BinaryOperator::JsonFieldNumbers => "json_field_numbers",
            BinaryOperator::JsonFieldBooleans => "json_field_booleans",
            BinaryOperator::Split => "split",
            BinaryOperator::Join => "join",
            BinaryOperator::StartsWith => "starts_with",
            BinaryOperator::EndsWith => "ends_with",
            BinaryOperator::AtIndex => "at_index",
            BinaryOperator::Range => "range",
            BinaryOperator::Take => "take",
            BinaryOperator::Drop => "drop",
            BinaryOperator::And => "and",
            BinaryOperator::Or => "or",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListTransformKind {
    Map,
    Filter,
    Partition,
    Count,
    Any,
    All,
}

impl ListTransformKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Map => "map",
            Self::Filter => "filter",
            Self::Partition => "partition",
            Self::Count => "count",
            Self::Any => "any",
            Self::All => "all",
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
    Repeat {
        count: Box<Expr>,
        value: Box<Expr>,
        span: SourceSpan,
    },
    Conditional {
        condition: Box<Expr>,
        consequence: Box<Expr>,
        alternative: Box<Expr>,
        span: SourceSpan,
    },
    FunctionCall {
        name: String,
        arguments: Vec<Expr>,
        span: SourceSpan,
    },
    Reduce {
        iterator: String,
        iterator_span: SourceSpan,
        accumulator: String,
        accumulator_span: SourceSpan,
        iterable: Box<Expr>,
        initial: Box<Expr>,
        body: Box<Expr>,
        span: SourceSpan,
    },
    ZipTransform {
        left_name: String, left_span: SourceSpan, left_list: Box<Expr>,
        right_name: String, right_span: SourceSpan, right_list: Box<Expr>,
        body: Box<Expr>, span: SourceSpan,
    },
    ListTransform {
        kind: ListTransformKind,
        iterator: String,
        binding_span: SourceSpan,
        iterable: Box<Expr>,
        body: Box<Expr>,
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
            | Expr::FunctionCall { span, .. }
            | Expr::Conditional { span, .. }
            | Expr::Repeat { span, .. }
            | Expr::Reduce { span, .. }
            | Expr::ZipTransform { span, .. }
            | Expr::ListTransform { span, .. }
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
            Expr::FunctionCall { name, arguments, .. } => Expr::FunctionCall { name, arguments, span },
            Expr::Repeat { count, value, .. } => Expr::Repeat { count, value, span },
            Expr::Conditional { condition, consequence, alternative, .. } => Expr::Conditional {
                condition, consequence, alternative, span,
            },
            Expr::Reduce { iterator, iterator_span, accumulator, accumulator_span, iterable, initial, body, .. } =>
                Expr::Reduce { iterator, iterator_span, accumulator, accumulator_span, iterable, initial, body, span },
            Expr::ZipTransform { left_name, left_span, left_list, right_name, right_span, right_list, body, .. } =>
                Expr::ZipTransform { left_name, left_span, left_list, right_name, right_span, right_list, body, span },
            Expr::ListTransform { kind, iterator, binding_span, iterable, body, .. } => Expr::ListTransform {
                kind, iterator, binding_span, iterable, body, span,
            },
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
            Expr::Repeat { count, value, .. } => {
                base.insert("type".into(), json!("repeat_expression"));
                base.insert("count".into(), count.to_json());
                base.insert("value".into(), value.to_json());
            }
            Expr::Conditional { condition, consequence, alternative, .. } => {
                base.insert("type".into(), json!("conditional_expression"));
                base.insert("condition".into(), condition.to_json());
                base.insert("consequence".into(), consequence.to_json());
                base.insert("alternative".into(), alternative.to_json());
            }
            Expr::FunctionCall { name, arguments, .. } => {
                base.insert("type".into(), json!("function_call"));
                base.insert("name".into(), json!(name));
                base.insert("arguments".into(), JsonValue::Array(arguments.iter().map(Expr::to_json).collect()));
            }
            Expr::Variable { name, .. } => {
                base.insert("type".into(), json!("variable_reference"));
                base.insert("name".into(), json!(name));
            }
            Expr::Reduce { iterator, accumulator, iterable, initial, body, .. } => {
                base.insert("type".into(), json!("reduce_expression"));
                base.insert("iterator".into(), json!(iterator));
                base.insert("accumulator".into(), json!(accumulator));
                base.insert("iterable".into(), iterable.to_json());
                base.insert("initial".into(), initial.to_json());
                base.insert("body".into(), body.to_json());
            }
            Expr::ZipTransform { left_name, left_list, right_name, right_list, body, .. } => {
                base.insert("type".into(), json!("zip_transform"));
                base.insert("left_iterator".into(), json!(left_name));
                base.insert("left_list".into(), left_list.to_json());
                base.insert("right_iterator".into(), json!(right_name));
                base.insert("right_list".into(), right_list.to_json());
                base.insert("body".into(), body.to_json());
            }
            Expr::ListTransform { kind, iterator, iterable, body, .. } => {
                base.insert("type".into(), json!("list_transform"));
                base.insert("operator".into(), json!(kind.as_str()));
                base.insert("iterator".into(), json!(iterator));
                base.insert("iterable".into(), iterable.to_json());
                base.insert("body".into(), body.to_json());
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
    DefineFunction {
        name: String,
        parameters: Vec<(String, Type)>,
        body: Expr,
        span: SourceSpan,
    },
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
            Stmt::DefineFunction { span, .. }
            | Stmt::CreateArtifact { span, .. }
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
            Stmt::DefineFunction { name, parameters, body, .. } => {
                object.insert("type".into(), json!("function_definition"));
                object.insert("name".into(), json!(name));
                object.insert("parameters".into(), JsonValue::Array(parameters.iter()
                    .map(|(name, ty)| json!({"name": name, "type": ty.to_json()})).collect()));
                object.insert("body".into(), body.to_json());
            }
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

