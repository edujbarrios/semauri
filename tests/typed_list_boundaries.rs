// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-boundaries-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn first_and_last_preserve_the_element_type_and_source_list() {
    let result = run(r#"let values be a list of 9, 2, 5.
let first_value be first of values.
let last_value be last of values.
assert first_value is equal to 9.
assert last_value is equal to 5.
assert (values at 0) is equal to 9.
assert first of (sorted of values) is equal to 2.
print first_value.
print last_value."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "9\n5\n");
}

#[test]
fn handles_boolean_string_and_nested_list_elements() {
    let result = run(r#"assert first of (a list of true, false) is equal to true.
assert last of (a list of "red", "blue") is equal to "blue".
let nested be a list of (a list of 1, 2), (a list of 3, 4).
let first_inner be first of nested.
assert (first_inner at 0) is equal to 1.
print last of a list of false, true."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "true\n");
}

#[test]
fn preserves_legacy_first_and_last_variable_identifiers() {
    let result = run(r#"let first be a list of 1, 2.
let last be a list of 3, 4.
let joined be first plus last.
assert length of joined is equal to 4.
print first.
print last.
print first of joined."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[1, 2]\n[3, 4]\n1\n");
}

#[test]
fn single_item_list_is_its_own_first_and_last() {
    let result = run("print first of a list of 7. print last of a list of 7.");
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "7\n7\n");
}

#[test]
fn rejects_nonlist_inputs_with_precise_type_error() {
    for source in [
        "print first of 5.",
        "print last of true.",
        "print first of \"hello\".",
    ] {
        let err = Compiler::new().hir(source).unwrap_err();
        assert_eq!(err.code, "S361", "{source}: {err}");
        assert!(err.span.is_some());
    }
}

#[test]
fn formatter_ast_hir_and_effects_support_new_prefix_expressions() {
    let compiler = Compiler::new();
    assert_eq!(
        compiler.format_source("PRINT FIRST OF a LIST OF 4, 3.").unwrap(),
        "print first of a list of 4, 3."
    );
    assert_eq!(
        compiler.format_source("Create a web called My First page.").unwrap(),
        "create a web called My First page."
    );
    assert!(compiler.hir("let x be last of a list of 4, 3.")
        .unwrap().to_json().to_string().contains("\"last\""));
    assert_eq!(
        compiler.effect_analysis("print first of a list of 3, 4.")
            .unwrap().effects(),
        vec!["console_write".to_owned()]
    );
}
