// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path=std::env::temp_dir().join(format!(
        "semauri-concise-list-{}-{}.suri",std::process::id(),
        NEXT.fetch_add(1,Ordering::Relaxed)));
    fs::write(&path,source).unwrap();
    let r=run_cli(&["run".into(),"--allow".into(),"console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    r
}
#[test]
fn list_and_text_shorthands_preserve_inputs_and_behavior() {
    let r=run(r#"let values be a list of 3, 1, 2.
print rev of values.
print sort of values.
print values.
print rev of "ñ🌍a".
let nested be a list of (a list of 1, 2), (a list of 3).
print flat of nested."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[2, 1, 3]\n[1, 2, 3]\n[3, 1, 2]\na🌍ñ\n[1, 2, 3]\n");
}
#[test]
fn strict_types_share_original_diagnostics() {
    let c=Compiler::new();
    for (short, original, value) in [
        ("rev","reverse","9"),
        ("sort","sorted",r#""abc""#),
        ("flat","flatten","true"),
    ] {
        let a=c.hir(&format!("print {short} of {value}.")).unwrap_err();
        let b=c.hir(&format!("print {original} of {value}.")).unwrap_err();
        assert_eq!(a.code,b.code,"{short} should keep {original} checks");
        assert!(a.span.is_some());
    }
}
#[test]
fn preserves_precision_and_one_level_flattening() {
    let r=run(r#"let values be a list of 9007199254740995, 9007199254740993.
print sort of values.
let sub be a list of (a list of 1), (a list of 2).
let deep be a list of sub, sub.
print size of (flat of deep).
print size of (flat of (flat of deep))."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[9007199254740993, 9007199254740995]\n4\n4\n");
}
#[test]
fn formatting_contextual_identifiers_optimizer_and_effects() {
    let c=Compiler::new();
    for (source,expected) in [
        ("PRINT REV OF \"abc\".","print rev of \"abc\"."),
        ("PRINT SORT OF (A LIST OF 3, 2).","print sort of (a list of 3, 2)."),
        ("PRINT FLAT OF (A LIST OF (A LIST OF 2)).",
         "print flat of (a list of (a list of 2))."),
    ] { assert_eq!(c.format_source(source).unwrap(),expected); }
    c.compile(r#"let rev be 9.
let sort be 7.
let flat be 3.
assert rev is equal to 9.
assert sort is equal to 7.
assert flat is equal to 3."#,None).unwrap();
    for expression in [
        "print rev of \"abc\".",
        "print sort of (a list of 3, 1).",
        "print flat of (a list of (a list of 1), (a list of 2)).",
    ] {
        assert!(c.optimized_hir(expression).is_ok(),"{expression}");
        assert_eq!(c.effect_analysis(expression).unwrap().effects(),
            vec!["console_write".to_string()]);
    }
    let ast=c.parse("print rev of \"abc\".").unwrap().to_json().to_string();
    assert!(ast.contains("reverse"),"{ast}");
}
#[test]
fn official_example_runs() {
    let r=run(include_str!("../examples/concise_list_operators.suri"));
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"Concise list operators OK\n");
}
#[test]
fn vscode_collection_grammar_includes_aliases() {
    let grammar:serde_json::Value=serde_json::from_str(include_str!(
        "../editors/vscode/syntaxes/semauri.tmLanguage.json")).unwrap();
    let functions=grammar["repository"]["collections"]["match"].as_str().unwrap();
    for word in ["rev","sort","flat"] {
        assert!(functions.contains(word),"missing {word}");
    }
}
