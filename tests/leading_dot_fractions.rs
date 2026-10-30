// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::Compiler;

#[test]
fn leading_dot_literals_are_finite_decimal_numbers() {
    let compiler = Compiler::new();
    let source = "let fraction_value be .5. let exponent_value be .25e+2. let tiny_value be .0_1. assert fraction_value is equal to 0.5. assert exponent_value is equal to 25. assert tiny_value is equal to 0.01. print exponent_value.";
    compiler.analyze(source).unwrap();
    let values: Vec<_> = compiler.tokenize(source).unwrap().into_iter()
        .filter(|token| token.to_json()["type"] == "NUMBER").collect();
    assert_eq!(values[0].lexeme, ".5");
    assert_eq!(values[0].to_json()["literal"], 0.5);
    assert_eq!(values[1].to_json()["literal"], 25.0);
    assert_eq!(values[2].to_json()["literal"], 0.01);
}

#[test]
fn leading_dots_do_not_change_statement_termination_or_formatter() {
    let compiler = Compiler::new();
    let source = "LET fraction_value BE .75.\nPRINT fraction_value.\n";
    let formatted = "let fraction_value be .75.\nprint fraction_value.\n";
    assert_eq!(compiler.format_source(source).unwrap(), formatted);
    assert_eq!(compiler.format_source(formatted).unwrap(), formatted);
    compiler.analyze(formatted).unwrap();
    compiler.analyze("let integer_value be 2. let decimal_value be 2.5. print integer_value plus decimal_value.").unwrap();
}

#[test]
fn invalid_leading_dot_exponents_retain_located_s106_errors() {
    let compiler = Compiler::new();
    for bad in [".5e", ".5e+", ".1__2", ".1_", ".1e9999"] {
        let source = format!("let fraction_value be {bad}. print fraction_value.");
        let error = compiler.tokenize(&source).unwrap_err();
        assert_eq!(error.code, "S106", "{bad}: {error}");
        assert!(error.span.is_some());
    }
}
