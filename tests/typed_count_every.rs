// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-count-{}-{}.sema", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let r = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    r
}

#[test]
fn counts_predicate_matches_without_mutating_source() {
    let result = run(r#"let scores be a list of 2, 9, 3, 10, 7.
let passes be count every score in scores if (score is greater than 6).
let misses be count every score in scores if (score is less than 0).
assert passes is equal to 3.
assert misses is equal to 0.
assert length of scores is equal to 5.
print passes.
print misses."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "3\n0\n");
}

#[test]
fn counts_typed_empty_lists_and_boolean_predicates() {
    let result = run(r#"let xs be take 0 from a list of 1, 2.
let count be count every x in xs if true.
let booleans be a list of true, false, true.
print count.
print count every x in booleans if x."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "0\n2\n");
}

#[test]
fn counts_with_user_functions_and_outer_lexical_bindings() {
    let result = run(r#"let cutoff be 4.
define function good taking score as number to (score is greater than cutoff).
let samples be range from 1 to 7.
let passing be count every x in samples if (invoke good to x).
assert passing is equal to 3.
print passing."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "3\n");
}

#[test]
fn rejects_nonlist_and_nonboolean_predicates() {
    for (source, code) in [
        ("print count every x in 2 if true.", "S365"),
        ("print count every x in (a list of 1, 2) if (x plus 1).", "S366"),
    ] {
        let e = Compiler::new().hir(source).unwrap_err();
        assert_eq!(e.code, code, "{source}: {e}");
        assert!(e.span.is_some());
    }
}

#[test]
fn iterator_cannot_escape_count_expression() {
    let err = Compiler::new().hir(
        "let count be count every x in (a list of 1, 2) if true. print x."
    ).unwrap_err();
    assert_eq!(err.code, "S312");
}

#[test]
fn count_keyword_is_contextual_for_old_variable_names_and_labels() {
    let result = run(r#"let count be 7.
print count.
let xs be a list of 2, 4, 6.
print count every x in xs if (x is greater than count)."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "7\n0\n");
}

#[test]
fn formatter_hir_optimizer_and_effects_understand_count() {
    let c = Compiler::new();
    assert_eq!(
        c.format_source("PRINT COUNT EVERY x IN (a LIST OF 1, 2) IF (x IS GREATER THAN 1).").unwrap(),
        "print count every x in (a list of 1, 2) if (x is greater than 1)."
    );
    assert_eq!(c.format_source("LET Count BE 2. PRINT Count.").unwrap(),
        "let Count be 2. print Count."
    );
    let source = "print count every x in (a list of 1, 2) if (x is greater than 1).";
    assert!(c.hir(source).unwrap().to_json().to_string().contains("\"count\""));
    assert!(c.optimized_hir(source).unwrap().to_json().to_string().contains("\"count\""));
    assert_eq!(c.effect_analysis(source).unwrap().effects(), vec!["console_write".to_owned()]);
}
