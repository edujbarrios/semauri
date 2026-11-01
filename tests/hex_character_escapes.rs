// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::Compiler;

#[test]
fn hex_escapes_decode_to_latin1_unicode_scalars() {
    let compiler = Compiler::new();
    let source = r#"let letters_value be "\x41\x42\xE9".
let alternate be '\x41\x42\xE9'.
assert letters_value is equal to alternate.
assert letters_value is equal to "ABé".
print alternate."#;
    compiler.analyze(source).unwrap();
    let ast = compiler.parse(source).unwrap().to_json();
    assert_eq!(ast["statements"][0]["value"]["value"], "ABé");
    assert_eq!(ast["statements"][1]["value"]["value"], "ABé");
    let token = compiler.tokenize(r#"print "\x00\xFF"."#).unwrap().into_iter()
        .find(|token| token.to_json()["type"] == "STRING").unwrap();
    assert_eq!(token.to_json()["literal"], "\0ÿ");
}

#[test]
fn hex_escapes_require_exactly_two_hex_digits() {
    let compiler = Compiler::new();
    for source in [
        r#"print "\x"."#, r#"print "\x0"."#, r#"print "\xGG"."#,
        r#"print "\x0G"."#, r#"print '\xG0'."#,
    ] {
        let error = compiler.tokenize(source).unwrap_err();
        assert_eq!(error.code, "S104", "{source}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn formatter_preserves_hex_escape_spelling() {
    let compiler = Compiler::new();
    let before = r#"LET letters_value BE '\x41\xE9'. PRINT letters_value."#;
    let formatted = r#"let letters_value be '\x41\xE9'. print letters_value."#;
    assert_eq!(compiler.format_source(before).unwrap(), formatted);
    assert_eq!(compiler.format_source(formatted).unwrap(), formatted);
    compiler.analyze(r#"let raw_value be r"\x41". assert raw_value is equal to "\\x41". print raw_value."#).unwrap();
}
