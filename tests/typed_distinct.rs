// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-distinct-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)
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
fn distinct_preserves_first_occurrence_and_source_immutability() {
    let result = run(r#"let input be a list of 3, 1, 3, 2, 1, 3.
let output be distinct of input.
assert length of output is equal to 3.
assert (output at 0) is equal to 3.
assert (output at 1) is equal to 1.
assert (output at 2) is equal to 2.
assert length of input is equal to 6.
print output.
print input."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[3, 1, 2]\n[3, 1, 3, 2, 1, 3]\n");
}

#[test]
fn distinct_supports_strings_booleans_and_colors() {
    let result = run(r#"print distinct of a list of "hi", "hi", "bye", "hi".
print distinct of a list of true, false, true, false.
print distinct of a list of red, blue, red."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[hi, bye]\n[true, false]\n[red, blue]\n");
}

#[test]
fn composes_with_sorted_and_other_typed_list_operators() {
    let result = run(r#"let values be a list of 3, 1, 3, 2, 1.
let unique_sorted be sorted of (distinct of values).
assert (unique_sorted at 0) is equal to 1.
assert (unique_sorted at 2) is equal to 3.
assert length of unique_sorted is equal to 3.
print unique_sorted."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[1, 2, 3]\n");
}

#[test]
fn distinguishes_integer_and_float_storage_like_contains() {
    let result = run(
        "print distinct of a list of 1, 1.0, 1, 1.0."
    );
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[1, 1.0]\n");
}

#[test]
fn rejects_non_list_values_with_source_aware_error() {
    for source in [
        "print distinct of 2.",
        "print distinct of true.",
        "print distinct of \"text\".",
    ] {
        let error = Compiler::new().hir(source).unwrap_err();
        assert_eq!(error.code, "S360", "{source}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn formatter_hir_and_effects_recognize_distinct() {
    let compiler = Compiler::new();
    assert_eq!(
        compiler.format_source("PRINT DISTINCT OF a LIST OF 1, 2, 1.").unwrap(),
        "print distinct of a list of 1, 2, 1."
    );
    assert!(compiler.hir("let x be distinct of a list of true, false, true.")
        .unwrap().to_json().to_string().contains("\"distinct\""));
    assert_eq!(
        compiler.effect_analysis("print distinct of a list of 1, 1.")
            .unwrap().effects(),
        vec!["console_write".to_owned()]
    );
}
