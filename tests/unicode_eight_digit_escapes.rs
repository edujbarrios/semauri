// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::Compiler;

#[test]
fn fixed_width_unicode_escapes_support_ascii_bmp_and_astral_planes() {
    let compiler = Compiler::new();
    let source = r#"let unicode_value be "\U00000041\U000000E9\U0001F600".
let single_quote_value be '\U00000041\U000000e9\U0001f600'.
assert unicode_value is equal to single_quote_value.
assert unicode_value is equal to "Aé😀".
print unicode_value."#;
    compiler.analyze(source).unwrap();
    let ast = compiler.parse(source).unwrap().to_json();
    assert_eq!(ast["statements"][0]["value"]["value"], "Aé😀");
    assert_eq!(ast["statements"][1]["value"]["value"], "Aé😀");
}

#[test]
fn bad_fixed_width_unicode_escapes_have_source_aware_diagnostics() {
    let compiler = Compiler::new();
    for source in [
        r#"print "\U"."#, r#"print "\U0001"."#,
        r#"print "\U000000GG"."#, r#"print '\U0000D800'."#,
        r#"print "\U00110000"."#,
    ] {
        let error = compiler.tokenize(source).unwrap_err();
        assert_eq!(error.code, "S104", "{source}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn unicode_spelling_and_raw_string_compatibility() {
    let compiler = Compiler::new();
    let before = r#"LET unicode_value BE '\U0001F600'. PRINT unicode_value."#;
    let expected = r#"let unicode_value be '\U0001F600'. print unicode_value."#;
    assert_eq!(compiler.format_source(before).unwrap(), expected);
    compiler.analyze(r#"let raw_value be r"\U0001F600". assert raw_value is equal to "\\U0001F600". print raw_value."#).unwrap();
    compiler.analyze(r#"let brace_value be "\u{1F600}". let eight_value be "\U0001F600". assert brace_value is equal to eight_value. print eight_value."#).unwrap();
}
