// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-string-case-{}-{}.sema",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&[
        "run".into(),
        "--allow".into(),
        "console_write".into(),
        path.to_string_lossy().into_owned(),
    ]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn supports_string_transforms_in_bindings_print_and_assertions() {
    let source = r#"let name be "Semauri".
let loud be uppercase of name.
let quiet be lowercase of loud.
assert loud is equal to "SEMAURI".
assert quiet is equal to "semauri".
print loud.
print quiet."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "SEMAURI\nsemauri\n");
}

#[test]
fn transforms_unicode_using_rust_unicode_case_rules() {
    let result = run("print uppercase of \"straße\". print lowercase of \"ÑANDÚ\".");
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "STRASSE\nñandú\n");
}

#[test]
fn supports_nested_and_parenthesized_expressions() {
    let result = run(
        "print uppercase of (\"hello\" plus \" world\").          print lowercase of (uppercase of \"WoRlD\").",
    );
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "HELLO WORLD\nworld\n");
}

#[test]
fn rejects_non_string_values_at_type_check_time() {
    let compiler = Compiler::new();
    for source in [
        "print uppercase of 12.",
        "print lowercase of true.",
        "print uppercase of a list of 1, 2.",
    ] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S351", "{source}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn constant_folding_and_formatter_support_both_operators() {
    let compiler = Compiler::new();
    let optimized = compiler
        .optimized_hir("print uppercase of \"abc\".")
        .unwrap()
        .to_json()
        .to_string();
    assert!(optimized.contains("ABC"), "{optimized}");
    assert_eq!(
        compiler.format_source("PRINT UPPERCASE OF \"Case-sensitive text\".")
            .unwrap(),
        "print uppercase of \"Case-sensitive text\"."
    );
    assert_eq!(
        compiler.effect_analysis("print lowercase of \"TEST\".")
            .unwrap()
            .effects(),
        vec!["console_write".to_string()]
    );
}
