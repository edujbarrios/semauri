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
        for (index, token) in tokens.iter().enumerate() {
            // Contextual function syntax is canonicalized only at operator positions.
            let is_define = token.kind == TokenKind::Word
                && token.lexeme.eq_ignore_ascii_case("define")
                && tokens.get(index + 1).is_some_and(|next| next.lexeme.eq_ignore_ascii_case("function"));
            let is_function = token.kind == TokenKind::Word
                && token.lexeme.eq_ignore_ascii_case("function")
                && index > 0 && tokens[index - 1].lexeme.eq_ignore_ascii_case("define");
            let is_invoke = token.kind == TokenKind::Word
                && token.lexeme.eq_ignore_ascii_case("invoke")
                && tokens.get(index + 1).is_some_and(|next| next.kind == TokenKind::Word)
                && tokens.get(index + 2).is_some_and(|next| next.kind == TokenKind::To);
            // 'then' is only a delimiter after an 'if' within this
            // statement; existing identifiers or prose containing 'then'
            // remain untouched.
            let is_then = token.kind == TokenKind::Word
                && token.lexeme.eq_ignore_ascii_case("then")
                && tokens[..index].iter().rev()
                    .take_while(|previous| previous.kind != TokenKind::Dot)
                    .any(|previous| previous.kind == TokenKind::If);
            let is_product = token.kind == TokenKind::Word
                && token.lexeme.eq_ignore_ascii_case("product")
                && tokens.get(index + 1).is_some_and(|next| next.kind == TokenKind::Of);
            let is_absolute = token.kind == TokenKind::Word
                && token.lexeme.eq_ignore_ascii_case("absolute")
                && tokens.get(index + 1).is_some_and(|next| next.kind == TokenKind::Of);
            let is_flatten = token.kind == TokenKind::Word
                && token.lexeme.eq_ignore_ascii_case("flatten")
                && tokens.get(index + 1).is_some_and(|next| next.kind == TokenKind::Of);
            let is_text_predicate = token.kind == TokenKind::Word
                && ((matches!(token.lexeme.to_ascii_lowercase().as_str(), "starts" | "ends")
                    && tokens.get(index + 1).is_some_and(|next| next.lexeme.eq_ignore_ascii_case("with")))
                    || (token.lexeme.eq_ignore_ascii_case("with")
                        && index > 0
                        && matches!(tokens[index - 1].lexeme.to_ascii_lowercase().as_str(), "starts" | "ends")));
            let is_json_field = matches!(token.kind, TokenKind::Word | TokenKind::DomainAction)
                && ((token.lexeme.eq_ignore_ascii_case("json")
                    && tokens.get(index + 1).is_some_and(|next| matches!(
                        next.lexeme.to_ascii_lowercase().as_str(),
                        "string" | "number" | "boolean" | "strings" | "numbers" | "booleans"
                    ))
                    && tokens.get(index + 2).is_some_and(|next| next.kind == TokenKind::At))
                    || (matches!(token.lexeme.to_ascii_lowercase().as_str(),
                        "string" | "number" | "boolean" | "strings" | "numbers" | "booleans")
                        && index > 0 && tokens[index - 1].lexeme.eq_ignore_ascii_case("json")
                        && tokens.get(index + 1).is_some_and(|next| next.kind == TokenKind::At)));
            let is_json_object_operation = token.kind == TokenKind::Word
                && ((token.lexeme.eq_ignore_ascii_case("keys")
                    && tokens.get(index + 1).is_some_and(|next| next.kind == TokenKind::Of)
                    && tokens.get(index + 2).is_some_and(|next| next.lexeme.eq_ignore_ascii_case("json")))
                || (token.lexeme.eq_ignore_ascii_case("has")
                    && tokens.get(index + 1).is_some_and(|next| next.lexeme.eq_ignore_ascii_case("json"))
                    && tokens.get(index + 2).is_some_and(|next| next.lexeme.eq_ignore_ascii_case("key")))
                || (token.lexeme.eq_ignore_ascii_case("json")
                    && ((index > 1 && tokens[index - 2].lexeme.eq_ignore_ascii_case("keys")
                        && tokens[index - 1].kind == TokenKind::Of)
                        || (index > 0 && tokens[index - 1].lexeme.eq_ignore_ascii_case("has"))))
                || (token.lexeme.eq_ignore_ascii_case("key")
                    && index > 1 && tokens[index - 2].lexeme.eq_ignore_ascii_case("has")
                    && tokens[index - 1].lexeme.eq_ignore_ascii_case("json")));
            let is_json_decode = token.kind == TokenKind::Word
                && (
                    (token.lexeme.eq_ignore_ascii_case("decode")
                        && tokens.get(index + 1).is_some_and(|next| next.lexeme.eq_ignore_ascii_case("json"))
                        && tokens.get(index + 2).is_some_and(|next| next.lexeme.eq_ignore_ascii_case("as"))
                        && tokens.get(index + 3).is_some_and(|next| matches!(
                            next.lexeme.to_ascii_lowercase().as_str(),
                            "string" | "number" | "boolean" | "strings" | "numbers" | "booleans"
                        ))
                        && tokens.get(index + 4).is_some_and(|next| next.kind == TokenKind::From))
                    || (token.lexeme.eq_ignore_ascii_case("json")
                        && index > 0 && tokens[index - 1].lexeme.eq_ignore_ascii_case("decode")
                        && tokens.get(index + 1).is_some_and(|next| next.lexeme.eq_ignore_ascii_case("as")))
                    || (token.lexeme.eq_ignore_ascii_case("as")
                        && index > 1 && tokens[index - 2].lexeme.eq_ignore_ascii_case("decode")
                        && tokens[index - 1].lexeme.eq_ignore_ascii_case("json"))
                    || (matches!(token.lexeme.to_ascii_lowercase().as_str(),
                            "string" | "number" | "boolean" | "strings" | "numbers" | "booleans")
                        && index > 2 && tokens[index - 3].lexeme.eq_ignore_ascii_case("decode")
                        && tokens[index - 2].lexeme.eq_ignore_ascii_case("json")
                        && tokens[index - 1].lexeme.eq_ignore_ascii_case("as")
                        && tokens.get(index + 1).is_some_and(|next| next.kind == TokenKind::From))
                );
            let is_parse_conversion = token.kind == TokenKind::Word
                && ((token.lexeme.eq_ignore_ascii_case("parse")
                    && tokens.get(index + 1).is_some_and(|next|
                        matches!(next.lexeme.to_ascii_lowercase().as_str(), "number" | "boolean"))
                    && tokens.get(index + 2).is_some_and(|next| next.kind == TokenKind::From))
                    || (matches!(token.lexeme.to_ascii_lowercase().as_str(), "number" | "boolean")
                        && index > 0 && tokens[index - 1].lexeme.eq_ignore_ascii_case("parse")
                        && tokens.get(index + 1).is_some_and(|next| next.kind == TokenKind::From)));
            let is_text_operation = token.kind == TokenKind::Word
                && matches!(token.lexeme.to_ascii_lowercase().as_str(), "trim" | "words" | "lines" | "json" | "split" | "join")
                && tokens.get(index + 1).is_some_and(|next| next.kind == TokenKind::Of);
            let is_join_with = token.kind == TokenKind::Word
                && token.lexeme.eq_ignore_ascii_case("with")
                && tokens[..index].iter().rev()
                    .take_while(|previous| previous.kind != TokenKind::Dot)
                    .any(|previous| previous.lexeme.eq_ignore_ascii_case("join"));
            let is_repeat = token.kind == TokenKind::Word
                && token.lexeme.eq_ignore_ascii_case("repeat")
                && tokens.get(index + 1).is_some_and(|next|
                    matches!(next.kind, TokenKind::Number | TokenKind::Word
                        | TokenKind::Minus | TokenKind::LParen));
            // Map/filter are contextual words, not reserved identifiers or label text.
            if token.kind == TokenKind::Word
                && !is_define && !is_function && !is_invoke && !is_then && !is_repeat && !is_flatten && !is_absolute && !is_product && !is_text_operation && !is_text_predicate && !is_join_with && !is_parse_conversion && !is_json_decode && !is_json_object_operation && !is_json_field
                && !(matches!(token.lexeme.to_ascii_lowercase().as_str(), "map" | "filter" | "count" | "reduce" | "zip" | "partition")
                    && tokens.get(index + 1).is_some_and(|next| next.kind == TokenKind::Every))
                && !(token.lexeme.eq_ignore_ascii_case("range")
                    && tokens.get(index + 1).is_some_and(|next| next.kind == TokenKind::From))
            {
                continue;
            }
            // Contextual first/last preserve free-form labels and identifiers:
            // only a following 'of' makes them operators to canonicalize.
            if matches!(token.kind, TokenKind::First | TokenKind::Last)
                && !tokens.get(index + 1).is_some_and(|next| next.kind == TokenKind::Of)
            {
                continue;
            }
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
                    | TokenKind::Median
                    | TokenKind::Sorted
                    | TokenKind::Distinct
                    | TokenKind::First
                    | TokenKind::Last
                    | TokenKind::Take
                    | TokenKind::Drop
                    | TokenKind::From
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
                    | TokenKind::Word
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
