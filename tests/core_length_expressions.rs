// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "semauri-length-{}-{id}.suri", std::process::id()
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&[
        "run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().to_string(),
    ]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn length_is_typed_and_usable_in_core_arithmetic_and_conditions() {
    let source = r#"Let label be "Semauri".
Let count be Length of label.
If count is equal to 7:
  Print count plus 1.
Otherwise:
  Print 0.
End."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "8\n");
}

#[test]
fn length_counts_unicode_scalar_values_not_utf8_bytes() {
    let result = run("Print Length of \"ñ🌍\".");
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "2\n");
}

#[test]
fn length_counts_typed_list_items_without_changing_list_values() {
    let result = run(
        "Let numbers be a list of 10, 20, 30. Print Length of numbers. Print numbers."
    );
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "3\n[10, 20, 30]\n");
}

#[test]
fn length_supports_parenthesized_expressions() {
    let result = run("Print Length of (\"Hi\" plus \" there\").");
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "8\n");
}

#[test]
fn length_rejects_non_collection_inputs_without_dynamic_coercion() {
    for source in [
        "Print Length of 42.",
        "Print Length of true.",
        "Print Length of red.",
    ] {
        let error = Compiler::new().hir(source).unwrap_err();
        assert_eq!(error.code, "S343", "{source}");
    }
}

#[test]
fn length_folds_without_erasing_effectful_computation() {
    let optimized = Compiler::new()
        .optimized_hir("Print Length of \"abc\".")
        .unwrap()
        .to_json().to_string();
    assert!(optimized.contains("\"value\":3") || optimized.contains("\"value\": 3"));
    assert_eq!(
        Compiler::new().effect_analysis("Print Length of \"abc\".")
            .unwrap().effects(),
        vec!["console_write".to_string()]
    );
}
