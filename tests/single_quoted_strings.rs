// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::Compiler;

#[test]
fn single_quoted_text_supports_embedded_double_quotes_and_escapes() {
    let compiler = Compiler::new();
    let source = r#"let caption_value be 'She said "hello"'.
let apology be 'it\'s fine'.
assert caption_value is equal to "She said \"hello\"".
assert apology is equal to "it's fine".
print caption_value."#;
    compiler.analyze(source).unwrap();
    let ast = compiler.parse(source).unwrap().to_json();
    assert_eq!(ast["statements"][0]["value"]["value"], "She said \"hello\"");
    assert_eq!(ast["statements"][1]["value"]["value"], "it's fine");
    let strings: Vec<_> = compiler.tokenize(source).unwrap().into_iter()
        .filter(|token| token.to_json()["type"] == "STRING")
        .collect();
    assert!(strings.len() >= 4);
    assert_eq!(strings[0].to_json()["line"], 1);
}

#[test]
fn single_quote_errors_use_existing_located_diagnostics() {
    let compiler = Compiler::new();
    let missing_quote = compiler.tokenize("let x be 'unfinished").unwrap_err();
    assert_eq!(missing_quote.code, "S103");
    assert!(missing_quote.span.is_some());
    let bad_escape = compiler.tokenize(r#"let x be 'bad\q'."#).unwrap_err();
    assert_eq!(bad_escape.code, "S104");
    assert!(bad_escape.span.is_some());
}

#[test]
fn formatter_preserves_quote_style_and_ordinary_apostrophes() {
    let compiler = Compiler::new();
    let source = "LET message BE 'Hello, \"world\"'.\nPRINT message.\n";
    let expected = "let message be 'Hello, \"world\"'.\nprint message.\n";
    let formatted = compiler.format_source(source).unwrap();
    assert_eq!(formatted, expected);
    assert_eq!(compiler.format_source(&formatted).unwrap(), expected);
    compiler.analyze("create a web called Writer's Studio.").unwrap();
    compiler.parse("let name be \"it's fine\".").unwrap();
}
