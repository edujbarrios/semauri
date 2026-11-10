// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-squared-stats-{}-{}.suri",
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
fn checked_in_example_runs_and_checks() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/squared_statistics.suri");
    let check = run_cli(&["check".into(), path.into()]);
    assert_eq!(check.status, 0, "{}", check.stderr);
    let result = run(include_str!("../examples/squared_statistics.suri"));
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Squared statistics OK\n");
}

#[test]
fn supports_exact_large_integer_squares_and_finite_decimals() {
    let result = run(r#"assert (sum of squares of (a list of 134217729)) is equal to 18014398777917441.
assert (mean square of (a list of 3, 4)) is equal to 12.5.
assert (root mean square of (a list of 3, 4)) is equal to (sqrt of 12.5).
assert (rms of (a list of 3, 4)) is equal to (sqrt of 12.5).
assert (sum of squares of (a list of 1.5, 2)) is equal to 6.25.
assert (mean square of (a list of 1.5, 2)) is equal to 3.125.
assert (sum of squares of (drop 1 from (a list of 7))) is equal to 0."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn rejects_bad_inputs_and_checked_overflow() {
    let compiler = Compiler::new();
    for source in [
        "print sum of squares of 5.",
        "print mean square of true.",
        "print rms of 3.",
    ] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S365", "{source}: {error}");
        assert!(error.span.is_some());
    }
    for source in [
        "print root mean square of (a list of \"wrong\").",
        "print sum of squares of (a list of true).",
    ] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S421", "{source}: {error}");
    }
    for (source, code) in [
        ("print rms of (drop 1 from (a list of 1)).", "S354"),
        ("print mean square of (drop 1 from (a list of 1)).", "S354"),
        ("print sum of squares of (a list of 3037000500).", "S422"),
        ("print sum of squares of (a list of 2147483647, 2147483647, 2147483647).", "S354"),
    ] {
        let result = run(source);
        assert_eq!(result.status, 65, "{source}: {}", result.stderr);
        assert!(result.stderr.contains(code), "{source}: {}", result.stderr);
    }
}

#[test]
fn prefixes_are_contextual_and_formatter_is_idempotent() {
    let compiler = Compiler::new();
    let raw = "PRINT SUM OF SQUARES OF (A LIST OF 3, 4).\nPRINT MEAN SQUARE OF (A LIST OF 3, 4).\nPRINT ROOT MEAN SQUARE OF (A LIST OF 3, 4).\nPRINT RMS OF (A LIST OF 3, 4).";
    let formatted = "print sum of squares of (a list of 3, 4).\nprint mean square of (a list of 3, 4).\nprint root mean square of (a list of 3, 4).\nprint rms of (a list of 3, 4).";
    assert_eq!(compiler.format_source(raw).unwrap(), formatted);
    assert_eq!(compiler.format_source(formatted).unwrap(), formatted);

    let source = r#"let rms be 1.
let root be 2.
let square be 3.
let squares be 4.
let mean be 5.
assert rms is equal to 1.
assert root is equal to 2.
assert square is equal to 3.
assert squares is equal to 4.
assert mean is equal to 5."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    let preserved = "LET Rms BE 1. PRINT Rms.";
    assert_eq!(compiler.format_source(preserved).unwrap(), "let Rms be 1. print Rms.");
}

#[test]
fn expansions_remain_typed_and_pure_and_analyzable() {
    let compiler = Compiler::new();
    for (source, name) in [
        ("print sum of squares of (a list of 1, 2).", "sum"),
        ("print mean square of (a list of 1, 2).", "average"),
        ("print root mean square of (a list of 1, 2).", "sqrt"),
        ("print rms of (a list of 1, 2).", "sqrt"),
    ] {
        let ast = compiler.parse(source).unwrap().to_json().to_string();
        assert!(ast.contains(name) && ast.contains("map") && ast.contains("square"),
            "{source}: {ast}");
        assert!(compiler.optimized_hir(source).is_ok(), "{source}");
        assert_eq!(compiler.effect_analysis(source).unwrap().effects(),
            vec!["console_write".to_string()]);
    }
}
