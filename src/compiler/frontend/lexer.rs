#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Create,
    Make,
    Let,
    Be,
    Set,
    If,
    Otherwise,
    End,
    For,
    Every,
    In,
    Within,
    List,
    Is,
    Greater,
    Less,
    Than,
    Equal,
    And,
    Or,
    Not,
    Plus,
    Minus,
    Times,
    Divided,
    Modulo,
    By,
    Boolean,
    Of,
    To,
    Article,
    Called,
    Add,
    Title,
    Pronoun,
    Color,
    DomainArtifact,
    DomainElement,
    DomainProperty,
    DomainAction,
    Word,
    Number,
    String,
    Dot,
    Comma,
    Colon,
    LParen,
    RParen,
    Eof,
}

impl TokenKind {
    fn as_str(&self) -> &'static str {
        match self {
            TokenKind::Create => "CREATE",
            TokenKind::Make => "MAKE",
            TokenKind::Let => "LET",
            TokenKind::Be => "BE",
            TokenKind::Set => "SET",
            TokenKind::If => "IF",
            TokenKind::Otherwise => "OTHERWISE",
            TokenKind::End => "END",
            TokenKind::For => "FOR",
            TokenKind::Every => "EVERY",
            TokenKind::In => "IN",
            TokenKind::Within => "WITHIN",
            TokenKind::List => "LIST",
            TokenKind::Is => "IS",
            TokenKind::Greater => "GREATER",
            TokenKind::Less => "LESS",
            TokenKind::Than => "THAN",
            TokenKind::Equal => "EQUAL",
            TokenKind::And => "AND",
            TokenKind::Or => "OR",
            TokenKind::Not => "NOT",
            TokenKind::Plus => "PLUS",
            TokenKind::Minus => "MINUS",
            TokenKind::Times => "TIMES",
            TokenKind::Divided => "DIVIDED",
            TokenKind::Modulo => "MODULO",
            TokenKind::By => "BY",
            TokenKind::Boolean => "BOOLEAN",
            TokenKind::Of => "OF",
            TokenKind::To => "TO",
            TokenKind::Article => "ARTICLE",
            TokenKind::Called => "CALLED",
            TokenKind::Add => "ADD",
            TokenKind::Title => "TITLE",
            TokenKind::Pronoun => "PRONOUN",
            TokenKind::Color => "COLOR",
            TokenKind::DomainArtifact => "DOMAIN_ARTIFACT",
            TokenKind::DomainElement => "DOMAIN_ELEMENT",
            TokenKind::DomainProperty => "DOMAIN_PROPERTY",
            TokenKind::DomainAction => "DOMAIN_ACTION",
            TokenKind::Word => "WORD",
            TokenKind::Number => "NUMBER",
            TokenKind::String => "STRING",
            TokenKind::Dot => "DOT",
            TokenKind::Comma => "COMMA",
            TokenKind::Colon => "COLON",
            TokenKind::LParen => "LPAREN",
            TokenKind::RParen => "RPAREN",
            TokenKind::Eof => "EOF",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenLiteral {
    None,
    Number(NumberValue),
    String(String),
    DomainTerm(DomainTerm),
    ActionCandidates(Vec<DomainTerm>),
}

impl TokenLiteral {
    fn to_json(&self) -> JsonValue {
        match self {
            TokenLiteral::None => JsonValue::Null,
            TokenLiteral::Number(value) => value.to_json(),
            TokenLiteral::String(value) => json!(value),
            TokenLiteral::DomainTerm(term) => term.to_json(),
            TokenLiteral::ActionCandidates(candidates) => {
                if candidates.len() == 1 {
                    candidates[0].to_json()
                } else {
                    json!({
                        "candidates": candidates.iter().map(DomainTerm::to_json).collect::<Vec<_>>()
                    })
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub literal: TokenLiteral,
    pub span: SourceSpan,
}

impl Token {
    pub fn to_json(&self) -> JsonValue {
        json!({
            "type": self.kind.as_str(),
            "lexeme": self.lexeme,
            "literal": self.literal.to_json(),
            "line": self.span.line(),
            "column": self.span.column(),
            "end_line": self.span.end_line(),
            "end_column": self.span.end_column(),
            "span": self.span
        })
    }
}

pub struct Lexer<'a> {
    chars: Vec<char>,
    index: usize,
    line: usize,
    column: usize,
    domains: &'a DomainRegistry,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &str, domains: &'a DomainRegistry) -> Self {
        Self {
            chars: source.chars().collect(),
            index: 0,
            line: 1,
            column: 1,
            domains,
        }
    }

    pub fn tokens(mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let end = token.kind == TokenKind::Eof;
            tokens.push(token);
            if end {
                return Ok(tokens);
            }
        }
    }

    fn next_token(&mut self) -> Result<Token> {
        self.skip_ignored()?;
        if self.eof() {
            return Ok(Token {
                kind: TokenKind::Eof,
                lexeme: String::new(),
                literal: TokenLiteral::None,
                span: SourceSpan::new(self.line, self.column, self.line, self.column),
            });
        }

        let start_line = self.line;
        let start_column = self.column;
        let current = self.current().unwrap();

        let punctuation = match current {
            '.' => Some(TokenKind::Dot),
            ',' => Some(TokenKind::Comma),
            ':' => Some(TokenKind::Colon),
            '(' => Some(TokenKind::LParen),
            ')' => Some(TokenKind::RParen),
            _ => None,
        };
        if let Some(kind) = punctuation {
            self.advance();
            return Ok(Token {
                kind,
                lexeme: current.to_string(),
                literal: TokenLiteral::None,
                span: SourceSpan::new(start_line, start_column, self.line, self.column),
            });
        }

        if current == '"' {
            return self.string_token(start_line, start_column);
        }
        if current.is_ascii_digit() {
            return self.number_token(start_line, start_column);
        }
        if current.is_alphabetic() {
            return self.word_token(start_line, start_column);
        }

        Err(SemauriError::lex(
            "S101",
            format!("Unexpected character {current:?}"),
            SourceSpan::new(start_line, start_column, start_line, start_column + 1),
        ))
    }

    fn word_token(&mut self, line: usize, column: usize) -> Result<Token> {
        let mut text = String::new();
        while let Some(ch) = self.current() {
            if ch.is_alphanumeric() || matches!(ch, '_' | '\'' | '-') {
                text.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        let normalized = text.to_lowercase();
        let (kind, literal) = if let Some(kind) = keyword_kind(&normalized) {
            (kind, TokenLiteral::None)
        } else if is_color(&normalized) {
            (TokenKind::Color, TokenLiteral::String(normalized.clone()))
        } else if let Some(domain_literal) = self.domains.classify(&normalized) {
            let kind = match &domain_literal {
                TokenLiteral::DomainTerm(term) => term.category.token_kind(),
                TokenLiteral::ActionCandidates(_) => TokenKind::DomainAction,
                _ => TokenKind::Word,
            };
            (kind, domain_literal)
        } else {
            (TokenKind::Word, TokenLiteral::String(text.clone()))
        };

        Ok(Token {
            kind,
            lexeme: text,
            literal,
            span: SourceSpan::new(line, column, self.line, self.column),
        })
    }

    fn number_token(&mut self, line: usize, column: usize) -> Result<Token> {
        let mut text = String::new();
        while self.current().is_some_and(|ch| ch.is_ascii_digit()) {
            text.push(self.advance().unwrap());
        }
        if self.current() == Some('.')
            && self.peek_char().is_some_and(|ch| ch.is_ascii_digit())
        {
            text.push(self.advance().unwrap());
            while self.current().is_some_and(|ch| ch.is_ascii_digit()) {
                text.push(self.advance().unwrap());
            }
        }
        let value = if text.contains('.') {
            NumberValue::Float(text.parse::<f64>().unwrap())
        } else {
            NumberValue::Int(text.parse::<i64>().unwrap())
        };
        Ok(Token {
            kind: TokenKind::Number,
            lexeme: text,
            literal: TokenLiteral::Number(value),
            span: SourceSpan::new(line, column, self.line, self.column),
        })
    }

    fn string_token(&mut self, line: usize, column: usize) -> Result<Token> {
        self.advance();
        let mut value = String::new();
        while !self.eof() && self.current() != Some('"') {
            if self.current() == Some('\n') {
                return Err(SemauriError::lex(
                    "S102",
                    "Literal newlines are not allowed in strings; use an escaped newline",
                    SourceSpan::new(line, column, self.line, self.column),
                ));
            }
            if self.current() == Some('\\') {
                let escape_line = self.line;
                let escape_column = self.column;
                self.advance();
                let escaped = self.advance().ok_or_else(|| {
                    SemauriError::lex(
                        "S104",
                        "Unterminated string escape",
                        SourceSpan::new(line, column, self.line, self.column),
                    )
                })?;
                let replacement = match escaped {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '"' => '"',
                    '\\' => '\\',
                    other => {
                        return Err(SemauriError::lex(
                            "S104",
                            format!("Unsupported string escape \\{other}"),
                            SourceSpan::new(
                                escape_line,
                                escape_column,
                                self.line,
                                self.column,
                            ),
                        ))
                    }
                };
                value.push(replacement);
            } else {
                value.push(self.advance().unwrap());
            }
        }

        if self.eof() {
            return Err(SemauriError::lex(
                "S103",
                "Unterminated string",
                SourceSpan::new(line, column, self.line, self.column),
            ));
        }
        self.advance();
        Ok(Token {
            kind: TokenKind::String,
            lexeme: value.clone(),
            literal: TokenLiteral::String(value),
            span: SourceSpan::new(line, column, self.line, self.column),
        })
    }

    fn skip_ignored(&mut self) -> Result<()> {
        loop {
            while self.current().is_some_and(char::is_whitespace) {
                self.advance();
            }
            if self.current() == Some('#')
                || (self.current() == Some('/') && self.peek_char() == Some('/'))
            {
                while !self.eof() && self.current() != Some('\n') {
                    self.advance();
                }
                continue;
            }
            if self.current() == Some('/') && self.peek_char() == Some('*') {
                let start_line = self.line;
                let start_column = self.column;
                self.advance();
                self.advance();
                let mut depth = 1usize;
                while depth > 0 {
                    if self.eof() {
                        return Err(SemauriError::lex(
                            "S105",
                            "Unterminated block comment",
                            SourceSpan::new(start_line, start_column, self.line, self.column),
                        ));
                    }
                    if self.current() == Some('/') && self.peek_char() == Some('*') {
                        self.advance();
                        self.advance();
                        depth += 1;
                    } else if self.current() == Some('*') && self.peek_char() == Some('/') {
                        self.advance();
                        self.advance();
                        depth -= 1;
                    } else {
                        self.advance();
                    }
                }
                continue;
            }
            break;
        }
        Ok(())
    }

    fn current(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.index + 1).copied()
    }

    fn eof(&self) -> bool {
        self.index >= self.chars.len()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.current()?;
        self.index += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }
}

fn keyword_kind(word: &str) -> Option<TokenKind> {
    Some(match word {
        "create" => TokenKind::Create,
        "make" => TokenKind::Make,
        "let" => TokenKind::Let,
        "be" => TokenKind::Be,
        "set" => TokenKind::Set,
        "if" => TokenKind::If,
        "otherwise" | "else" => TokenKind::Otherwise,
        "end" => TokenKind::End,
        "for" => TokenKind::For,
        "every" => TokenKind::Every,
        "in" => TokenKind::In,
        "within" => TokenKind::Within,
        "list" => TokenKind::List,
        "is" => TokenKind::Is,
        "greater" => TokenKind::Greater,
        "less" => TokenKind::Less,
        "than" => TokenKind::Than,
        "equal" => TokenKind::Equal,
        "and" => TokenKind::And,
        "or" => TokenKind::Or,
        "not" => TokenKind::Not,
        "plus" => TokenKind::Plus,
        "minus" => TokenKind::Minus,
        "times" => TokenKind::Times,
        "divided" => TokenKind::Divided,
        "modulo" => TokenKind::Modulo,
        "by" => TokenKind::By,
        "true" | "false" => TokenKind::Boolean,
        "of" => TokenKind::Of,
        "to" => TokenKind::To,
        "a" | "an" | "the" => TokenKind::Article,
        "called" | "named" => TokenKind::Called,
        "add" => TokenKind::Add,
        "title" => TokenKind::Title,
        "it" => TokenKind::Pronoun,
        _ => return None,
    })
}

fn is_color(word: &str) -> bool {
    matches!(
        word,
        "black"
            | "white"
            | "red"
            | "green"
            | "blue"
            | "yellow"
            | "orange"
            | "purple"
            | "pink"
            | "gray"
            | "grey"
            | "brown"
    )
}

