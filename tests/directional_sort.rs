// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path=std::env::temp_dir().join(format!(
        "semauri-directional-sort-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result=run_cli(&["run".into(),"--allow".into(),
        "console_write".into(),path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}
#[test]
fn explicit_sort_directions_preserve_inputs_and_compatibility() {
    let result=run(r#"let nums be a list of 3, 1, 2.
print sorted ascending of nums.
print sorted descending of nums.
print sort ascending of nums.
print sort descending of nums.
print sorted of nums.
print sort of nums.
print nums."#);
    assert_eq!(result.status,0,"{}",result.stderr);
    assert_eq!(result.stdout,
        "[1, 2, 3]\n[3, 2, 1]\n[1, 2, 3]\n[3, 2, 1]\n[1, 2, 3]\n[1, 2, 3]\n[3, 1, 2]\n");
}
#[test]
fn sorts_text_flags_and_exact_large_integers() {
    let result=run(r#"print sorted descending of (a list of "Apple", "é", "z").
print sort descending of (a list of false, true, false).
print sorted descending of (a list of 9007199254740993, 9007199254740995, 9007199254740992.0)."#);
    assert_eq!(result.status,0,"{}",result.stderr);
    assert_eq!(result.stdout,
        "[é, z, Apple]\n[true, false, false]\n[9007199254740995, 9007199254740993, 9007199254740992.0]\n");
}
#[test]
fn statically_rejects_invalid_inputs_with_existing_diagnostic() {
    let compiler=Compiler::new();
    for operation in ["sorted ascending", "sorted descending",
                      "sort ascending", "sort descending"] {
        for value in ["17", r#""text""#] {
            let source=format!("print {operation} of {value}.");
            let error=compiler.hir(&source).unwrap_err();
            assert_eq!(error.code,"S359","{source}: {error}");
            assert!(error.span.is_some());
        }
    }
}
#[test]
fn formatting_optimizing_and_contextual_identifiers() {
    let compiler=Compiler::new();
    for (source,expected) in [
        ("PRINT SORTED DESCENDING OF (A LIST OF 3, 1).",
         "print sorted descending of (a list of 3, 1)."),
        ("PRINT SORT ASCENDING OF (A LIST OF 3, 1).",
         "print sort ascending of (a list of 3, 1)."),
        ("PRINT SORT DESCENDING OF (A LIST OF 3, 1).",
         "print sort descending of (a list of 3, 1)."),
    ] { assert_eq!(compiler.format_source(source).unwrap(),expected); }
    let source=r#"let ascending be 11.
let descending be 9.
assert ascending is greater than descending."#;
    compiler.compile(source,None).unwrap();
    assert_eq!(compiler.format_source(
        "LET Ascending BE 11. LET Descending BE 9.").unwrap(),
        "let Ascending be 11. let Descending be 9.");
    for expression in ["print sorted ascending of (a list of 2, 1).",
                       "print sort descending of (a list of 2, 1)."] {
        assert!(compiler.optimized_hir(expression).is_ok(),"{expression}");
        assert_eq!(compiler.effect_analysis(expression).unwrap().effects(),
            vec!["console_write".to_string()]);
    }
    let tree=compiler.parse("print sorted descending of (a list of 2, 1).")
        .unwrap().to_json().to_string();
    assert!(tree.contains("reverse") && tree.contains("sorted"),"{tree}");
}
#[test]
fn documented_example_executes() {
    let result=run(include_str!("../examples/directional_sort.suri"));
    assert_eq!(result.status,0,"{}",result.stderr);
    assert_eq!(result.stdout,"Directional sorting OK\n");
}
#[test]
fn syntax_grammar_mentions_direction_words() {
    let grammar:serde_json::Value=serde_json::from_str(include_str!(
        "../editors/vscode/syntaxes/semauri.tmLanguage.json")).unwrap();
    let operations=grammar["repository"]["collections"]["match"].as_str().unwrap();
    assert!(operations.contains("ascending"));
    assert!(operations.contains("descending"));
}
