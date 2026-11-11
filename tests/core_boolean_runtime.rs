// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn fixture(source: &str, config: Option<&str>, allow_effects: bool) -> semauri::CliResult {
    let dir = std::env::temp_dir().join(format!(
        "semauri-core-boolean-{}-{}", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&dir).unwrap();
    let file = dir.join("program.suri");
    fs::write(&file, source).unwrap();
    if let Some(config) = config {
        fs::write(dir.join(".config.suri"), config).unwrap();
    }
    let mut args = vec!["run".into(), file.to_string_lossy().into_owned()];
    if allow_effects {
        args.extend([
            "--allow".into(), "config_read".into(),
            "--allow".into(), "console_write".into(),
        ]);
    }
    let result = run_cli(&args);
    fs::remove_dir_all(dir).unwrap();
    result
}

#[test]
fn xor_is_typed_left_associative_and_has_distinct_precedence() {
    let source = r#"
assert true xor false.
assert false xor true.
assert not (true xor true).
assert not (false xor false).
assert (true xor true xor true).
assert (true xor true and false).
assert (true or true xor true).
assert not (false and true xor false).
"#;
    let result = fixture(source, None, false);
    assert_eq!(result.status, 0, "{}", result.stderr);

    let example = include_str!("../examples/boolean_logic.suri");
    let result = fixture(example, None, true);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Boolean expressions verified.\n");
}

#[test]
fn xor_rejects_non_boolean_operands_at_semantic_analysis() {
    let compiler = Compiler::new();
    for source in [
        "assert 1 xor true.",
        "assert true xor 3.",
        r#"assert "true" xor false."#,
        "let wrong be false xor 1.",
    ] {
        let err = compiler.hir(source).unwrap_err();
        assert_eq!(err.code, "S319", "{source}: {err}");
    }
}

#[test]
fn xor_remains_a_contextual_identifier_and_formatter_is_idempotent() {
    let compiler = Compiler::new();
    let input = "LET XOR BE true.\nASSERT XOR XOR false.\nASSERT true XOR false.";
    let expected = "let XOR be true.\nassert XOR xor false.\nassert true xor false.";
    let formatted = compiler.format_source(input).unwrap();
    assert_eq!(formatted, expected);
    assert_eq!(compiler.format_source(&formatted).unwrap(), expected);
    assert!(compiler.hir("let XOR be true. assert XOR xor false.").is_ok());
}

#[test]
fn optimizer_folds_boolean_xor_and_preserves_effect_analysis() {
    let compiler = Compiler::new();
    let source = "let truth be true xor false. assert truth.";
    compiler.hir(source).unwrap();
    let optimized = compiler.optimized_hir(source).unwrap().to_json().to_string();
    assert!(!optimized.contains("\"operator\":\"xor\""), "{optimized}");
    assert!(compiler.effect_analysis(source).unwrap().effects().is_empty());
}

#[test]
fn not_xor_and_equality_work_on_values_only_known_at_runtime() {
    let source = r#"
let gate be boolean config "gate".
let other be boolean config "other".
let inverse be not gate.
let exclusive be gate xor other.
let equal be gate is equal to other.
let different be gate is not equal to other.
let nested be not (gate xor other).
print inverse.
print exclusive.
print equal.
print different.
print nested.
"#;
    let config = "parameter gate = true.\nparameter other = false.\n";
    let result = fixture(source, Some(config), true);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "false\ntrue\nfalse\ntrue\nfalse\n");

    let plan = Compiler::new().runtime_plan(source).unwrap();
    let operations = plan.to_json()["operations"].as_array().unwrap().clone();
    assert_eq!(operations.len(), 12);
    let core = operations.iter().filter(|operation| operation["domain"] == "core")
        .collect::<Vec<_>>();
    assert_eq!(core.len(), 5);
    for node in core {
        assert_eq!(node["return_type"], "boolean");
        assert_eq!(node["effects"], serde_json::json!([]));
    }
    assert!(operations.iter().any(|node| node["domain"] == "core" && node["name"] == "not"));
    assert!(operations.iter().any(|node| node["domain"] == "core" && node["name"] == "xor"));
    assert!(operations.iter().any(|node| node["domain"] == "core" && node["name"] == "equal"));
    assert!(operations.iter().any(|node| node["domain"] == "core" && node["name"] == "not_equal"));
}

#[test]
fn runtime_negation_can_feed_assert_without_extra_permission() {
    let source = "let gate be boolean config \"gate\". assert not gate.";
    let result = fixture(source, Some("parameter gate = false.\n"), true);
    assert_eq!(result.status, 0, "{}", result.stderr);
    let failing = fixture(source, Some("parameter gate = true.\n"), true);
    assert_eq!(failing.status, 70, "{}", failing.stderr);
    assert!(failing.stderr.contains("S502"), "{}", failing.stderr);
}

#[test]
fn effect_gates_remain_mandatory_for_runtime_configuration() {
    let source = "let gate be boolean config \"gate\". let inverted be not gate.";
    let denied = fixture(source, Some("parameter gate = true.\n"), false);
    assert_eq!(denied.status, 65);
    assert!(denied.stderr.contains("S334"), "{}", denied.stderr);
}
