pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SourceSpan {
    pub start: Position,
    pub end: Position,
}

impl SourceSpan {
    pub fn new(start_line: usize, start_column: usize, end_line: usize, end_column: usize) -> Self {
        Self {
            start: Position {
                line: start_line,
                column: start_column,
            },
            end: Position {
                line: end_line,
                column: end_column,
            },
        }
    }

    pub fn point(line: usize, column: usize) -> Self {
        Self::new(line, column, line, column + 1)
    }

    pub fn line(&self) -> usize {
        self.start.line
    }

    pub fn column(&self) -> usize {
        self.start.column
    }

    pub fn end_line(&self) -> usize {
        self.end.line
    }

    pub fn end_column(&self) -> usize {
        self.end.column
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Lex,
    Parse,
    Semantic,
    Backend,
    Usage,
    Runtime,
}

#[derive(Clone, Debug)]
pub struct SemauriError {
    pub kind: ErrorKind,
    pub code: String,
    pub message: String,
    pub span: Option<SourceSpan>,
    pub hint: Option<String>,
}

impl SemauriError {
    fn new(
        kind: ErrorKind,
        code: impl Into<String>,
        message: impl Into<String>,
        span: Option<SourceSpan>,
        hint: Option<String>,
    ) -> Self {
        Self {
            kind,
            code: code.into(),
            message: message.into(),
            span,
            hint,
        }
    }

    fn lex(code: &str, message: impl Into<String>, span: SourceSpan) -> Self {
        Self::new(ErrorKind::Lex, code, message, Some(span), None)
    }

    fn parse(
        code: &str,
        message: impl Into<String>,
        span: SourceSpan,
        hint: Option<String>,
    ) -> Self {
        Self::new(ErrorKind::Parse, code, message, Some(span), hint)
    }

    fn semantic(
        code: &str,
        message: impl Into<String>,
        span: Option<SourceSpan>,
        hint: Option<String>,
    ) -> Self {
        Self::new(ErrorKind::Semantic, code, message, span, hint)
    }

    fn backend(code: &str, message: impl Into<String>, hint: Option<String>) -> Self {
        Self::new(ErrorKind::Backend, code, message, None, hint)
    }

    pub fn diagnostic(&self, source: Option<&str>) -> String {
        let location = self
            .span
            .map(|span| format!(" at {}:{}", span.line(), span.column()))
            .unwrap_or_default();
        let header = format!("{}{}: {}", self.code, location, self.message);

        let mut text = if let (Some(source), Some(span)) = (source, self.span) {
            if let Some(source_line) = source.lines().nth(span.line().saturating_sub(1)) {
                let gutter_width = span.line().to_string().len();
                let end_column = if span.end_line() == span.line() {
                    span.end_column()
                } else {
                    source_line.chars().count() + 1
                };
                let width = end_column.saturating_sub(span.column()).max(1);
                let marker = format!(
                    "{}^{}",
                    " ".repeat(span.column().saturating_sub(1)),
                    "~".repeat(width.saturating_sub(1))
                );
                format!(
                    "{}\n{} |\n{} | {}\n{} | {}",
                    header,
                    " ".repeat(gutter_width),
                    span.line().to_string().rjust(gutter_width),
                    source_line,
                    " ".repeat(gutter_width),
                    marker
                )
            } else {
                header
            }
        } else {
            header
        };

        if let Some(hint) = &self.hint {
            text.push_str("\nHint: ");
            text.push_str(hint);
        }
        text
    }
}

trait Rjust {
    fn rjust(self, width: usize) -> String;
}

impl Rjust for String {
    fn rjust(self, width: usize) -> String {
        if self.len() >= width {
            self
        } else {
            format!("{}{}", " ".repeat(width - self.len()), self)
        }
    }
}

impl fmt::Display for SemauriError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.diagnostic(None))
    }
}

impl std::error::Error for SemauriError {}

pub type Result<T> = std::result::Result<T, SemauriError>;

#[derive(Clone, Debug, PartialEq)]
pub enum NumberValue {
    Int(i64),
    Float(f64),
}

impl NumberValue {
    fn as_f64(&self) -> f64 {
        match self {
            NumberValue::Int(value) => *value as f64,
            NumberValue::Float(value) => *value,
        }
    }

    fn is_zero(&self) -> bool {
        self.as_f64() == 0.0
    }

    fn is_positive(&self) -> bool {
        self.as_f64() > 0.0
    }

    fn is_negative(&self) -> bool {
        self.as_f64() < 0.0
    }

