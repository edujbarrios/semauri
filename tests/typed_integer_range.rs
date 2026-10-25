// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-range-{}-{}.sema", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn produces_inclusive_ascending_descending_and_singleton_ranges() {
    let result = run(r#"print range from 1 to 4.
print range from 3 to 1.
print range from 7 to 7."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[1, 2, 3, 4]\n[3, 2, 1]\n[7]\n");
}

#[test]
fn negative_and_extreme_integer_bounds_do_not_overflow() {
    let result = run(r#"print range from minus 2 to 1.
print range from 9223372036854775806 to 9223372036854775807.
print range from minus 9223372036854775807 to minus 9223372036854775806."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout,
        "[-2, -1, 0, 1]\n[9223372036854775806, 9223372036854775807]\n[-9223372036854775807, -9223372036854775806]\n");
}

#[test]
fn generated_lists_compose_with_map_and_filter() {
    let result = run(r#"let numbers be range from 1 to 6.
let squared be map every n in numbers to (n times n).
let even be filter every n in squared if ((n modulo 2) is equal to 0).
assert length of even is equal to 3.
assert (even at 0) is equal to 4.
assert (even at 2) is equal to 36.
print even."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[4, 16, 36]\n");
}

#[test]
fn refuses_fractional_and_oversized_ranges_at_evaluation() {
    for source in [
        "print range from 1.5 to 4.",
        "print range from 1 to 3.2.",
        "print range from 1 to 10001.",
        "print range from 10001 to 1.",
        "print range from minus 9223372036854775807 to 9223372036854775807.",
    ] {
        let result = run(source);
        assert_ne!(result.status, 0, "{source}");
        assert!(result.stderr.contains("S368"), "{source}: {}", result.stderr);
    }
}

#[test]
fn invalid_endpoint_types_fail_early_with_source_span() {
    for source in [
        "print range from true to 4.",
        "print range from 1 to \"hi\".",
        "print range from \"a\" to false.",
    ] {
        let err = Compiler::new().hir(source).unwrap_err();
        assert_eq!(err.code, "S367", "{source}: {err}");
        assert!(err.span.is_some());
    }
}

#[test]
fn range_is_contextual_and_does_not_break_prior_identifier_names() {
    let result = run(r#"let range be 9.
print range.
print range from 1 to 2."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "9\n[1, 2]\n");
    let c = Compiler::new();
    assert_eq!(
        c.format_source("LET Range BE 9. PRINT Range.").unwrap(),
        "let Range be 9. print Range."
    );
    assert_eq!(
        c.format_source("PRINT RANGE FROM 1 TO 3.").unwrap(),
        "print range from 1 to 3."
    );
}

#[test]
fn ast_hir_optimizer_and_effects_cover_range() {
    let c = Compiler::new();
    let source = "let xs be range from 1 to 3. print xs.";
    assert!(c.parse(source).unwrap().to_json().to_string().contains("range"));
    assert!(c.hir(source).unwrap().to_json().to_string().contains("range"));
    assert!(c.optimized_hir(source).unwrap().to_json().to_string().contains("range"));
    assert_eq!(c.effect_analysis(source).unwrap().effects(), vec!["console_write".to_owned()]);
}
