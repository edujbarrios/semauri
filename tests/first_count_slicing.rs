// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path=std::env::temp_dir().join(format!(
        "semauri-first-slice-{}-{}.suri",std::process::id(),
        NEXT.fetch_add(1,Ordering::Relaxed)));
    fs::write(&path,source).unwrap();
    let r=run_cli(&["run".into(),"--allow".into(),"console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    r
}
#[test]
fn prefix_slicing_preserves_order_and_inputs() {
    let r=run(r#"let xs be a list of 1, 2, 3, 4.
print take first 2 from xs.
print drop first 2 from xs.
print xs.
print take first 0 from xs.
print drop first 0 from xs.
print take first 99 from xs.
print drop first 99 from xs."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,
        "[1, 2]\n[3, 4]\n[1, 2, 3, 4]\n[]\n[1, 2, 3, 4]\n[1, 2, 3, 4]\n[]\n");
}
#[test]
fn unicode_and_contextual_variable_are_preserved() {
    let r=run(r#"print take first 2 from "ñ🌍a".
print drop first 2 from "ñ🌍a".
let first be 2.
let xs be a list of 5, 6, 7.
print take first from xs.
print drop first from xs.
print first of xs."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"ñ🌍\na\n[5, 6]\n[7]\n5\n");
}
#[test]
fn type_errors_and_invalid_counts_are_unchanged() {
    let c=Compiler::new();
    for src in [
        "print take first true from (a list of 1, 2).",
        "print drop first \"2\" from \"abc\".",
        "print take first 1 from false.",
        "print drop first 1 from 8.",
    ] {
        let err=c.hir(src).unwrap_err();
        assert_eq!(err.code,"S363","{src}: {err}");
        assert!(err.span.is_some());
    }
    for src in [
        "print take first minus 1 from (a list of 1, 2).",
        "print drop first 1.5 from \"abc\".",
    ] {
        let r=run(src);
        assert_eq!(r.status,65,"{src}: {}",r.stderr);
        assert!(r.stderr.contains("S364"),"{src}: {}",r.stderr);
    }
}
#[test]
fn formatting_optimizer_and_ast() {
    let c=Compiler::new();
    for (s,expected) in [
        ("PRINT TAKE FIRST 2 FROM (A LIST OF 3, 1).",
         "print take first 2 from (a list of 3, 1)."),
        ("PRINT DROP FIRST 2 FROM \"ñ🌍a\".",
         "print drop first 2 from \"ñ🌍a\"."),
    ] { assert_eq!(c.format_source(s).unwrap(),expected); }
    assert_eq!(c.format_source("LET First BE 2.").unwrap(),"let First be 2.");
    for s in [
        "print take first 2 from (a list of 1, 2, 3).",
        "print drop first 2 from \"abc\".",
    ] {
        assert!(c.optimized_hir(s).is_ok());
        assert_eq!(c.effect_analysis(s).unwrap().effects(),vec!["console_write".to_string()]);
    }
    let ast=c.parse("print take first 2 from (a list of 1, 2).")
        .unwrap().to_json().to_string();
    assert!(ast.contains("take"),"{ast}");
}
#[test]
fn example_executes() {
    let r=run(include_str!("../examples/first_count_slicing.suri"));
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"First-count slicing OK\n");
}
