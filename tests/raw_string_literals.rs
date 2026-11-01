// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::Compiler;

#[test]
fn raw_strings_preserve_slashes_and_unescaped_opposite_quotes() {
    let compiler = Compiler::new();
    let source = r##"let path_value be r'C:\Temp\logs'.
let message be R'He said "hello"'.
let line_marker be r"\n".
assert path_value is equal to "C:\\Temp\\logs".
assert message is equal to 'He said "hello"'.
assert line_marker is equal to "\\n".
print path_value."##;
    compiler.analyze(source).unwrap();
    let parsed = compiler.parse(source).unwrap().to_json();
    assert_eq!(parsed["statements"][0]["value"]["value"], r"C:\Temp\logs");
    assert_eq!(parsed["statements"][1]["value"]["value"], "He said \"hello\"");
    assert_eq!(parsed["statements"][2]["value"]["value"], r"\n");
}

#[test]
fn raw_strings_keep_span_and_formatting() {
    let compiler = Compiler::new();
    let source = r#"LET raw_value BE r"\n". PRINT raw_value."#;
    let expected = r#"let raw_value be r"\n". print raw_value."#;
    assert_eq!(compiler.format_source(source).unwrap(), expected);
    assert_eq!(compiler.format_source(expected).unwrap(), expected);
    let values: Vec<_> = compiler.tokenize(source).unwrap().into_iter()
        .filter(|token| token.to_json()["type"] == "STRING")
        .collect();
    assert_eq!(values.len(), 1);
    assert_eq!(values[0].to_json()["literal"], r"\n");
    assert_eq!(values[0].to_json()["column"], 18);
}

#[test]
fn raw_errors_and_identifier_compatibility() {
    let compiler = Compiler::new();
    let unfinished = compiler.tokenize("print r'bad.").unwrap_err();
    assert_eq!(unfinished.code, "S103");
    let newline = compiler.tokenize("print r'line\nbreak'.").unwrap_err();
    assert_eq!(newline.code, "S102");
    compiler.analyze("let r be 5. let raw_value be 6. print r plus raw_value.").unwrap();
}
