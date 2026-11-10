// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-series-{}-{}.suri",
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
fn executable_series_example_checks_and_runs() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/running_statistics.suri");
    let check = run_cli(&["check".into(), path.into()]);
    assert_eq!(check.status, 0, "{}", check.stderr);
    let result = run(include_str!("../examples/running_statistics.suri"));
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Running statistics OK\n");
}

#[test]
fn aliases_prefix_lengths_signs_and_empty_sequences() {
    let source = r#"let readings be a list of 3, 5, 2, 6.
assert (running sum of readings) is equal to (cumulative total of readings).
assert (running total of readings) is equal to (cumulative sum of readings).
assert (running average of readings) is equal to (cumulative average of readings).
assert (adjacent differences of readings) is equal to (successive differences of readings).
assert (adjacent differences of (a list of 8, 3, 7)) is equal to (a list of (minus 5), 4).
assert (running total of (drop 1 from (a list of 9))) is equal to (drop 1 from (a list of 9)).
assert (running average of (drop 1 from (a list of 9))) is equal to (drop 1 from (a list of 9)).
assert (adjacent differences of (a list of 9)) is equal to (drop 1 from (a list of 1)).
assert (adjacent differences of (drop 1 from (a list of 1))) is equal to (drop 1 from (a list of 1))."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn large_integer_precision_and_decimal_values() {
    let source = r#"let large be a list of 9007199254740993, 2.
assert (running total of large) is equal to (a list of 9007199254740993, 9007199254740995).
assert (adjacent differences of (a list of 9007199254740995, 9007199254740993)) is equal to (a list of (minus 2)).
assert (running total of (a list of 1.5, 2, (minus 0.5))) is equal to (a list of 1.5, 3.5, 3).
assert (running average of (a list of 1.5, 2.5)) is equal to (a list of 1.5, 2).
assert (adjacent differences of (a list of 1.5, 2, 1)) is equal to (a list of 0.5, (minus 1))."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn type_checks_and_overflow_have_located_errors() {
    let compiler = Compiler::new();
    for source in [
        "print running total of true.",
        "print running average of 4.",
        "print cumulative sum of (a list of \"wrong\").",
        "print adjacent differences of (a list of false).",
        "print successive differences of \"text\".",
    ] {
        let err = compiler.hir(source).unwrap_err();
        assert_eq!(err.code, "S353", "{source}: {err}");
        assert!(err.span.is_some());
    }
    for (source, code) in [
        ("print running total of (a list of 9223372036854775807, 1).", "S471"),
        ("print cumulative average of (a list of 9223372036854775807, 1).", "S471"),
        ("print adjacent differences of (a list of (minus 9223372036854775807), 2).", "S470"),
    ] {
        let result = run(source);
        assert_eq!(result.status, 65, "{source}: {}", result.stderr);
        assert!(result.stderr.contains(code), "{source}: {}", result.stderr);
    }
}

#[test]
fn formatter_is_idempotent_and_words_remain_identifiers() {
    let compiler = Compiler::new();
    let raw = "PRINT RUNNING TOTAL OF (A LIST OF 1, 2).\nPRINT RUNNING SUM OF (A LIST OF 1, 2).\nPRINT CUMULATIVE AVERAGE OF (A LIST OF 1, 2).\nPRINT ADJACENT DIFFERENCES OF (A LIST OF 1, 2).\nPRINT SUCCESSIVE DIFFERENCES OF (A LIST OF 1, 2).";
    let formatted = "print running total of (a list of 1, 2).\nprint running sum of (a list of 1, 2).\nprint cumulative average of (a list of 1, 2).\nprint adjacent differences of (a list of 1, 2).\nprint successive differences of (a list of 1, 2).";
    assert_eq!(compiler.format_source(raw).unwrap(), formatted);
    assert_eq!(compiler.format_source(formatted).unwrap(), formatted);
    let source = r#"let running be 1.
let cumulative be 2.
let adjacent be 3.
let successive be 4.
let differences be 5.
let total be 6.
assert running is equal to 1.
assert cumulative is equal to 2.
assert adjacent is equal to 3.
assert successive is equal to 4.
assert differences is equal to 5.
assert total is equal to 6."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(compiler.format_source("LET Running BE 3. PRINT Running.").unwrap(),
        "let Running be 3. print Running.");
}

#[test]
fn type_ir_optimizer_effects_and_single_source_evaluation() {
    let compiler = Compiler::new();
    for (source, operator) in [
        ("print running total of (a list of 1, 2).", "running_total"),
        ("print cumulative average of (a list of 1, 2).", "running_average"),
        ("print adjacent differences of (a list of 1, 2).", "adjacent_differences"),
    ] {
        let ast = compiler.parse(source).unwrap().to_json().to_string();
        assert!(ast.contains(operator), "{source}: {ast}");
        assert!(compiler.optimized_hir(source).is_ok(), "{source}");
        assert_eq!(compiler.effect_analysis(source).unwrap().effects(),
            vec!["console_write".to_owned()]);
    }
    assert!(compiler.effect_analysis(
        "let xs be a list of 1, 2. let ys be running total of xs."
    ).unwrap().effects().is_empty());
}
