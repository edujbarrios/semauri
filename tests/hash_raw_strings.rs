// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::Compiler;

#[test]
fn hash_raw_strings_can_embed_quotes_and_backslashes() {
    let compiler = Compiler::new();
    let source = r###"let quote_value be r#"She said "hello" and used \n literally"#.
let double_hash_value be R##"A "quoted" value and "# together"##.
let single_hash_value be r#'It's a "quote" and \t'#.
assert quote_value is equal to 'She said "hello" and used \\n literally'.
assert double_hash_value is equal to 'A "quoted" value and "# together'.
assert single_hash_value is equal to "It's a \"quote\" and \\t".
print quote_value."###;
    compiler.analyze(source).unwrap();
    let ast = compiler.parse(source).unwrap().to_json();
    assert_eq!(ast["statements"][0]["value"]["value"], "She said \"hello\" and used \\n literally");
    assert_eq!(ast["statements"][1]["value"]["value"], "A \"quoted\" value and \"# together");
    assert_eq!(ast["statements"][2]["value"]["value"], "It's a \"quote\" and \\t");
}

#[test]
fn hash_raw_strings_preserve_source_spelling_in_formatter() {
    let compiler = Compiler::new();
    let before = r###"LET quote_value BE r#"a "quote" \x41"#. PRINT quote_value."###;
    let after = r###"let quote_value be r#"a "quote" \x41"#. print quote_value."###;
    assert_eq!(compiler.format_source(before).unwrap(), after);
    assert_eq!(compiler.format_source(after).unwrap(), after);
    let values: Vec<_> = compiler.tokenize(before).unwrap().into_iter()
        .filter(|token| token.to_json()["type"] == "STRING").collect();
    assert_eq!(values.len(), 1);
    assert_eq!(values[0].to_json()["literal"], "a \"quote\" \\x41");
    assert_eq!(values[0].to_json()["column"], 20);
}

#[test]
fn hash_delimiters_still_reject_unterminated_and_multiline_literals() {
    let compiler = Compiler::new();
    for source in [r###"print r#"unfinished"."###, r####"print r##"unmatched "#."####] {
        let error = compiler.tokenize(source).unwrap_err();
        assert_eq!(error.code, "S103", "{source}");
        assert!(error.span.is_some());
    }
    let error = compiler.tokenize("print r#\"line\nbreak\"#.").unwrap_err();
    assert_eq!(error.code, "S102");
    // Existing unprefixed and un-hashed forms remain valid.
    compiler.analyze("let simple be r'path\\value'. print simple.").unwrap();
    compiler.analyze("let ordinary be 'quote'. print ordinary.").unwrap();
}
