// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-median-{}-{}.sema", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn odd_and_even_lists_produce_correct_medians() {
    let result = run(r#"let odd be a list of 9, 1, 3.
let even be a list of 10, 2, 8, 4.
assert median of odd is equal to 3.
assert median of even is equal to 6.
assert (odd at 0) is equal to 9.
assert median of (reverse of odd) is equal to 3.
print median of odd.
print median of even."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "3\n6\n");
}

#[test]
fn handles_fractional_median_and_negative_values() {
    let result = run(
        "print median of a list of 2, 3.          print median of a list of minus 8, 2, 5.          print median of a list of 1.5, 0.5, 2.5."
    );
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "2.5\n2\n1.5\n");
}

#[test]
fn preserves_integer_precision_when_medians_are_exact() {
    let result = run(
        "print median of a list of 9007199254740995, 9007199254740993.          print median of a list of 9007199254740997, 9007199254740993, 9007199254740995."
    );
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "9007199254740994\n9007199254740995\n");
}

#[test]
fn keeps_integer_midpoint_safe_at_i64_bounds() {
    let result = run(
        "print median of a list of 9223372036854775807, 9223372036854775807.          print median of a list of minus 9223372036854775807, 9223372036854775807."
    );
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "9223372036854775807\n0\n");
}

#[test]
fn unary_minus_preserves_exact_large_integer_values() {
    let result = run("print minus 9007199254740993.");
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "-9007199254740993\n");
    let optimized = Compiler::new().optimized_hir("print minus 9007199254740993.")
        .unwrap().to_json().to_string();
    assert!(optimized.contains("-9007199254740993"), "{optimized}");
}

#[test]
fn rejects_wrong_operands_with_numeric_type_error() {
    for source in [
        "print median of 9.",
        "print median of true.",
        "print median of a list of \"x\", \"y\".",
    ] {
        let error = Compiler::new().hir(source).unwrap_err();
        assert_eq!(error.code, "S353", "{source}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn median_integrates_with_formatter_hir_and_effects() {
    let compiler = Compiler::new();
    assert_eq!(
        compiler.format_source("PRINT MEDIAN OF a LIST OF 1, 3.").unwrap(),
        "print median of a list of 1, 3."
    );
    assert!(compiler.hir("let middle be median of a list of 2, 4.")
        .unwrap().to_json().to_string().contains("\"median\""));
    assert_eq!(
        compiler.effect_analysis("print median of a list of 2, 4.")
            .unwrap().effects(),
        vec!["console_write".to_owned()]
    );
}
