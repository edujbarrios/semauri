// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-compact-numeric-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let output = run_cli(&[
        "run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()
    ]);
    fs::remove_file(path).unwrap();
    output
}

#[test]
fn med_and_prod_reuse_existing_numeric_aggregations() {
    let output = run("let numbers be a list of 2, 4, 6.\nprint med of numbers.\nprint prod of numbers.\nprint med of a list of 1, 2.\nprint prod of a list of minus 2, 3.");
    assert_eq!(output.status, 0, "{}", output.stderr);
    assert_eq!(output.stdout, "4\n48\n1.5\n-6\n");
}

#[test]
fn aliases_keep_type_diagnostics_and_optimizer_paths() {
    let compiler = Compiler::new();
    for (alias, canonical, operand) in [
        ("med", "median", "true"),
        ("prod", "product", r#""hello""#),
    ] {
        let actual = compiler.hir(&format!("print {alias} of {operand}.")).unwrap_err();
        let expected = compiler.hir(&format!("print {canonical} of {operand}.")).unwrap_err();
        assert_eq!(actual.code, expected.code, "{alias} must keep {canonical} checks");
        assert!(actual.span.is_some());
    }
    for source in [
        "print med of a list of 2, 4, 6.",
        "print prod of a list of 2, 4, 6.",
    ] {
        assert!(compiler.optimized_hir(source).is_ok());
        assert_eq!(compiler.effect_analysis(source).unwrap().effects(),
            vec!["console_write".to_string()]);
    }
}

#[test]
fn formatter_and_contextual_identifiers_are_preserved() {
    let compiler = Compiler::new();
    for alias in ["med", "prod"] {
        let upper = format!("PRINT {} OF (A LIST OF 2, 4).", alias.to_uppercase());
        let lower = format!("print {alias} of (a list of 2, 4).");
        assert_eq!(compiler.format_source(&upper).unwrap(), lower);
    }
    compiler.compile("let med be 9. let prod be 7. assert med is equal to 9. assert prod is equal to 7.", None).unwrap();
}

#[test]
fn official_example_runs() {
    let output = run(include_str!("../examples/compact_numeric_aliases.suri"));
    assert_eq!(output.status, 0, "{}", output.stderr);
    assert_eq!(output.stdout, "Compact median and product OK\n");
}

#[test]
fn vscode_collection_grammar_includes_new_aliases() {
    let grammar: serde_json::Value = serde_json::from_str(include_str!("../editors/vscode/syntaxes/semauri.tmLanguage.json")).unwrap();
    let functions = grammar["repository"]["collections"]["match"].as_str().unwrap();
    assert!(functions.contains("|med|prod"), "{functions}");
}
