// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;

#[test]
fn leading_utf8_bom_is_accepted_without_shifting_token_positions() {
    let compiler = Compiler::new();
    let source = "\u{feff}let value be 42.\r\nassert value is equal to 42.\r\n";
    compiler.analyze(source).expect("BOM-prefixed source must compile");
    let tokens = compiler.tokenize(source).unwrap();
    assert_eq!(tokens[0].lexeme, "let");
    assert_eq!(tokens[0].to_json()["line"], 1);
    assert_eq!(tokens[0].to_json()["column"], 1);
    assert_eq!(tokens[0].to_json()["type"], "LET");
}

#[test]
fn bom_is_supported_on_imported_modules_and_first_line_imports() {
    let dir = std::env::temp_dir().join(format!(
        "semauri-bom-{}-{:?}", std::process::id(), std::thread::current().id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let lib = dir.join("lib.suri");
    let main = dir.join("main.suri");
    fs::write(&lib, "\u{feff}define function double taking n as number to (n times 2).\n").unwrap();
    fs::write(&main, "\u{feff}import \"lib.suri\".\nassert invoke double to 21 is equal to 42.\n").unwrap();
    let result = run_cli(&["check".into(), main.to_string_lossy().into_owned()]);
    let _ = fs::remove_dir_all(&dir);
    assert_eq!(result.status, 0, "{} {}", result.stdout, result.stderr);
}

#[test]
fn formatter_normalizes_initial_bom_without_corrupting_source_spans() {
    let formatted = Compiler::new()
        .format_source("\u{feff}LET answer BE 42.\nPRINT answer.")
        .expect("BOM-prefixed source can be formatted");
    assert_eq!(formatted, "let answer be 42.\nprint answer.");
}

#[test]
fn embedded_bom_remains_a_lexical_error() {
    let error = Compiler::new()
        .tokenize("print 1.\u{feff}print 2.")
        .expect_err("only an initial BOM is valid");
    assert!(error.span.is_some());
}
