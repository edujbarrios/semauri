// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-index-{}-{}.sema", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&[
        "run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()
    ]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn zero_based_access_preserves_element_types() {
    let source = r#"Let numbers be a list of 5, 9, 13.
Let fruits be a list of "apple", "pear".
Print numbers at 0.
Print numbers at 2.
Print fruits at 1.
Let selected be numbers at 1.
Print selected."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "5\n13\npear\n9\n");
}

#[test]
fn indexed_values_compose_with_arithmetic_and_assertions() {
    let source = r#"Let numbers be a list of 5, 9, 13.
Print (numbers at 1) plus 3.
Assert (numbers at 1) is equal to 9."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "12\n");
}

#[test]
fn list_access_rejects_wrong_types_during_hir_validation() {
    for source in [
        "Print 3 at 0.",
        "Print \"abc\" at 0.",
        "Let values be a list of 1, 2. Print values at \"first\".",
    ] {
        let error = Compiler::new().hir(source).unwrap_err();
        assert_eq!(error.code, "S345", "{source}");
    }
}

#[test]
fn bounds_and_fractional_indices_report_source_aware_errors() {
    for (source, diagnostic) in [
        ("Let xs be a list of 1, 2. Print xs at 5.", "out of bounds"),
        ("Let xs be a list of 1, 2. Print xs at minus 1.", "non-negative integer"),
        ("Let xs be a list of 1, 2. Print xs at 1.5.", "non-negative integer"),
    ] {
        let result = run(source);
        assert_eq!(result.status, 65, "{source}: {}", result.stderr);
        assert!(result.stderr.contains("S346"), "{source}: {}", result.stderr);
        assert!(result.stderr.contains(diagnostic), "{source}: {}", result.stderr);
    }
}

#[test]
fn plan_retains_typed_at_index_operator_and_no_new_effects() {
    let compiler = Compiler::new();
    let source = "Let numbers be a list of 1, 2. Print numbers at 1.";
    let ast = compiler.parse(source).unwrap().to_json().to_string();
    assert!(ast.contains("at_index"), "{ast}");
    assert_eq!(
        compiler.effect_analysis(source).unwrap().effects(),
        vec!["console_write".to_string()]
    );
}
