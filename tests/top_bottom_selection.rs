// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path=std::env::temp_dir().join(format!(
        "semauri-top-bottom-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result=run_cli(&["run".into(),"--allow".into(),"console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}
#[test]
fn selects_ordered_extremes_without_mutating_the_source() {
    let result=run(r#"let items be a list of 9, 3, 8, 1.
print top 2 from items.
print bottom 3 from items.
print items.
print top 0 from items.
print bottom 10 from items."#);
    assert_eq!(result.status,0,"{}",result.stderr);
    assert_eq!(result.stdout,
        "[9, 8]\n[1, 3, 8]\n[9, 3, 8, 1]\n[]\n[1, 3, 8, 9]\n");
}
#[test]
fn preserves_typed_sorting_of_unicode_boolean_and_exact_integer_values() {
    let result=run(r#"print top 2 from (a list of "é", "a", "z").
print bottom 2 from (a list of true, false, true).
print top 2 from (a list of 9007199254740995, 9007199254740992.0, 9007199254740993)."#);
    assert_eq!(result.status,0,"{}",result.stderr);
    assert_eq!(result.stdout,
        "[é, z]\n[false, true]\n[9007199254740995, 9007199254740993]\n");
}
#[test]
fn static_type_errors_match_checked_sorted_and_take() {
    let compiler=Compiler::new();
    for source in [
        "print top 2 from 12.",
        "print bottom 1 from \"abc\".",
    ] {
        let error=compiler.hir(source).unwrap_err();
        assert_eq!(error.code,"S359","{source}: {error}");
        assert!(error.span.is_some());
    }
    for source in [
        "print top true from (a list of 1, 2).",
        "print bottom \"2\" from (a list of 1, 2).",
    ] {
        let error=compiler.hir(source).unwrap_err();
        assert_eq!(error.code,"S363","{source}: {error}");
    }
}
#[test]
fn counts_must_be_nonnegative_integral_values() {
    for source in [
        "print top minus 1 from (a list of 1, 2).",
        "print bottom 1.5 from (a list of 1, 2).",
    ] {
        let result=run(source);
        assert_eq!(result.status,65,"{source}: {}",result.stderr);
        assert!(result.stderr.contains("S364"),"{source}: {}",result.stderr);
    }
}
#[test]
fn handles_contextual_names_formatter_and_compiler_passes() {
    let compiler=Compiler::new();
    for (source,expected) in [
        ("PRINT TOP 2 FROM (A LIST OF 3, 2).",
         "print top 2 from (a list of 3, 2)."),
        ("PRINT BOTTOM 1 FROM (A LIST OF 3, 2).",
         "print bottom 1 from (a list of 3, 2)."),
    ] { assert_eq!(compiler.format_source(source).unwrap(),expected); }
    compiler.compile(r#"let top be 10.
let bottom be 3.
assert top is greater than bottom."#,None).unwrap();
    assert_eq!(compiler.format_source(
        "LET Top BE 10. LET Bottom BE 3.").unwrap(),
        "let Top be 10. let Bottom be 3.");
    for source in [
        "print top 2 from (a list of 3, 1, 2).",
        "print bottom 2 from (a list of 3, 1, 2).",
    ] {
        assert!(compiler.optimized_hir(source).is_ok(),"{source}");
        assert_eq!(compiler.effect_analysis(source).unwrap().effects(),
            vec!["console_write".to_owned()]);
    }
    let ast=compiler.parse("print top 2 from (a list of 3, 1, 2).")
        .unwrap().to_json().to_string();
    assert!(ast.contains("sorted") && ast.contains("reverse")
        && ast.contains("take"),"{ast}");
}
#[test]
fn reference_example_executes() {
    let result=run(include_str!("../examples/top_bottom_selection.suri"));
    assert_eq!(result.status,0,"{}",result.stderr);
    assert_eq!(result.stdout,"Top and bottom selection OK\n");
}
#[test]
fn vscode_operator_grammar_supports_selection() {
    let grammar:serde_json::Value=serde_json::from_str(include_str!(
        "../editors/vscode/syntaxes/semauri.tmLanguage.json")).unwrap();
    let functions=grammar["repository"]["collections"]["match"].as_str().unwrap();
    assert!(functions.contains("top") && functions.contains("bottom"));
}
