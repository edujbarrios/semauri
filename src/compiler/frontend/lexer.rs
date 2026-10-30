#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Create,
    Make,
    Let,
    Be,
    Set,
    If,
    Unless,
    Otherwise,
    End,
    For,
    Every,
    In,
    Within,
    List,
    Length,
    Reverse,
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
    Take,
    Drop,
    From,
    Uppercase,
    Lowercase,
    Contains,
    At,
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
            TokenKind::Unless => "UNLESS",
            TokenKind::Otherwise => "OTHERWISE",
            TokenKind::End => "END",
            TokenKind::For => "FOR",
            TokenKind::Every => "EVERY",
            TokenKind::In => "IN",
            TokenKind::Within => "WITHIN",
            TokenKind::List => "LIST",
            TokenKind::Length => "LENGTH",
            TokenKind::Reverse => "REVERSE",
            TokenKind::Sum => "SUM",
            TokenKind::Average => "AVERAGE",
            TokenKind::Minimum => "MINIMUM",
            TokenKind::Maximum => "MAXIMUM",
            TokenKind::Any => "ANY",
            TokenKind::All => "ALL",
            TokenKind::Median => "MEDIAN",
            TokenKind::Sorted => "SORTED",
            TokenKind::Distinct => "DISTINCT",
            TokenKind::First => "FIRST",
            TokenKind::Last => "LAST",
            TokenKind::Take => "TAKE",
            TokenKind::Drop => "DROP",
            TokenKind::From => "FROM",
            TokenKind::Uppercase => "UPPERCASE",
            TokenKind::Lowercase => "LOWERCASE",
            TokenKind::Contains => "CONTAINS",
            TokenKind::At => "AT",
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

        if matches!(current, 'r' | 'R') {
            // Hash delimiters let raw strings contain their quote character.
            // Without hashes this is the existing r"..." / r'...' syntax.
            let hashes = self.chars[self.index + 1..]
                .iter()
                .take_while(|ch| **ch == '#')
                .count();
            if self.chars.get(self.index + 1 + hashes)
                .is_some_and(|quote| matches!(*quote, '"' | '\''))
            {
                return self.raw_string_token(start_line, start_column, hashes);
            }
        }

        if matches!(current, '"' | '\'') {
            if self.chars.get(self.index..self.index + 3)
                .is_some_and(|slice| slice.iter().all(|ch| *ch == current))
            {
                return self.multiline_string_token(start_line, start_column, current);
            }
            return self.string_token(start_line, start_column, current);
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
        // Prefixed literals remain integers; decimal fractions and exponents
        // continue to use the existing decimal-only parser.
        if self.current() == Some('0') {
            let radix = match self.peek_char() {
                Some('b' | 'B') => Some(2),
                Some('o' | 'O') => Some(8),
                Some('x' | 'X') => Some(16),
                Some('z' | 'Z') => Some(36),
                _ => None,
            };
            if let Some(radix) = radix {
                return self.radix_number_token(line, column, radix);
            }
        }
        let mut text = String::new();
        self.number_digits(&mut text, line, column)?;

        let mut is_float = false;
        if self.current() == Some('.')
            && self.peek_char().is_some_and(|ch| ch.is_ascii_digit())
        {
            is_float = true;
            text.push(self.advance().unwrap());
            self.number_digits(&mut text, line, column)?;
        }

        if matches!(self.current(), Some('e' | 'E')) {
            is_float = true;
            text.push(self.advance().unwrap());
            if matches!(self.current(), Some('+' | '-')) {
                text.push(self.advance().unwrap());
            }
            if !self.current().is_some_and(|ch| ch.is_ascii_digit()) {
                return Err(self.invalid_number(
                    line,
                    column,
                    "Scientific notation requires exponent digits",
                ));
            }
            self.number_digits(&mut text, line, column)?;
        }

        let normalized = text.replace('_', "");
        let value = if is_float {
            let number = normalized
                .parse::<f64>()
                .map_err(|_| self.invalid_number(line, column, "Invalid number literal"))?;
            if !number.is_finite() {
                return Err(self.invalid_number(
                    line,
                    column,
                    "Number literal is outside the finite range",
                ));
            }
            NumberValue::Float(number)
        } else {
            let number = normalized
                .parse::<i64>()
                .map_err(|_| self.invalid_number(line, column, "Integer literal is out of range"))?;
            NumberValue::Int(number)
        };
        Ok(Token {
            kind: TokenKind::Number,
            lexeme: text,
            literal: TokenLiteral::Number(value),
            span: SourceSpan::new(line, column, self.line, self.column),
        })
    }

    fn radix_number_token(&mut self, line: usize, column: usize, radix: u32) -> Result<Token> {
        let mut text = String::new();
        text.push(self.advance().unwrap()); // leading zero
        text.push(self.advance().unwrap()); // radix prefix
        let mut digits = String::new();
        while let Some(ch) = self.current() {
            if ch == '_' {
                if digits.is_empty()
                    || !self.peek_char().is_some_and(|next| next.is_digit(radix))
                {
                    return Err(self.invalid_number(
                        line, column, "Digit separators must appear between digits",
                    ));
                }
                text.push(self.advance().unwrap());
                digits.push('_');
            } else if ch.is_digit(radix) {
                text.push(self.advance().unwrap());
                digits.push(ch);
            } else if ch.is_ascii_alphanumeric() {
                return Err(self.invalid_number(line, column, "Invalid digit for integer base"));
            } else {
                break;
            }
        }
        if digits.is_empty() {
            return Err(self.invalid_number(line, column, "Expected digits after integer base prefix"));
        }
        let number = i64::from_str_radix(&digits.replace('_', ""), radix)
            .map_err(|_| self.invalid_number(line, column, "Integer literal is out of range"))?;
        Ok(Token {
            kind: TokenKind::Number,
            lexeme: text,
            literal: TokenLiteral::Number(NumberValue::Int(number)),
            span: SourceSpan::new(line, column, self.line, self.column),
        })
    }

    fn number_digits(&mut self, text: &mut String, line: usize, column: usize) -> Result<()> {
        while let Some(ch) = self.current() {
            if ch.is_ascii_digit() {
                text.push(self.advance().unwrap());
            } else if ch == '_' {
                if !self.peek_char().is_some_and(|next| next.is_ascii_digit()) {
                    return Err(self.invalid_number(
                        line,
                        column,
                        "Digit separators must appear between digits",
                    ));
                }
                text.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        Ok(())
    }

    fn invalid_number(&self, line: usize, column: usize, message: &str) -> SemauriError {
        SemauriError::lex(
            "S106",
            message,
            SourceSpan::new(line, column, self.line, self.column),
        )
    }

    fn raw_string_token(&mut self, line: usize, column: usize, hashes: usize) -> Result<Token> {
        self.advance(); // r/R prefix
        for _ in 0..hashes {
            self.advance(); // opening hash delimiters
        }
        let quote = self.advance().unwrap();
        let mut value = String::new();
        while !self.eof() {
            // The closing quote must be followed by precisely enough hashes.
            // A quote within a hash-delimited string is ordinary content.
            let closing = self.current() == Some(quote)
                && self.chars.get(self.index + 1..self.index + 1 + hashes)
                    .is_some_and(|suffix| suffix.iter().all(|ch| *ch == '#'));
            if closing {
                self.advance();
                for _ in 0..hashes {
                    self.advance();
                }
                return Ok(Token {
                    kind: TokenKind::String,
                    lexeme: value.clone(),
                    literal: TokenLiteral::String(value),
                    span: SourceSpan::new(line, column, self.line, self.column),
                });
            }
            if matches!(self.current(), Some('\n' | '\r')) {
                return Err(SemauriError::lex(
                    "S102",
                    "Literal newlines are not allowed in raw strings",
                    SourceSpan::new(line, column, self.line, self.column),
                ));
            }
            value.push(self.advance().unwrap());
        }
        Err(SemauriError::lex(
            "S103",
            "Unterminated raw string",
            SourceSpan::new(line, column, self.line, self.column),
        ))
    }

    fn multiline_string_token(&mut self, line: usize, column: usize, quote: char) -> Result<Token> {
        // Triple quotes create literal multiline UTF-8 blocks. Like raw
        // strings, backslashes are not decoded, so prompts and templates
        // can embed code and regular expressions without double escaping.
        for _ in 0..3 {
            self.advance();
        }
        let mut value = String::new();
        while !self.eof() {
            if self.chars.get(self.index..self.index + 3)
                .is_some_and(|slice| slice.iter().all(|ch| *ch == quote))
            {
                for _ in 0..3 {
                    self.advance();
                }
                return Ok(Token {
                    kind: TokenKind::String,
                    lexeme: value.clone(),
                    literal: TokenLiteral::String(value),
                    span: SourceSpan::new(line, column, self.line, self.column),
                });
            }
            value.push(self.advance().unwrap());
        }
        Err(SemauriError::lex(
            "S103",
            "Unterminated triple-quoted string",
            SourceSpan::new(line, column, self.line, self.column),
        ))
    }

    fn string_token(&mut self, line: usize, column: usize, quote: char) -> Result<Token> {
        self.advance();
        let mut value = String::new();
        while !self.eof() && self.current() != Some(quote) {
            if matches!(self.current(), Some('\n' | '\r')) {
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
                    '0' => '\0',
                    'b' => '\u{0008}',
                    'f' => '\u{000C}',
                    '"' => '"',
                    '\'' => '\'',
                    '\\' => '\\',
                    'x' => {
                        // Exactly two hexadecimal digits represent U+0000 through U+00FF.
                        let high = self.current()
                            .and_then(|ch| ch.to_digit(16))
                            .ok_or_else(|| SemauriError::lex(
                                "S104",
                                "Expected exactly two hexadecimal digits after '\\x'",
                                SourceSpan::new(escape_line, escape_column, self.line, self.column),
                            ))?;
                        self.advance();
                        let low = self.current()
                            .and_then(|ch| ch.to_digit(16))
                            .ok_or_else(|| SemauriError::lex(
                                "S104",
                                "Expected exactly two hexadecimal digits after '\\x'",
                                SourceSpan::new(escape_line, escape_column, self.line, self.column),
                            ))?;
                        self.advance();
                        char::from_u32((high << 4) | low).unwrap()
                    }
                    'u' => {
                        // Unicode scalar escapes require braces and 1-6 hex digits.
                        if self.current() != Some('{') {
                            return Err(SemauriError::lex(
                                "S104",
                                "Expected '{' after '\\u' in Unicode escape",
                                SourceSpan::new(escape_line, escape_column, self.line, self.column),
                            ));
                        }
                        self.advance();
                        let mut hex = String::new();
                        while self.current().is_some_and(|ch| ch.is_ascii_hexdigit()) {
                            if hex.len() == 6 {
                                return Err(SemauriError::lex(
                                    "S104",
                                    "Unicode escape supports at most six hex digits",
                                    SourceSpan::new(escape_line, escape_column, self.line, self.column),
                                ));
                            }
                            hex.push(self.advance().unwrap());
                        }
                        if hex.is_empty() || self.current() != Some('}') {
                            return Err(SemauriError::lex(
                                "S104",
                                "Expected 1-6 hexadecimal digits followed by '}' in Unicode escape",
                                SourceSpan::new(escape_line, escape_column, self.line, self.column),
                            ));
                        }
                        self.advance();
                        u32::from_str_radix(&hex, 16)
                            .ok()
                            .and_then(char::from_u32)
                            .ok_or_else(|| {
                                SemauriError::lex(
                                    "S104",
                                    "Unicode escape must be a valid Unicode scalar value",
                                    SourceSpan::new(
                                        escape_line,
                                        escape_column,
                                        self.line,
                                        self.column,
                                    ),
                                )
                            })?
                    }
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
        "unless" => TokenKind::Unless,
        "otherwise" | "else" => TokenKind::Otherwise,
        "end" => TokenKind::End,
        "for" => TokenKind::For,
        "every" => TokenKind::Every,
        "in" => TokenKind::In,
        "within" => TokenKind::Within,
        "list" => TokenKind::List,
        "length" => TokenKind::Length,
        "reverse" => TokenKind::Reverse,
        "sum" => TokenKind::Sum,
        "average" => TokenKind::Average,
        "minimum" => TokenKind::Minimum,
        "maximum" => TokenKind::Maximum,
        "any" => TokenKind::Any,
        "all" => TokenKind::All,
        "median" => TokenKind::Median,
        "sorted" => TokenKind::Sorted,
        "distinct" => TokenKind::Distinct,
        "first" => TokenKind::First,
        "last" => TokenKind::Last,
        "take" => TokenKind::Take,
        "drop" => TokenKind::Drop,
        "from" => TokenKind::From,
        "uppercase" => TokenKind::Uppercase,
        "lowercase" => TokenKind::Lowercase,
        "contains" => TokenKind::Contains,
        "at" => TokenKind::At,
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

