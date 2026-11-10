// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-absolute-error-stats-{}-{}.suri",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&[
        "run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned(),
    ]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn checked_in_example_and_paired_metrics_run() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/absolute_error_statistics.suri");
    let checked = run_cli(&["check".into(), path.into()]);
    assert_eq!(checked.status, 0, "{}", checked.stderr);
    let result = run(include_str!("../examples/absolute_error_statistics.suri"));
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Absolute error statistics OK\n");
}

#[test]
fn exact_large_integers_decimals_and_empty_sum() {
    let result = run(r#"let vals be a list of 9007199254740993, (minus 2).
assert (sum of magnitudes of vals) is equal to 9007199254740995.
assert (max magnitude of vals) is equal to 9007199254740993.
assert (sum of magnitudes of (a list of minus 3, 4)) is equal to 7.
assert (mean magnitude of (a list of minus 3, 5)) is equal to 4.
assert (mean absolute error of (a list of minus 3, 5)) is equal to 4.
assert (mae of (a list of minus 3, 5)) is equal to 4.
assert (mae of (a list of minus 2.5, 1.5)) is equal to 2.
assert (sum of magnitudes of (drop 1 from (a list of 9))) is equal to 0."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn types_empty_inputs_and_overflow_are_checked() {
    let compiler = Compiler::new();
    for source in [
        "print sum of magnitudes of 3.",
        "print mean magnitude of true.",
        "print mae of 1.",
        "print max magnitude of 8.",
        "print mean absolute error of false.",
    ] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S365", "{source}: {error}");
        assert!(error.span.is_some());
    }
    for source in [
        "print mean magnitude of (a list of true).",
        "print max magnitude of (a list of \"wrong\").",
    ] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S386", "{source}: {error}");
        assert!(error.span.is_some());
    }
    for (source, code) in [
        ("print mae of (drop 1 from (a list of 1)).", "S354"),
        ("print max magnitude of (drop 1 from (a list of 1)).", "S355"),
        ("print mean absolute error of (a list of ((minus 9223372036854775807) minus 1)).", "S388"),
        ("print sum of magnitudes of (a list of 9223372036854775807, 1).", "S354"),
    ] {
        let result = run(source);
        assert_eq!(result.status, 65, "{source}: {}", result.stderr);
        assert!(result.stderr.contains(code), "{source}: {}", result.stderr);
    }
}

#[test]
fn names_remain_contextual_and_formatter_is_idempotent() {
    let compiler = Compiler::new();
    let raw = "PRINT SUM OF MAGNITUDES OF (A LIST OF 2, 3).\nPRINT MEAN MAGNITUDE OF (A LIST OF 2, 3).\nPRINT MEAN ABSOLUTE ERROR OF (A LIST OF 2, 3).\nPRINT MAE OF (A LIST OF 2, 3).\nPRINT MAX MAGNITUDE OF (A LIST OF 2, 3).";
    let formatted = "print sum of magnitudes of (a list of 2, 3).\nprint mean magnitude of (a list of 2, 3).\nprint mean absolute error of (a list of 2, 3).\nprint mae of (a list of 2, 3).\nprint max magnitude of (a list of 2, 3).";
    assert_eq!(compiler.format_source(raw).unwrap(), formatted);
    assert_eq!(compiler.format_source(formatted).unwrap(), formatted);
    let source = r#"let mae be 1.
let magnitude be 2.
let magnitudes be 3.
let error be 4.
assert mae is equal to 1.
assert magnitude is equal to 2.
assert magnitudes is equal to 3.
assert error is equal to 4."#;
    assert_eq!(run(source).status, 0);
    assert_eq!(compiler.format_source("LET Magnitude BE 2. PRINT Magnitude.").unwrap(),
        "let Magnitude be 2. print Magnitude.");
}

#[test]
fn lowered_expressions_preserve_optimizer_and_effects() {
    let compiler = Compiler::new();
    for (source, op) in [
        ("print sum of magnitudes of (a list of 1, 2).", "sum"),
        ("print mean magnitude of (a list of 1, 2).", "average"),
        ("print mean absolute error of (a list of 1, 2).", "average"),
        ("print mae of (a list of 1, 2).", "average"),
        ("print max magnitude of (a list of 1, 2).", "maximum"),
    ] {
        let ast = compiler.parse(source).unwrap().to_json().to_string();
        assert!(ast.contains(op) && ast.contains("map") && ast.contains("absolute"),
            "{source}: {ast}");
        assert!(compiler.optimized_hir(source).is_ok(), "{source}");
        assert_eq!(compiler.effect_analysis(source).unwrap().effects(),
            vec!["console_write".to_string()]);
    }
}
