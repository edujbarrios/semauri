// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-json-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(),
        "console_write".into(), path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn serializes_scalars_lists_and_escapes() {
    let source = r#"let labels be a list of "one", "two".
let encoded be json of labels.
assert encoded is equal to "[\\"one\\",\\"two\\"]".
assert (json of true) is equal to "true".
assert (json of false) is equal to "false".
assert (json of 42) is equal to "42".
assert (json of "hello") is equal to "\\"hello\\"".
assert (json of "") is equal to "\\"\\"".
print encoded."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[\"one\",\"two\"]\n");
}

#[test]
fn contextual_json_keyword_preserves_identifiers() {
    let result = run(r#"let json be "value".
assert (json of json) is equal to "\\"value\\"".
assert json is equal to "value"."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn type_checks_and_keeps_effects_explicit() {
    let compiler = Compiler::new();
    let source = "print json of (a list of 1, 2, 3).";
    assert_eq!(compiler.effect_analysis(source).unwrap().effects(),
        vec!["console_write".to_string()]);
    let source = "print json of (connect language model \"local/default\").";
    let error = compiler.hir(source).unwrap_err();
    assert_eq!(error.code, "S390", "{error}");
}

#[test]
fn folds_literal_json_and_formats_contextually() {
    let compiler = Compiler::new();
    let optimized = compiler.optimized_hir("print json of true.").unwrap()
        .to_json().to_string();
    assert!(optimized.contains("true"), "{optimized}");
    assert_eq!(compiler.format_source("PRINT JSON OF true.").unwrap(),
        "print json of true.");
}
