// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path=std::env::temp_dir().join(format!(
        "semauri-sorted-unique-{}-{}.suri",std::process::id(),
        NEXT.fetch_add(1,Ordering::Relaxed)));
    fs::write(&path,source).unwrap();
    let r=run_cli(&["run".into(),"--allow".into(),"console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    r
}
#[test]
fn all_spellings_deduplicate_then_sort_without_mutating_input() {
    let r=run(r#"let values be a list of 3, 1, 3, 2, 1.
print sorted distinct of values.
print sorted unique of values.
print sort distinct of values.
print sort unique of values.
print values."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[1, 2, 3]\n".repeat(4)+"[3, 1, 3, 2, 1]\n");
}
#[test]
fn unicode_bool_and_empty_lists_work() {
    let r=run(r#"print sorted distinct of (a list of "é", "a", "é", "z").
print sort unique of (a list of true, false, true).
print sorted unique of (take 0 from (a list of 1, 2))."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[a, z, é]\n[false, true]\n[]\n");
}
#[test]
fn exact_large_number_sorting_and_distinct_storage() {
    let r=run(r#"print sorted distinct of (a list of 9007199254740995, 9007199254740993, 9007199254740995).
print cardinality of (sorted unique of (a list of 1, 1.0, 1, 1.0))."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[9007199254740993, 9007199254740995]\n2\n");
}
#[test]
fn typed_errors_preserve_existing_codes() {
    let c=Compiler::new();
    for form in ["sorted distinct", "sorted unique", "sort distinct", "sort unique"] {
        for input in ["42",r#""abc""#,"false"] {
            let src=format!("print {form} of {input}.");
            let err=c.hir(&src).unwrap_err();
            assert_eq!(err.code,"S360","{src}: {err}");
            assert!(err.span.is_some());
        }
    }
}
#[test]
fn formatting_identifiers_optimizer_and_effects() {
    let c=Compiler::new();
    for (src,expected) in [
        ("PRINT SORTED DISTINCT OF (A LIST OF 3, 1).",
         "print sorted distinct of (a list of 3, 1)."),
        ("PRINT SORTED UNIQUE OF (A LIST OF 3, 1).",
         "print sorted unique of (a list of 3, 1)."),
        ("PRINT SORT DISTINCT OF (A LIST OF 3, 1).",
         "print sort distinct of (a list of 3, 1)."),
        ("PRINT SORT UNIQUE OF (A LIST OF 3, 1).",
         "print sort unique of (a list of 3, 1)."),
    ] { assert_eq!(c.format_source(src).unwrap(),expected); }
    c.compile(r#"let sort be 9.
let unique be 4.
assert sort is greater than unique."#,None).unwrap();
    assert_eq!(c.format_source("LET Sort BE 2. LET Unique BE 3.").unwrap(),
        "let Sort be 2. let Unique be 3.");
    for src in [
        "print sorted unique of (a list of 3, 1, 3).",
        "print sort distinct of (a list of 3, 1, 3).",
    ] {
        assert!(c.optimized_hir(src).is_ok());
        assert_eq!(c.effect_analysis(src).unwrap().effects(),
            vec!["console_write".to_owned()]);
    }
    let ast=c.parse("print sort unique of (a list of 3, 1, 3).")
        .unwrap().to_json().to_string();
    assert!(ast.contains("sorted")&&ast.contains("distinct"),"{ast}");
}
#[test]
fn official_example_runs() {
    let r=run(include_str!("../examples/sorted_unique.suri"));
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"Sorted unique OK\n");
}