    fn is_integer(&self) -> bool {
        match self {
            NumberValue::Int(_) => true,
            NumberValue::Float(value) => value.fract() == 0.0,
        }
    }

    fn to_json(&self) -> JsonValue {
        match self {
            NumberValue::Int(value) => json!(value),
            NumberValue::Float(value) => json!(value),
        }
    }

    fn from_f64(value: f64) -> Self {
        if value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64 {
            NumberValue::Int(value as i64)
        } else {
            NumberValue::Float(value)
        }
    }
}

impl fmt::Display for NumberValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumberValue::Int(value) => write!(f, "{value}"),
            NumberValue::Float(value) => {
                let mut rendered = value.to_string();
                if !rendered.contains('.') && !rendered.contains('e') && !rendered.contains('E') {
                    rendered.push_str(".0");
                }
                write!(f, "{rendered}")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NominalType {
    pub domain: String,
    pub name: String,
    pub base_type: Box<Type>,
    pub promote_from_base: bool,
}

impl NominalType {
    pub fn new(domain: &str, name: &str, base_type: Type, promote_from_base: bool) -> Self {
        Self {
            domain: domain.to_string(),
            name: name.to_string(),
            base_type: Box::new(base_type),
            promote_from_base,
        }
    }

    fn to_json(&self) -> JsonValue {
        json!({
            "kind": "nominal",
            "domain": self.domain,
            "name": self.name,
            "base": self.base_type.to_json(),
            "promote_from_base": self.promote_from_base
        })
    }
}

impl fmt::Display for NominalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.domain, self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Unit,
    Number,
    Boolean,
    String,
    Color,
    EntityRef,
    Opaque,
    List(Box<Type>),
    Nominal(NominalType),
}

impl Type {
    fn to_json(&self) -> JsonValue {
        match self {
            Type::Unit => json!("unit"),
            Type::Number => json!("number"),
            Type::Boolean => json!("boolean"),
            Type::String => json!("string"),
            Type::Color => json!("color"),
            Type::EntityRef => json!("entity_ref"),
            Type::Opaque => json!("opaque"),
            Type::List(element) => json!({
                "kind": "list",
                "element_type": element.to_json()
            }),
            Type::Nominal(nominal) => nominal.to_json(),
        }
    }

    fn assignment_kind(actual: &Type, expected: &Type) -> Option<AssignmentKind> {
        if actual == expected {
            return Some(AssignmentKind::Exact);
        }
        if let Type::Nominal(nominal) = expected {
            if nominal.promote_from_base && actual == nominal.base_type.as_ref() {
                return Some(AssignmentKind::Promote);
            }
        }
        None
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Unit => write!(f, "unit"),
            Type::Number => write!(f, "number"),
            Type::Boolean => write!(f, "boolean"),
            Type::String => write!(f, "string"),
            Type::Color => write!(f, "color"),
            Type::EntityRef => write!(f, "entity_ref"),
            Type::Opaque => write!(f, "opaque"),
            Type::List(element) => write!(f, "list<{element}>"),
            Type::Nominal(nominal) => write!(f, "{nominal}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AssignmentKind {
    Exact,
    Promote,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LiteralValue {
    Number(NumberValue),
    Boolean(bool),
    String(String),
    Color(String),
}

impl LiteralValue {
    fn to_json(&self) -> JsonValue {
        match self {
            LiteralValue::Number(value) => value.to_json(),
            LiteralValue::Boolean(value) => json!(value),
            LiteralValue::String(value) | LiteralValue::Color(value) => json!(value),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueData {
    Number(NumberValue),
    Boolean(bool),
    String(String),
    Color(String),
    List(Vec<ValueData>),
    Unit,
}

impl ValueData {
    fn to_json(&self) -> JsonValue {
        match self {
            ValueData::Number(value) => value.to_json(),
            ValueData::Boolean(value) => json!(value),
            ValueData::String(value) | ValueData::Color(value) => json!(value),
            ValueData::List(values) => JsonValue::Array(values.iter().map(ValueData::to_json).collect()),
            ValueData::Unit => JsonValue::Null,
        }
    }

    fn describe(&self, ty: &Type) -> String {
        match self {
            ValueData::Number(value) => format!("{ty} {value}"),
            ValueData::Boolean(value) => format!("{ty} {value}"),
            ValueData::String(value) | ValueData::Color(value) => {
                format!("{ty} {:?}", value)
            }
            ValueData::List(values) => format!("{ty} {:?}", values.iter().map(ValueData::to_json).collect::<Vec<_>>()),
            ValueData::Unit => "unit nil".to_string(),
        }
    }
}

