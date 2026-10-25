// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path=std::env::temp_dir().join(format!("semauri-flatten-{}-{}.sema",
        std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
    fs::write(&path,source).unwrap();
    let result=run_cli(&["run".into(),"--allow".into(),"console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn flattens_one_level_in_order_without_mutation() {
    let r=run(r#"let nested be a list of (a list of 1, 2), (a list of 3), (a list of 4, 5).
let merged be flatten of nested.
assert length of merged is equal to 5.
assert (merged at 0) is equal to 1.
assert (merged at 4) is equal to 5.
assert length of nested is equal to 3.
print merged."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[1, 2, 3, 4, 5]\n");
}

#[test]
fn composes_with_partition_and_preserves_typed_empty_lists() {
    let r=run(r#"let values be a list of 8, 2, 9, 1.
let groups be partition every n in values if (n is greater than 5).
let flattened be flatten of groups.
print flattened.
let empty be take 0 from (a list of 1, 2).
let groups2 be a list of empty, (a list of 3), empty.
let flat2 be flatten of groups2.
assert length of flat2 is equal to 1.
print flat2."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[8, 9, 2, 1]\n[3]\n");
}

#[test]
fn flatten_only_one_level_and_supports_strings_booleans() {
    let r=run(r#"let inner be a list of (a list of 1, 2), (a list of 3).
let deep be a list of inner, inner.
let once be flatten of deep.
let twice be flatten of once.
assert length of once is equal to 4.
assert length of twice is equal to 6.
let words be a list of (a list of "a", "b"), (a list of "c").
print flatten of words.
let flags be a list of (a list of true), (a list of false, true).
print flatten of flags."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[a, b, c]\n[true, false, true]\n");
}

#[test]
fn rejects_invalid_operands_with_static_diagnostic() {
    for src in [
        "print flatten of 4.",
        "print flatten of (a list of 1, 2).",
        "print flatten of \"hello\".",
    ] {
        let e=Compiler::new().hir(src).unwrap_err();
        assert_eq!(e.code,"S383","{src}: {e}");
        assert!(e.span.is_some());
    }
}

#[test]
fn flatten_keyword_is_contextual_and_preserves_identifiers() {
    let r=run(r#"let flatten be 9.
print flatten.
let nested be a list of (a list of 1), (a list of 2).
print flatten of nested."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"9\n[1, 2]\n");
}

#[test]
fn ast_hir_optimizer_formatter_and_effects_support_flatten() {
    let c=Compiler::new();
    let src="print flatten of (a list of (a list of 1), (a list of 2)).";
    assert!(c.parse(src).unwrap().to_json().to_string().contains("flatten"));
    assert!(c.hir(src).unwrap().to_json().to_string().contains("flatten"));
    assert!(c.optimized_hir(src).unwrap().to_json().to_string().contains("flatten"));
    assert_eq!(c.effect_analysis(src).unwrap().effects(),vec!["console_write".to_owned()]);
    assert_eq!(c.format_source("PRINT FLATTEN OF (a LIST OF (a LIST OF 1), (a LIST OF 2)).").unwrap(),
        "print flatten of (a list of (a list of 1), (a list of 2)).");
    assert_eq!(c.format_source("LET Flatten BE 3. PRINT Flatten.").unwrap(),
        "let Flatten be 3. print Flatten.");
}
