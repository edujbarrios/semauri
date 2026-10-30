// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::Compiler;

#[test]
fn control_escapes_decode_in_both_quote_styles() {
    let compiler = Compiler::new();
    let source = r#"let controls be "a\0b\bc\fd".
let alternate be 'a\0b\bc\fd'.
assert controls is equal to alternate.
print "ok"."#;
    compiler.analyze(source).unwrap();
    let parsed = compiler.parse(source).unwrap().to_json();
    let decoded = "a\0b\u{0008}c\u{000c}d";
    assert_eq!(parsed["statements"][0]["value"]["value"], decoded);
    assert_eq!(parsed["statements"][1]["value"]["value"], decoded);
}

#[test]
fn controls_keep_lexical_spans_and_formatting() {
    let compiler = Compiler::new();
    let source = r#"LET control_value BE '\0\b\f'. PRINT "ok"."#;
    let formatted = compiler.format_source(source).unwrap();
    assert_eq!(formatted, r#"let control_value be '\0\b\f'. print "ok"."#);
    let tokens = compiler.tokenize(source).unwrap();
    let encoded = tokens.iter().find(|token| token.to_json()["type"] == "STRING").unwrap();
    assert_eq!(encoded.to_json()["column"], 22);
    assert_eq!(encoded.to_json()["literal"], "\0\u{0008}\u{000c}");
}

#[test]
fn unrelated_unsupported_escapes_still_error() {
    let compiler = Compiler::new();
    for code in [r#"print "\v"."#, r#"print '\q'."#] {
        let error = compiler.tokenize(code).unwrap_err();
        assert_eq!(error.code, "S104");
        assert!(error.span.is_some());
    }
}
