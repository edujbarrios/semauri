// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::Compiler;

#[test]
fn base36_literals_preserve_values_and_original_lexemes() {
    let compiler = Compiler::new();
    let source = "let serial_value be 0z10. let letter_value be 0ZZ. let grouped_value be 0z1_z.";
    let values: Vec<_> = compiler.tokenize(source).unwrap().into_iter()
        .filter(|token| token.to_json()["type"] == "NUMBER")
        .collect();
    assert_eq!(values.len(), 3);
    assert_eq!(values[0].lexeme, "0z10");
    assert_eq!(values[0].to_json()["literal"], 36);
    assert_eq!(values[1].to_json()["literal"], 35);
    assert_eq!(values[2].to_json()["literal"], 71);
    compiler.analyze("let serial_value be 0z10. assert serial_value is equal to 36. print serial_value.").unwrap();
    assert_eq!(compiler.format_source("LET serial_value BE 0Z1_Z.\n").unwrap(),
               "let serial_value be 0Z1_Z.\n");
}

#[test]
fn base36_literals_are_checked_signed_64_bit_integers() {
    let compiler = Compiler::new();
    let source = "let biggest be 0z1y2p0ij32e8e7. print biggest.";
    let value = compiler.tokenize(source).unwrap().into_iter()
        .find(|token| token.to_json()["type"] == "NUMBER").unwrap();
    assert_eq!(value.to_json()["literal"], i64::MAX);
    for invalid in ["0z", "0Z_1", "0z1_", "0z1__2", "0z1y2p0ij32e8e8", "0zZZZZZZZZZZZZZZZZ"] {
        let source = format!("let value be {invalid}.");
        let error = compiler.tokenize(&source).unwrap_err();
        assert_eq!(error.code, "S106", "{invalid}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn legacy_radix_and_decimal_literals_remain_supported() {
    let compiler = Compiler::new();
    compiler.analyze("let binary_value be 0b1010. let hex_value be 0xA. let octal_value be 0o12. let decimal_value be 10. assert binary_value is equal to decimal_value. assert octal_value is equal to hex_value. print decimal_value.").unwrap();
}
