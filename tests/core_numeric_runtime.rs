// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str, config: &str, allowed: bool) -> semauri::CliResult {
    let directory = std::env::temp_dir().join(format!(
        "semauri-runtime-numeric-{}-{}", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
    ));
    fs::create_dir(&directory).unwrap();
    let path = directory.join("program.suri");
    fs::write(&path, source).unwrap();
    fs::write(directory.join(".config.suri"), config).unwrap();
    let mut args = vec!["run".into(), path.to_string_lossy().into_owned()];
    if allowed {
        args.extend([
            "--allow".into(), "config_read".into(),
            "--allow".into(), "console_write".into(),
        ]);
    }
    let result = run_cli(&args);
    fs::remove_dir_all(&directory).unwrap();
    result
}

#[test]
fn runtime_numeric_ordering_keeps_exact_int64_precision() {
    let source = r#"
let upper be number config "upper".
let lower be number config "lower".
let delta be upper minus lower.
let inverted be minus delta.
let higher_result be upper is greater than lower.
let smaller be lower is less than upper.
let inclusive be upper is at least lower.
let below_equal be lower is at most upper.
print delta.
print inverted.
print higher_result.
print smaller.
print inclusive.
print below_equal.
assert upper is greater than lower.
"#;
    let config = "parameter upper = 9007199254740993.\nparameter lower = 9007199254740992.";
    let output = run(source, config, true);
    assert_eq!(output.status, 0, "{}", output.stderr);
    assert_eq!(output.stdout, "1\n-1\ntrue\ntrue\ntrue\ntrue\n");

    let compiler = Compiler::new();
    compiler.hir(source).unwrap();
    let plan = compiler.runtime_plan(source).unwrap();
    let json = plan.to_json();
    let nodes = json["operations"].as_array().unwrap();
    for name in [
        "subtract", "negate", "greater_than", "less_than",
        "greater_than_or_equal", "less_than_or_equal",
    ] {
        assert!(nodes.iter().any(|node| node["domain"] == "core"
            && node["name"] == name), "{name}: {json}");
    }
    for node in nodes.iter().filter(|node| node["domain"] == "core") {
        assert_eq!(node["effects"], serde_json::json!([]));
    }
    assert!(plan.dataflow_graph().is_ok());
}

#[test]
fn runtime_subtraction_reports_exact_overflow() {
    let source = r#"
let upper be number config "upper".
let lower be number config "lower".
let result be upper minus lower.
print result.
"#;
    let config = "parameter upper = 9223372036854775807.\nparameter lower = -1.";
    let result = run(source, config, true);
    assert_eq!(result.status, 70, "{}", result.stderr);
    assert!(result.stderr.contains("S470"), "{}", result.stderr);
    assert!(result.stdout.is_empty());
}

#[test]
fn runtime_negation_reports_i64_min_overflow() {
    let source = r#"
let lower be number config "lower".
let opposite be minus lower.
print opposite.
"#;
    let result = run(source, "parameter lower = -9223372036854775808.", true);
    assert_eq!(result.status, 70, "{}", result.stderr);
    assert!(result.stderr.contains("S358"), "{}", result.stderr);
    assert!(result.stdout.is_empty());
}

#[test]
fn static_types_and_runtime_effect_gates_are_preserved() {
    let compiler = Compiler::new();
    for source in [
        r#"let flag be true. print flag minus 1."#,
        r#"assert "a" is greater than 2."#,
        r#"print minus "three"."#,
    ] {
        assert!(compiler.hir(source).is_err(), "{source}");
    }

    let source = r#"
let upper be number config "upper".
let result be minus upper.
print result.
"#;
    let denied = run(source, "parameter upper = 42.", false);
    assert_eq!(denied.status, 65, "{}", denied.stderr);
    assert!(denied.stderr.contains("S334"), "{}", denied.stderr);
    let allowed = run(source, "parameter upper = 42.", true);
    assert_eq!(allowed.status, 0, "{}", allowed.stderr);
    assert_eq!(allowed.stdout, "-42\n");
    let effects = compiler.effect_analysis(source).unwrap().effects();
    assert!(effects.contains(&"config_read".to_string()));
    assert!(effects.contains(&"console_write".to_string()));
}

#[test]
fn optimizer_still_constant_folds_static_arithmetic() {
    let compiler = Compiler::new();
    let source = "let difference be 10 minus 3. let negative be minus difference. assert negative is less than 0.";
    compiler.hir(source).unwrap();
    let result = run(source, "", false);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert!(compiler.effect_analysis(source).unwrap().pure());
    assert_eq!(compiler.format_source(&compiler.format_source(source).unwrap()).unwrap(),
        compiler.format_source(source).unwrap());
}
