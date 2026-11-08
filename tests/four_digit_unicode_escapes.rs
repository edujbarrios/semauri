// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::Compiler;

#[test]
fn four_digit_unicode_escapes_decode_in_both_quote_styles() {
    let compiler = Compiler::new();
    let source = r#"let greeting be "\u0041\u00e9\u4F60\u597D".
let alternate be '\u0041\u00E9\u4f60\u597d'.
assert greeting is equal to alternate.
assert greeting is equal to "Aé你好".
assert "\u{1F600}" is equal to "\U0001F600"."#;
    compiler.analyze(source).unwrap();
    let ast = compiler.parse(source).unwrap().to_json();
    assert_eq!(ast["statements"][0]["value"]["value"], "Aé你好");
    assert_eq!(ast["statements"][1]["value"]["value"], "Aé你好");
}

#[test]
fn malformed_four_digit_unicode_escapes_keep_source_aware_errors() {
    let compiler = Compiler::new();
    for source in [
        r#"print "\u"."#,
        r#"print "\u000"."#,
        r#"print "\u00G0"."#,
        r#"print '\uD800'."#,
        r#"print "\uDEAD"."#,
    ] {
        let error = compiler.tokenize(source).unwrap_err();
        assert_eq!(error.code, "S104", "{source}: {error}");
        assert!(error.span.is_some(), "{source}: {error}");
    }
}

#[test]
fn escape_character_and_fixed_width_unicode_preserve_formatting_and_raw_text() {
    let compiler = Compiler::new();
    let source = r#"let escape_value be "\e".
let alternate be '\x1B'.
assert escape_value is equal to alternate.
assert "\u0000" is equal to "\0".
assert "\u0041" is equal to "\x41"."#;
    compiler.analyze(source).unwrap();
    let parsed = compiler.parse(source).unwrap().to_json();
    assert_eq!(parsed["statements"][0]["value"]["value"], "\u{001B}");

    let original = r#"LET text_value BE '\u0041\u00e9\e'. PRINT text_value."#;
    let expected = r#"let text_value be '\u0041\u00e9\e'. print text_value."#;
    assert_eq!(compiler.format_source(original).unwrap(), expected);
    assert_eq!(compiler.format_source(expected).unwrap(), expected);

    compiler.analyze(r#"let raw_value be r"\u0041\e".
assert raw_value is equal to "\\u0041\\e"."#).unwrap();
    compiler.analyze(r#"let block_value be """\u0041\e""".
assert block_value is equal to "\\u0041\\e"."#).unwrap();
}

#[test]
fn unicode_example_is_valid_checked_language_code() {
    Compiler::new().analyze(include_str!("../examples/unicode_four_digit_escapes.suri")).unwrap();
}
