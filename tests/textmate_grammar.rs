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

#[test]
fn grammar_highlights_radix_integers_and_both_string_styles() {
    let grammar: Value = serde_json::from_str(GRAMMAR).unwrap();
    let strings = &grammar["repository"]["strings"]["patterns"];
    assert_eq!(strings[0]["name"], "string.quoted.double.semauri");
    assert_eq!(strings[1]["name"], "string.quoted.single.semauri");
    assert_eq!(strings[0]["begin"], "\"");
    assert_eq!(strings[1]["begin"], "'");
    for quoted in strings.as_array().unwrap().iter().take(2) {
        let escapes = quoted["patterns"][0]["match"].as_str().unwrap();
        assert!(escapes.contains("nrt"), "{escapes}");
        assert!(escapes.contains("u"), "{escapes}");
    }
    let number = grammar["repository"]["numbers"]["match"].as_str().unwrap();
    for prefix in ["0[bB]", "0[oO]", "0[xX]", "[0-9A-Fa-f]"] {
        assert!(number.contains(prefix), "missing {prefix}: {number}");
    }
    assert!(number.contains("[eE]"), "decimal exponents must remain supported");
}

#[test]
fn raw_string_scopes_and_control_escapes_are_documented_in_grammar() {
    let grammar: Value = serde_json::from_str(GRAMMAR).unwrap();
    let strings = grammar["repository"]["strings"]["patterns"].as_array().unwrap();
    assert_eq!(strings.len(), 4);
    assert_eq!(strings[2]["name"], "string.quoted.double.raw.semauri");
    assert_eq!(strings[3]["name"], "string.quoted.single.raw.semauri");
    assert_eq!(strings[2]["begin"], "[rR]\"");
    assert_eq!(strings[3]["begin"], "[rR]'");
    assert!(strings[2].get("patterns").is_none(), "raw strings must not decode escapes");
    assert!(strings[3].get("patterns").is_none(), "raw strings must not decode escapes");
    for quoted in &strings[..2] {
        let escapes = quoted["patterns"][0]["match"].as_str().unwrap();
        assert!(escapes.contains("nrt0bf"), "{escapes}");
    }
}
