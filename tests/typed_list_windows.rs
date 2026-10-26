// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-windows-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn take_and_drop_preserve_order_type_and_source() {
    let result = run(r#"let values be a list of 8, 2, 5, 1.
let first_two be take 2 from values.
let after_two be drop 2 from values.
assert length of first_two is equal to 2.
assert length of after_two is equal to 2.
assert first of first_two is equal to 8.
assert last of first_two is equal to 2.
assert first of after_two is equal to 5.
assert (values at 0) is equal to 8.
print first_two.
print after_two.
print values."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[8, 2]\n[5, 1]\n[8, 2, 5, 1]\n");
}

#[test]
fn zero_or_excess_count_clamps_and_keeps_typed_empty_lists() {
    let result = run(r#"let values be a list of 9, 8.
let no_values be take 0 from values.
let dropped_all be drop 99 from values.
assert length of no_values is equal to 0.
assert length of dropped_all is equal to 0.
assert length of (take 99 from values) is equal to 2.
assert length of (drop 0 from values) is equal to 2.
print no_values.
print dropped_all."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[]\n[]\n");
}

#[test]
fn composes_with_sort_and_distinct_and_parenthesized_count() {
    let result = run(r#"let data be a list of 9, 3, 3, 1, 8.
let top be take (1 plus 1) from (reverse of (sorted of (distinct of data))).
assert (top at 0) is equal to 9.
assert (top at 1) is equal to 8.
print top."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[9, 8]\n");
}

#[test]
fn works_for_strings_booleans_and_nested_lists() {
    let result = run(r#"print take 2 from a list of "a", "b", "c".
print drop 1 from a list of true, false, true.
let nested be a list of (a list of 1, 2), (a list of 3, 4).
let selected be take 1 from nested.
assert (first of selected at 0) is equal to 1."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert!(result.stdout.starts_with("[a, b]\n[false, true]\n"), "{}", result.stdout);
}

#[test]
fn rejects_invalid_type_combinations_at_compile_time() {
    for source in [
        "print take true from a list of 1, 2.",
        "print drop 1 from 123.",
        "print take 2 from false.",
    ] {
        let e = Compiler::new().hir(source).unwrap_err();
        assert_eq!(e.code, "S363", "{source}: {e}");
        assert!(e.span.is_some());
    }
}

#[test]
fn rejects_negative_and_fractional_counts_at_evaluation() {
    for source in [
        "print take minus 1 from a list of 4, 5.",
        "print drop 1.5 from a list of 4, 5.",
    ] {
        let result = run(source);
        assert_ne!(result.status, 0, "{source}");
        assert!(result.stderr.contains("S364"), "{source}: {}", result.stderr);
    }
}

#[test]
fn formatter_hir_and_effects_preserve_binary_selection_operators() {
    let c = Compiler::new();
    assert_eq!(
        c.format_source("PRINT TAKE 2 FROM a LIST OF 4, 5.").unwrap(),
        "print take 2 from a list of 4, 5."
    );
    assert!(c.hir("let x be drop 2 from a list of 4, 5.")
        .unwrap().to_json().to_string().contains("\"drop\""));
    assert_eq!(
        c.effect_analysis("print take 1 from a list of 2, 3.")
            .unwrap().effects(),
        vec!["console_write".to_owned()]
    );
}
