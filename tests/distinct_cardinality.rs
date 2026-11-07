// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path=std::env::temp_dir().join(format!(
        "semauri-distinct-cardinality-{}-{}.suri",std::process::id(),
        NEXT.fetch_add(1,Ordering::Relaxed)));
    fs::write(&path,source).unwrap();
    let result=run_cli(&["run".into(),"--allow".into(),"console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}
#[test]
fn equivalent_cardinality_spellings_count_distinct_values() {
    let result=run(r#"let values be a list of 3, 1, 3, 2, 1, 3.
print cardinality of values.
print count distinct of values.
print count unique of values.
print length of values.
print values."#);
    assert_eq!(result.status,0,"{}",result.stderr);
    assert_eq!(result.stdout,"3\n3\n3\n6\n[3, 1, 3, 2, 1, 3]\n");
}
#[test]
fn supports_boolean_text_colors_and_storage_aware_numbers() {
    let result=run(r#"print cardinality of (a list of "hi", "hi", "bye").
print count unique of (a list of true, false, true).
print count distinct of (a list of red, blue, red).
print cardinality of (a list of 1, 1.0, 1, 1.0).
print cardinality of (take 0 from (a list of 1, 2))."#);
    assert_eq!(result.status,0,"{}",result.stderr);
    assert_eq!(result.stdout,"2\n2\n2\n2\n0\n");
}
#[test]
fn reuses_checked_distinct_list_diagnostic() {
    let compiler=Compiler::new();
    for form in ["cardinality of","count distinct of","count unique of"] {
        for value in ["42",r#""hello""#,"true"] {
            let source=format!("print {form} {value}.");
            let e=compiler.hir(&source).unwrap_err();
            assert_eq!(e.code,"S360","{source}: {e}");
            assert!(e.span.is_some());
        }
    }
}
#[test]
fn formatter_contextual_names_and_optimized_hir() {
    let compiler=Compiler::new();
    for (source,expected) in [
        ("PRINT CARDINALITY OF (A LIST OF 1, 2, 2).",
         "print cardinality of (a list of 1, 2, 2)."),
        ("PRINT COUNT DISTINCT OF (A LIST OF 1, 2, 2).",
         "print count distinct of (a list of 1, 2, 2)."),
        ("PRINT COUNT UNIQUE OF (A LIST OF 1, 2, 2).",
         "print count unique of (a list of 1, 2, 2)."),
    ] { assert_eq!(compiler.format_source(source).unwrap(),expected); }
    compiler.compile(r#"let cardinality be 7.
let count be 8.
let unique be 9.
assert cardinality is equal to 7.
assert count is equal to 8.
assert unique is equal to 9."#,None).unwrap();
    assert_eq!(compiler.format_source(
        "LET Cardinality BE 7. LET Count BE 8. LET Unique BE 9.").unwrap(),
        "let Cardinality be 7. let Count be 8. let Unique be 9.");
    for source in [
        "print cardinality of (a list of 1, 1, 2).",
        "print count distinct of (a list of 1, 1, 2).",
        "print count unique of (a list of 1, 1, 2).",
    ] {
        assert!(compiler.optimized_hir(source).is_ok(),"{source}");
        assert_eq!(compiler.effect_analysis(source).unwrap().effects(),
            vec!["console_write".to_string()]);
    }
    let ast=compiler.parse("print count distinct of (a list of 1, 2, 1).")
        .unwrap().to_json().to_string();
    assert!(ast.contains("distinct") && ast.contains("length"),"{ast}");
}
#[test]
fn reference_example_runs() {
    let result=run(include_str!("../examples/unique_cardinality.suri"));
    assert_eq!(result.status,0,"{}",result.stderr);
    assert_eq!(result.stdout,"Distinct cardinality OK\n");
}
#[test]
fn editor_collection_grammar_highlights_cardinality() {
    let grammar:serde_json::Value=serde_json::from_str(include_str!(
        "../editors/vscode/syntaxes/semauri.tmLanguage.json")).unwrap();
    let collection=grammar["repository"]["collections"]["match"].as_str().unwrap();
    assert!(collection.contains("cardinality"));
}
