// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-difference-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&[
        "run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()
    ]);
    fs::remove_file(&path).unwrap();
    result
}

#[test]
fn examples_and_all_three_forms_execute() {
    let result = run(include_str!("../examples/numeric_differences.suri"));
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Numeric differences OK\n");
}

#[test]
fn signed_integer_precision_and_finite_decimal_arithmetic() {
    let source = r#"assert (difference between 9007199254740995 and 2) is equal to 9007199254740993.
assert (difference between 2 and 9007199254740995) is equal to (minus 9007199254740993).
assert (absolute difference between 2 and 9007199254740995) is equal to 9007199254740993.
assert (squared difference between 9007199254740995 and 9007199254740993) is equal to 4.
assert (difference between 2.75 and 1.5) is equal to 1.25.
assert (absolute difference between 1.5 and 2.75) is equal to 1.25.
assert (squared difference between 2.5 and 1.25) is equal to 1.5625.
assert (difference between (2 plus 3) and (1 plus 1)) is equal to 3."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn invalid_operands_and_missing_delimiter_have_diagnostics() {
    let compiler = Compiler::new();
    for source in [
        "print difference between true and 1.",
        "print absolute difference between 1 and \"x\".",
        "print squared difference between \"x\" and 2.",
    ] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S314", "{source}: {error}");
        assert!(error.span.is_some());
    }
    let error = compiler.parse("print difference between 5 2.").unwrap_err();
    assert_eq!(error.code, "S252", "{error}");
    assert!(error.span.is_some());
}

#[test]
fn checked_integer_overflow_is_not_silently_rounded() {
    for (source, code) in [
        ("print absolute difference between (minus 9223372036854775807) and 1.", "S388"),
        ("print squared difference between 3037000500 and 0.", "S422"),
    ] {
        let result = run(source);
        assert_eq!(result.status, 65, "{source}: {}", result.stderr);
        assert!(result.stderr.contains(code), "{source}: {}", result.stderr);
    }
}

#[test]
fn formatter_optimizer_effects_and_contextual_names() {
    let c = Compiler::new();
    let raw = "PRINT ABSOLUTE DIFFERENCE BETWEEN 1 AND 4.\nPRINT SQUARED DIFFERENCE BETWEEN 2 AND 4.\nPRINT DIFFERENCE BETWEEN 8 AND 3.";
    let formatted = "print absolute difference between 1 and 4.\nprint squared difference between 2 and 4.\nprint difference between 8 and 3.";
    assert_eq!(c.format_source(raw).unwrap(), formatted);
    assert_eq!(c.format_source(formatted).unwrap(), formatted);
    assert_eq!(c.runtime_plan(raw).unwrap().to_json(),
        c.runtime_plan(formatted).unwrap().to_json());
    let source = r#"let difference be 7.
let squared be 8.
let absolute be 9.
let between be 10.
assert difference is equal to 7.
assert squared is equal to 8.
assert absolute is equal to 9.
assert between is equal to 10."#;
    assert_eq!(run(source).status, 0);
    assert_eq!(c.format_source("LET Difference BE 7. PRINT Difference.").unwrap(),
        "let Difference be 7. print Difference.");
    let ast = c.parse("print squared difference between 5 and 2.")
        .unwrap().to_json().to_string();
    assert!(ast.contains("square") && ast.contains("subtract"), "{ast}");
    assert!(c.optimized_hir("print absolute difference between 5 and 2.").is_ok());
    assert_eq!(c.effect_analysis("print squared difference between 5 and 2.")
        .unwrap().effects(), vec!["console_write".to_string()]);
}
