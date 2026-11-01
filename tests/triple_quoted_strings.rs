// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::Compiler;

#[test]
fn triple_quoted_text_preserves_newlines_quotes_and_backslashes() {
    let compiler = Compiler::new();
    let source = r####"let prose_value be """First "quoted" line
Second \n literal # not a comment
Third line""".
let alternate_value be '''First "quoted" line
Second \n literal # not a comment
Third line'''.
assert prose_value is equal to alternate_value.
print prose_value."####;
    compiler.analyze(source).unwrap();
    let ast = compiler.parse(source).unwrap().to_json();
    let expected = "First \"quoted\" line\nSecond \\n literal # not a comment\nThird line";
    assert_eq!(ast["statements"][0]["value"]["value"], expected);
    assert_eq!(ast["statements"][1]["value"]["value"], expected);
    let tokens: Vec<_> = compiler.tokenize(source).unwrap().into_iter()
        .filter(|token| token.to_json()["type"] == "STRING").collect();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].to_json()["line"], 1);
    assert_eq!(tokens[0].to_json()["end_line"], 3);
}

#[test]
fn triple_quoted_blocks_round_trip_without_reformatting_contents() {
    let compiler = Compiler::new();
    let source = "LET content_value BE \"\"\"KEEP Case\n  spacing and \\\\t\"\"\".\nPRINT content_value.\n";
    let expected = "let content_value be \"\"\"KEEP Case\n  spacing and \\\\t\"\"\".\nprint content_value.\n";
    assert_eq!(compiler.format_source(source).unwrap(), expected);
    assert_eq!(compiler.format_source(expected).unwrap(), expected);
    compiler.analyze(expected).unwrap();
}

#[test]
fn triple_strings_report_unterminated_blocks_and_keep_short_strings() {
    let compiler = Compiler::new();
    for source in ["print \"\"\"unfinished.", "print '''unfinished."] {
        let error = compiler.tokenize(source).unwrap_err();
        assert_eq!(error.code, "S103");
        assert!(error.span.is_some());
    }
    compiler.analyze("let short_value be \"text\". let raw_value be r'\\n'. print short_value plus raw_value.").unwrap();
}
