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
    assert_eq!(strings[2]["name"], "string.quoted.double.semauri");
    assert_eq!(strings[3]["name"], "string.quoted.single.semauri");
    assert_eq!(strings[2]["begin"], "\"");
    assert_eq!(strings[3]["begin"], "'");
    for quoted in strings.as_array().unwrap().iter().skip(2).take(2) {
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
    assert!(strings.len() >= 4);
    assert_eq!(strings[4]["name"], "string.quoted.double.raw.semauri");
    assert_eq!(strings[5]["name"], "string.quoted.single.raw.semauri");
    assert_eq!(strings[4]["begin"], "[rR]\"");
    assert_eq!(strings[5]["begin"], "[rR]'");
    assert!(strings[4].get("patterns").is_none(), "raw strings must not decode escapes");
    assert!(strings[5].get("patterns").is_none(), "raw strings must not decode escapes");
    for quoted in &strings[2..4] {
        let escapes = quoted["patterns"][0]["match"].as_str().unwrap();
        assert!(escapes.contains("nrt0bfav"), "{escapes}");
    }
}

#[test]
fn grammar_highlights_base36_hex_escapes_and_hash_raw_literals() {
    let grammar: Value = serde_json::from_str(GRAMMAR).unwrap();
    let numeric = grammar["repository"]["numbers"]["match"].as_str().unwrap();
    assert!(numeric.contains("0[zZ][0-9A-Za-z]"), "{numeric}");
    let strings = grammar["repository"]["strings"]["patterns"].as_array().unwrap();
    assert_eq!(strings.len(), 8);
    for ordinary in &strings[2..4] {
        let escape = ordinary["patterns"][0]["match"].as_str().unwrap();
        assert!(escape.contains("x[0-9A-Fa-f]{2}"), "{escape}");
        assert!(escape.contains("u"), "{escape}");
    }
    let double = &strings[6];
    let single = &strings[7];
    assert_eq!(double["name"], "string.quoted.double.raw.hash.semauri");
    assert_eq!(single["name"], "string.quoted.single.raw.hash.semauri");
    assert_eq!(double["begin"], r#"[rR](#+)""#);
    assert_eq!(single["begin"], "[rR](#+)'");
    assert_eq!(double["end"], "\"\\1");
    assert_eq!(single["end"], "'\\1");
    assert!(double.get("patterns").is_none());
    assert!(single.get("patterns").is_none());
}

#[test]
fn grammar_highlights_scientific_math_operators() {
    let grammar: Value = serde_json::from_str(GRAMMAR).unwrap();
    let keywords = grammar["repository"]["collections"]["match"].as_str().unwrap();
    for name in ["exp", "ln", "log2", "log10", "sin", "cos", "asin", "acos", "atan", "sinh", "cosh", "tanh"] {
        assert!(keywords.contains(name), "missing {name} in {keywords}");
    }
}

#[test]
fn grammar_recognizes_leading_dot_fractions_triple_strings_and_unicode_escapes() {
    let grammar: Value = serde_json::from_str(GRAMMAR).unwrap();
    let numeric = grammar["repository"]["numbers"]["match"].as_str().unwrap();
    assert!(numeric.contains(r"\.[0-9]"), "{numeric}");
    let strings = grammar["repository"]["strings"]["patterns"].as_array().unwrap();
    assert_eq!(strings.len(), 8);
    assert_eq!(strings[0]["name"], "string.quoted.triple.double.semauri");
    assert_eq!(strings[1]["name"], "string.quoted.triple.single.semauri");
    assert_eq!(strings[0]["begin"], "\"\"\"");
    assert_eq!(strings[1]["begin"], "'''");
    assert_eq!(strings[0]["end"], "\"\"\"");
    assert_eq!(strings[1]["end"], "'''");
    for block in &strings[..2] {
        assert!(block.get("patterns").is_none(), "multiline literals preserve backslashes");
    }
    for ordinary in &strings[2..4] {
        let escape = ordinary["patterns"][0]["match"].as_str().unwrap();
        assert!(escape.contains("U[0-9A-Fa-f]{8}"), "{escape}");
        assert!(escape.contains("u"), "{escape}");
    }
}

#[test]
fn grammar_highlights_natural_conditional_and_join_syntax() {
    let grammar: Value = serde_json::from_str(GRAMMAR).unwrap();
    let control = grammar["repository"]["control"]["match"].as_str().unwrap();
    for word in ["when", "each"] { assert!(control.contains(word), "{control}"); }
    let operators = grammar["repository"]["operators"]["match"].as_str().unwrap();
    for word in ["joined", "different"] { assert!(operators.contains(word), "{operators}"); }
}

#[test]
fn grammar_highlights_python_like_conditionals_and_membership() {
    let grammar: Value = serde_json::from_str(GRAMMAR).unwrap();
    let control = grammar["repository"]["control"]["match"].as_str().unwrap();
    for word in ["elif", "then", "do", "in"] {
        assert!(control.contains(word), "missing {word} in {control}");
    }
}

#[test]
fn grammar_highlights_new_pure_math_functions() {
    let grammar: Value = serde_json::from_str(GRAMMAR).unwrap();
    let keywords = grammar["repository"]["collections"]["match"].as_str().unwrap();
    for name in ["asinh", "acosh", "atanh", "log1p", "expm1"] {
        assert!(keywords.contains(name), "{name} should be highlighted");
    }
}

#[test]
fn grammar_highlights_python_friendly_function_aliases() {
    let grammar: Value = serde_json::from_str(GRAMMAR).unwrap();
    let words = grammar["repository"]["collections"]["match"].as_str().unwrap();
    for word in ["len", "abs", "mean", "min", "max", "strip", "upper", "lower"] {
        assert!(words.contains(word), "{word} missing from {words}");
    }
}

#[test]
fn grammar_highlights_natural_empty_and_blank_predicates() {
    let grammar: Value = serde_json::from_str(GRAMMAR).unwrap();
    let operators = grammar["repository"]["operators"]["match"].as_str().unwrap();
    for word in ["empty", "blank"] {
        assert!(operators.contains(word), "{word} missing in {operators}");
    }
}

#[test]
fn grammar_highlights_natural_numeric_predicates() {
    let grammar: Value = serde_json::from_str(GRAMMAR).unwrap();
    let operators = grammar["repository"]["operators"]["match"].as_str().unwrap();
    for word in ["zero", "positive", "negative", "even", "odd"] {
        assert!(operators.contains(word), "missing {word} in {operators}");
    }
}

#[test]
fn grammar_highlights_natural_comparison_bounds() {
    let grammar: Value = serde_json::from_str(GRAMMAR).unwrap();
    let operators = grammar["repository"]["operators"]["match"].as_str().unwrap();
    for word in ["least", "most", "above", "below"] {
        assert!(operators.contains(word), "missing {word} in {operators}");
    }
}

#[test]
fn grammar_highlights_natural_membership_alias() {
    let grammar: Value = serde_json::from_str(GRAMMAR).unwrap();
    let operators = grammar["repository"]["operators"]["match"].as_str().unwrap();
    assert!(operators.contains("one"), "{operators}");
}

#[test]
fn grammar_highlights_divisibility_predicates() {
    let grammar: Value = serde_json::from_str(GRAMMAR).unwrap();
    let operators = grammar["repository"]["operators"]["match"].as_str().unwrap();
    for word in ["divisible", "multiple"] {
        assert!(operators.contains(word), "{word} missing in {operators}");
    }
}

#[test]
fn grammar_highlights_english_text_inclusion_and_negation() {
    let grammar: Value = serde_json::from_str(GRAMMAR).unwrap();
    let words = grammar["repository"]["operators"]["match"].as_str().unwrap();
    for item in ["does", "start", "contain", "include", "includes", "excludes"] {
        assert!(words.contains(item), "{item} missing from {words}");
    }
}

#[test]
fn grammar_highlights_integer_radix_functions() {
    let grammar: Value = serde_json::from_str(GRAMMAR).unwrap();
    let names = grammar["repository"]["collections"]["match"].as_str().unwrap();
    for name in ["binary", "octal", "hex", "hexadecimal"] {
        assert!(names.contains(name), "{name} missing from {names}");
    }
}

#[test]
fn grammar_supports_prime_predicates_and_short_function_aliases() {
    let grammar: Value = serde_json::from_str(GRAMMAR).unwrap();
    let ops = grammar["repository"]["operators"]["match"].as_str().unwrap();
    assert!(ops.contains("prime"));
    let functions = grammar["repository"]["collections"]["match"].as_str().unwrap();
    for name in ["avg", "var", "stddev", "total", "size", "unique", "head", "final"] {
        assert!(functions.contains(name), "{name} missing from {functions}");
    }
}

#[test]
fn grammar_highlights_four_digit_unicode_and_escape_character() {
    let grammar: Value = serde_json::from_str(GRAMMAR).unwrap();
    let strings = grammar["repository"]["strings"]["patterns"].as_array().unwrap();
    for quoted in &strings[2..4] {
        let escape = quoted["patterns"][0]["match"].as_str().unwrap();
        assert!(escape.contains(r"u(?:[0-9A-Fa-f]{4}|"), "{escape}");
        assert!(escape.contains("nrt0bfave"), "{escape}");
    }
}
