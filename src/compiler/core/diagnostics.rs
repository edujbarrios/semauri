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

