// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use serde_json::Value;

const PACKAGE: &str = include_str!("../editors/vscode/package.json");
const CONFIGURATION: &str = include_str!("../editors/vscode/language-configuration.json");
const GRAMMAR: &str = include_str!("../editors/vscode/syntaxes/semauri.tmLanguage.json");

#[test]
fn vscode_metadata_registers_suri_with_the_stable_textmate_scope() {
    let package: Value = serde_json::from_str(PACKAGE).unwrap();
    assert_eq!(package["contributes"]["languages"][0]["id"], "semauri");
    assert_eq!(package["contributes"]["languages"][0]["extensions"][0], ".suri");
    assert_eq!(package["contributes"]["grammars"][0]["scopeName"], "source.semauri");
    serde_json::from_str::<Value>(CONFIGURATION).unwrap();
}

#[test]
fn grammar_is_valid_json_and_covers_core_config_and_ai_syntax() {
    let grammar: Value = serde_json::from_str(GRAMMAR).unwrap();
    assert_eq!(grammar["scopeName"], "source.semauri");
    assert_eq!(grammar["fileTypes"][0], "suri");
    let serialized = grammar.to_string();
    for expected in [
        "comment.line.number-sign.semauri",
        "string.quoted.double.semauri",
        "keyword.declaration.config.semauri",
        "support.function.ai.semauri",
        "keyword.control.semauri",
    ] {
        assert!(serialized.contains(expected), "missing {expected}");
    }
}
