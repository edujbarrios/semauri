// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-map-filter-{}-{}.sema", std::process::id(),
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
fn map_transforms_list_elements_and_preserves_source_type() {
    let result = run(r#"let values be a list of 1, 2, 3.
let twice be map every value in values to (value times 2).
assert (twice at 0) is equal to 2.
assert (twice at 2) is equal to 6.
assert (values at 0) is equal to 1.
print twice.
print values."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[2, 4, 6]\n[1, 2, 3]\n");
}

#[test]
fn map_supports_type_changes_outer_bindings_and_shadowing() {
    let result = run(r#"let offset be 4.
let value be 100.
let values be a list of 1, 3.
let result be map every value in values to (value plus offset).
assert (result at 0) is equal to 5.
assert (result at 1) is equal to 7.
assert value is equal to 100.
let comparisons be map every n in values to (n is greater than 2).
assert (comparisons at 0) is equal to false.
assert (comparisons at 1) is equal to true.
print comparisons."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[false, true]\n");
}

#[test]
fn filter_uses_boolean_predicate_and_preserves_element_type() {
    let result = run(r#"let values be a list of 1, 4, 2, 5.
let selected be filter every n in values if (n is greater than 2).
assert length of selected is equal to 2.
assert (selected at 0) is equal to 4.
assert (selected at 1) is equal to 5.
print selected.
print values."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[4, 5]\n[1, 4, 2, 5]\n");
}

#[test]
fn supports_empty_results_and_composed_typed_pipelines() {
    let result = run(r#"let values be a list of 4, 1, 2, 3.
let filtered be filter every n in values if (n is greater than 20).
let mapped be map every n in filtered to (n times 2).
assert length of filtered is equal to 0.
assert length of mapped is equal to 0.
let even be filter every n in values if ((n modulo 2) is equal to 0).
let result be sorted of (map every n in even to (n times 3)).
assert (result at 0) is equal to 6.
assert (result at 1) is equal to 12.
print result."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[6, 12]\n");
}

#[test]
fn rejects_wrong_input_types_and_nonboolean_filters() {
    for (source, expected) in [
        ("let result be map every n in 3 to (n plus 1).", "S365"),
        ("let result be filter every n in 3 if true.", "S365"),
        ("let result be filter every n in (a list of 1, 2) if (n plus 1).", "S366"),
    ] {
        let err = Compiler::new().hir(source).unwrap_err();
        assert_eq!(err.code, expected, "{source}: {err}");
        assert!(err.span.is_some());
    }
}

#[test]
fn iterator_variable_is_lexically_scoped_and_not_leaked() {
    let error = Compiler::new().hir(
        "let values be a list of 1, 2. let output be map every n in values to (n times 2). print n."
    ).unwrap_err();
    assert_eq!(error.code, "S312");
}

#[test]
fn map_and_filter_remain_valid_variable_names() {
    let result = run(r#"let map be 4.
let filter be 5.
print map plus filter.
print map every x in (a list of 1, 2) to (x plus map)."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "9\n[5, 6]\n");
}

#[test]
fn formatter_ast_hir_optimizer_and_effects_integrate() {
    let c = Compiler::new();
    assert_eq!(
        c.format_source("PRINT MAP EVERY n IN (a LIST OF 1, 2) TO (n PLUS 1).").unwrap(),
        "print map every n in (a list of 1, 2) to (n plus 1)."
    );
    assert_eq!(
        c.format_source("let Map be 3. print Map.").unwrap(),
        "let Map be 3. print Map."
    );
    let source = "let xs be a list of 1, 2. let ys be filter every n in xs if (n is greater than 1). print ys.";
    assert!(c.parse(source).unwrap().to_json().to_string().contains("list_transform"));
    assert!(c.hir(source).unwrap().to_json().to_string().contains("iterator_symbol_id"));
    assert!(c.optimized_hir(source).unwrap().to_json().to_string().contains("list_transform"));
    assert_eq!(c.effect_analysis(source).unwrap().effects(), vec!["console_write".to_owned()]);
}
