// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use serde_json::json;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-ai-json-schema-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&[
        "run".into(), path.to_string_lossy().into_owned(),
        "--allow".into(), "console_write".into(),
    ]);
    fs::remove_file(path).unwrap();
    result
}

fn program(document: &str, schema: &str) -> String {
    format!("print validate ai json {} against schema {}.",
        serde_json::to_string(document).unwrap(),
        serde_json::to_string(schema).unwrap())
}

fn matches_json(document: &str, schema: &str) -> bool {
    let result = run(&program(document, schema));
    assert_eq!(result.status, 0, "{}", result.stderr);
    match result.stdout.trim() {
        "true" => true,
        "false" => false,
        other => panic!("unexpected boolean result: {other}"),
    }
}

#[test]
fn validates_nested_objects_required_fields_and_extra_properties() {
    let schema = json!({
        "type": "object",
        "required": ["label", "attributes"],
        "additionalProperties": false,
        "properties": {
            "label": {"type": "string", "minLength": 1},
            "attributes": {
                "type": "object",
                "required": ["confidence"],
                "properties": {"confidence": {"type": "number"}},
                "additionalProperties": false
            }
        }
    }).to_string();
    assert!(matches_json(r#"{"label":"gato","attributes":{"confidence":0.9}}"#, &schema));
    for document in [
        r#"{"label":"","attributes":{"confidence":0.9}}"#,
        r#"{"label":"gato","attributes":{}}"#,
        r#"{"label":"gato","attributes":{"confidence":"0.9"}}"#,
        r#"{"label":"gato","attributes":{"confidence":0.9},"unexpected":true}"#,
        r#"{"label":"gato"}"#,
        r#"["gato"]"#,
        "not json",
    ] {
        assert!(!matches_json(document, &schema), "{document}");
    }
}

#[test]
fn array_items_nested_types_and_unicode_limits_are_checked() {
    let schema = json!({
        "type": "array", "minItems": 1, "maxItems": 2,
        "items": {"type": "object", "required": ["value"],
            "properties": {"value": {"type": "string", "minLength": 2, "maxLength": 2}},
            "additionalProperties": false}
    }).to_string();
    assert!(matches_json(r#"[{"value":"ñ🙂"}]"#, &schema));
    assert!(!matches_json(r#"[{"value":"ñ"}]"#, &schema));
    assert!(!matches_json(r#"[{"value":"abc"}]"#, &schema));
    assert!(!matches_json(r#"[{"value":3}]"#, &schema));
    assert!(!matches_json(r#"[]"#, &schema));
    assert!(!matches_json(r#"[{"value":"ñ🙂"},{"value":"ab"},{"value":"cd"}]"#, &schema));
}

#[test]
fn scalar_types_are_checked_without_implicit_coercion() {
    for (kind, document, expected) in [
        ("string", r#""x""#, true),
        ("string", "12", false),
        ("number", "12.5", true),
        ("number", r#""12.5""#, false),
        ("integer", "12", true),
        ("integer", "1.5", false),
        ("boolean", "false", true),
        ("boolean", "0", false),
        ("null", "null", true),
        ("null", "false", false),
    ] {
        let schema = json!({"type":kind}).to_string();
        assert_eq!(matches_json(document, &schema), expected, "{kind} {document}");
    }
}

#[test]
fn unsupported_or_invalid_schema_fails_closed_with_runtime_diagnostic() {
    for schema in [
        "{}",
        "[]",
        "not-json",
        r#"{"type":"unknown"}"#,
        r#"{"type":"object","required":[1]}"#,
        r#"{"type":"object","required":["a","a"]}"#,
        r#"{"type":"string","minLength":-1}"#,
        r#"{"type":"string","minLength":3,"maxLength":1}"#,
        r#"{"type":"array","items":{"type":"string","pattern":"^ok$"}}"#,
        r#"{"type":"object","additionalProperties":"false"}"#,
        r#"{"type":"number","minimum":0}"#,
    ] {
        let result = run(&program("{}", schema));
        assert_eq!(result.status, 70, "{schema}: {}", result.stderr);
        assert!(result.stderr.contains("S503"), "{schema}: {}", result.stderr);
        assert!(!result.stderr.lines().next().unwrap_or("").contains("not-json"),
            "error summary should not echo input content");
    }
}

#[test]
fn depth_limits_are_enforced_and_oversized_responses_are_false() {
    let mut schema = json!({"type":"string"});
    for _ in 0..20 {
        schema = json!({"type":"array","items": schema});
    }
    let result = run(&program("[]", &schema.to_string()));
    assert_eq!(result.status, 70, "{}", result.stderr);
    assert!(result.stderr.contains("nesting or node limits"));

    let oversized = " ".repeat(1024 * 1024 + 1);
    assert!(!matches_json(&oversized, r#"{"type":"object"}"#));
}

#[test]
fn compiler_types_effects_and_formatter_are_stable() {
    let compiler = Compiler::new();
    let source = program(r#"{"label":"cat"}"#,
        r#"{"type":"object","properties":{"label":{"type":"string"}}}"#);
    compiler.hir(&source).unwrap();
    assert_eq!(compiler.effect_analysis(&source).unwrap().effects(),
        vec!["console_write".to_string()]);
    let plan = compiler.runtime_plan(&source).unwrap().to_json().to_string();
    assert!(plan.contains("validate_json_response"), "{plan}");

    let pure = "let checked be validate ai json \"{}\" against schema \"{\\\"type\\\":\\\"object\\\"}\".";
    compiler.hir(pure).unwrap();
    assert!(compiler.effect_analysis(pure).unwrap().effects().is_empty());
    let formatted = compiler.format_source(&source).unwrap();
    assert_eq!(compiler.format_source(&formatted).unwrap(), formatted);

    for invalid in [
        "let checked be validate ai json 3 against schema \"{}\".",
        "let checked be validate ai json \"{}\" against schema 42.",
    ] {
        assert!(compiler.hir(invalid).is_err(), "{invalid}");
    }
}

#[test]
fn checked_in_example_runs_without_model_or_network_permissions() {
    let source = include_str!("../examples/ai_structured_json.suri");
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "false\n{\"confidence\":0.91,\"label\":\"cat\"}\nStructured AI response validated.\n");
}

#[test]
fn enumerated_labels_reject_unknown_values_and_malformed_enums() {
    let schema = json!({"type":"string","enum":["cat","dog","ñ🙂"]}).to_string();
    assert!(matches_json(r#""cat""#, &schema));
    assert!(matches_json(r#""ñ🙂""#, &schema));
    assert!(!matches_json(r#""bird""#, &schema));
    assert!(!matches_json("1", &schema));
    for invalid in [
        r#"{"type":"string","enum":[]}"#,
        r#"{"type":"string","enum":["a","a"]}"#,
        r#"{"type":"string","enum":["a",1]}"#,
        r#"{"type":"number","enum":[1]}"#,
    ] {
        let result = run(&program(r#""a""#, invalid));
        assert_eq!(result.status, 70, "{invalid}: {}", result.stderr);
        assert!(result.stderr.contains("S503"));
    }
}

#[test]
fn normalized_json_is_canonical_and_fails_for_mismatched_responses() {
    let schema = r#"{"type":"object","required":["a","b"],"properties":{"a":{"type":"integer"},"b":{"type":"integer"}},"additionalProperties":false}"#;
    let source = format!("print normalize ai json {} against schema {}.",
        serde_json::to_string(r#"{"b":2, "a":1}"#).unwrap(),
        serde_json::to_string(schema).unwrap());
    let result = run(&source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "{\"a\":1,\"b\":2}\n");
    let compiler = Compiler::new();
    assert!(compiler.effect_analysis(
        "let clean be normalize ai json \"{}\" against schema \"{\\\"type\\\":\\\"object\\\"}\"."
    ).unwrap().effects().is_empty());
    let plan = compiler.runtime_plan(&source).unwrap().to_json().to_string();
    assert!(plan.contains("normalize_json_response"), "{plan}");

    let invalid = format!("print normalize ai json {} against schema {}.",
        serde_json::to_string(r#"{"a":"wrong","b":2}"#).unwrap(),
        serde_json::to_string(schema).unwrap());
    let error = run(&invalid);
    assert_eq!(error.status, 70, "{}", error.stderr);
    assert!(error.stderr.contains("AI JSON response does not match"));
}

#[test]
fn duplicate_json_keys_are_rejected_in_responses_and_contracts() {
    let contract = r#"{"type":"object","required":["label"],"properties":{"label":{"type":"string"}}}"#;
    assert!(!matches_json(r#"{"label":"cat","label":"dog"}"#, contract));
    assert!(!matches_json(r#"{"label":"cat","nested":{"x":1,"x":2}}"#, contract));
    let duplicate_schema = r#"{"type":"object","type":"string"}"#;
    let invalid = run(&program("{}", duplicate_schema));
    assert_eq!(invalid.status, 70, "{}", invalid.stderr);
    assert!(invalid.stderr.contains("AI JSON schema is not valid JSON"));
}

#[test]
fn nominal_ai_response_flows_safely_into_text_validation() {
    let source = r#"let model be connect language model "local/test".
let generated be prompt model with "{\"label\":\"cat\"}".
let checked be validate ai json generated against schema "{\"type\":\"object\"}".
print checked."#;
    let compiler = Compiler::new();
    compiler.hir(source).unwrap();
    let required = compiler.effect_analysis(source).unwrap().effects();
    for effect in ["model_load", "model_inference", "network", "console_write"] {
        assert!(required.contains(&effect.to_string()), "{required:?}");
    }

    let path = std::env::temp_dir().join(format!(
        "semauri-ai-response-text-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&[
        "run".into(), path.to_string_lossy().into_owned(),
        "--allow".into(), "console_write".into(),
        "--allow".into(), "model_load".into(),
        "--allow".into(), "model_inference".into(),
        "--allow".into(), "network".into(),
    ]);
    fs::remove_file(path).unwrap();
    assert_eq!(result.status, 0, "{}", result.stderr);
    // The local backend is a simulator: it adds a prefix, so it does not
    // claim to have generated valid JSON even when the prompt contains JSON.
    assert_eq!(result.stdout, "false\n");

    // Only ai.response is safely readable as text. Model identities stay nominal.
    assert!(compiler.hir(
        "let model be connect language model \"local/test\". let checked be validate ai json model against schema \"{}\"."
    ).is_err());
}
