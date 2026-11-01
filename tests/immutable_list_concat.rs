// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-list-concat-{}-{}.suri",
        std::process::id(),
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
fn plus_concatenates_homogeneous_number_lists_without_mutating_inputs() {
    let source = r#"Let first be a list of 10, 20.
Let second be a list of 30, 40.
Let joined be first plus second.
Print first.
Print second.
Print joined.
Print Length of joined."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[10, 20]\n[30, 40]\n[10, 20, 30, 40]\n4\n");
}

#[test]
fn plus_supports_typed_text_lists_and_chaining() {
    let source = r#"Let first be a list of "Ada", "Grace".
Let second be a list of "Alan".
Let third be a list of "Edsger".
Print first plus second plus third."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[Ada, Grace, Alan, Edsger]\n");
}

#[test]
fn plus_rejects_heterogeneous_lists_or_list_scalar_mix() {
    for source in [
        "Let first be a list of 1, 2. Let second be a list of \"3\", \"4\". Print first plus second.",
        "Let first be a list of 1, 2. Print first plus 3.",
    ] {
        let error = Compiler::new().hir(source).unwrap_err();
        assert_eq!(error.code, "S314", "{source}");
    }
}

#[test]
fn preserves_existing_string_and_numeric_plus_semantics() {
    let result = run("Print \"semi\" plus \"uri\". Print 20 plus 22.");
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "semiuri\n42\n");
}

#[test]
fn list_plus_is_pure_and_preserves_element_type_in_hir() {
    let source = "Let xs be a list of 1, 2. Let ys be a list of 3, 4. Print xs plus ys.";
    let compiler = Compiler::new();
    assert_eq!(
        compiler.effect_analysis(source).unwrap().effects(),
        vec!["console_write".to_string()]
    );
    let hir = compiler.hir(source).unwrap().to_json().to_string();
    assert!(hir.contains("element_type"), "{hir}");
}
