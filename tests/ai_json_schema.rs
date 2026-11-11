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
        assert!(!result.stderr.contains("not-json"), "diagnostics should not echo inputs");
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
    assert_eq!(result.stdout, "Structured AI response validated.\n");
}
