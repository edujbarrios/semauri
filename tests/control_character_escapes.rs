// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::Compiler;

#[test]
fn control_escapes_decode_in_both_quote_styles() {
    let compiler = Compiler::new();
    let source = r#"let controls be "a\0b\bc\fd\ae\vf".
let alternate be 'a\0b\bc\fd\ae\vf'.
assert controls is equal to alternate.
print "ok"."#;
    compiler.analyze(source).unwrap();
    let parsed = compiler.parse(source).unwrap().to_json();
    let decoded = "a\0b\u{0008}c\u{000c}d\u{0007}e\u{000b}f";
    assert_eq!(parsed["statements"][0]["value"]["value"], decoded);
    assert_eq!(parsed["statements"][1]["value"]["value"], decoded);
}

#[test]
fn controls_keep_lexical_spans_and_formatting() {
    let compiler = Compiler::new();
    let source = r#"LET control_value BE '\0\b\f\a\v'. PRINT "ok"."#;
    let formatted = compiler.format_source(source).unwrap();
    assert_eq!(formatted, r#"let control_value be '\0\b\f\a\v'. print "ok"."#);
    let tokens = compiler.tokenize(source).unwrap();
    let encoded = tokens.iter().find(|token| token.to_json()["type"] == "STRING").unwrap();
    assert_eq!(encoded.to_json()["column"], 22);
    assert_eq!(encoded.to_json()["literal"], "\0\u{0008}\u{000c}\u{0007}\u{000b}");
}

#[test]
fn raw_strings_preserve_backslashes_in_control_escape_text() {
    let compiler = Compiler::new();
    let source = r#"let raw_value be r"\a\v".
assert raw_value is equal to "\\a\\v"."#;
    compiler.analyze(source).unwrap();
}

#[test]
fn unrelated_unsupported_escapes_still_error() {
    let compiler = Compiler::new();
    for code in [r#"print "\q"."#, r#"print '\y'."#] {
        let error = compiler.tokenize(code).unwrap_err();
        assert_eq!(error.code, "S104");
        assert!(error.span.is_some());
    }
}
