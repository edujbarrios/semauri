// Canonical lowercase source spelling. Formatting changes recognized syntax
// words only: comments, string contents and free-form identifiers/labels are
// left byte-for-byte unchanged. The lexer already recognizes commands without
// regard to case, so this is not a breaking grammar change.
impl Compiler {
    pub fn format_source(&self, source: &str) -> Result<String> {
        let tokens = self.tokenize(source)?;
        let chars = source.chars().collect::<Vec<_>>();
        let mut byte_offsets = source.char_indices().map(|(idx, _)| idx).collect::<Vec<_>>();
        byte_offsets.push(source.len());

        let mut starts = vec![0usize];
        for (i, ch) in chars.iter().enumerate() {
            if *ch == '\n' {
                starts.push(i + 1);
            }
        }
        let mut rendered = String::with_capacity(source.len());
        let mut cursor = 0usize;
        for token in tokens {
            if !matches!(
                token.kind,
                TokenKind::Create
                    | TokenKind::Make
                    | TokenKind::Let
                    | TokenKind::Be
                    | TokenKind::Set
                    | TokenKind::If
                    | TokenKind::Unless
                    | TokenKind::Otherwise
                    | TokenKind::End
                    | TokenKind::For
                    | TokenKind::Every
                    | TokenKind::In
                    | TokenKind::Within
                    | TokenKind::List
                    | TokenKind::Length
                    | TokenKind::Reverse
                    | TokenKind::Sum
                    | TokenKind::Average
                    | TokenKind::Minimum
                    | TokenKind::Maximum
                    | TokenKind::Any
                    | TokenKind::All
                    | TokenKind::Uppercase
                    | TokenKind::Lowercase
                    | TokenKind::Contains
                    | TokenKind::At
                    | TokenKind::Is
                    | TokenKind::Greater
                    | TokenKind::Less
                    | TokenKind::Than
                    | TokenKind::Equal
                    | TokenKind::And
                    | TokenKind::Or
                    | TokenKind::Not
                    | TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Times
                    | TokenKind::Divided
                    | TokenKind::Modulo
                    | TokenKind::By
                    | TokenKind::Boolean
                    | TokenKind::Of
                    | TokenKind::To
                    | TokenKind::Article
                    | TokenKind::Called
                    | TokenKind::Add
                    | TokenKind::Title
                    | TokenKind::Pronoun
                    | TokenKind::Color
                    | TokenKind::DomainArtifact
                    | TokenKind::DomainElement
                    | TokenKind::DomainProperty
                    | TokenKind::DomainAction
            ) {
                continue;
            }
            // Lexical command tokens do not span physical lines.
            let start = starts[token.span.line() - 1] + token.span.column() - 1;
            let end = starts[token.span.end_line() - 1] + token.span.end_column() - 1;
            if start < cursor || end > chars.len() {
                return Err(SemauriError::semantic(
                    "S350",
                    "Cannot format an invalid token span",
                    Some(token.span),
                    None,
                ));
            }
            rendered.push_str(&source[byte_offsets[cursor]..byte_offsets[start]]);
            rendered.push_str(&token.lexeme.to_lowercase());
            cursor = end;
        }
        rendered.push_str(&source[byte_offsets[cursor]..]);
        Ok(rendered)
    }
}
