// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::Compiler;

#[test]
fn prefixed_integers_keep_exact_values_and_spans() {
    let compiler = Compiler::new();
    let source = "let binary be 0b1010_0101. let octal be 0O755. let hex be 0x7fff_FFFF_FFFF_FFFF.";
    let tokens = compiler.tokenize(source).unwrap();
    let numbers: Vec<_> = tokens.iter()
        .filter(|token| token.to_json()["type"] == "NUMBER")
        .collect();
    assert_eq!(numbers.len(), 3);
    assert_eq!(numbers[0].lexeme, "0b1010_0101");
    assert_eq!(numbers[0].to_json()["literal"], 165);
    assert_eq!(numbers[1].to_json()["literal"], 493);
    assert_eq!(numbers[2].to_json()["literal"], i64::MAX);
    assert_eq!(numbers[0].to_json()["column"], 15);

    compiler.analyze(
        "let binary_value be 0B1010. let octal_value be 0o12. let hex_value be 0XFF.          assert binary_value is equal to octal_value. assert hex_value is equal to 255. print hex_value."
    ).unwrap();
    let folded = compiler.optimized_hir("let n be 0x10 plus 0b10. print n.")
        .unwrap().to_json().to_string();
    assert!(folded.contains("18"), "{folded}");
}

#[test]
fn malformed_prefixed_literals_are_lexical_errors() {
    let compiler = Compiler::new();
    for literal in ["0x", "0b", "0o_", "0b102", "0o89", "0xG",
                    "0b1__0", "0x1_", "0x8000_0000_0000_0000"] {
        let source = format!("let value be {literal}.");
        let error = compiler.tokenize(&source).unwrap_err();
        assert_eq!(error.code, "S106", "{source}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn decimal_literals_and_statement_periods_stay_unchanged() {
    let compiler = Compiler::new();
    compiler.analyze(
        "let decimal be 1_000.25e-2. let zero be 0.          assert decimal is equal to 10.0025. assert zero is equal to 0. print zero."
    ).unwrap();
    assert_eq!(
        compiler.format_source("LET flags BE 0XFF.").unwrap(),
        "let flags be 0XFF."
    );
}
