// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path=std::env::temp_dir().join(format!(
        "semauri-suffix-slice-{}-{}.suri",std::process::id(),
        NEXT.fetch_add(1,Ordering::Relaxed)));
    fs::write(&path,source).unwrap();
    let r=run_cli(&["run".into(),"--allow".into(),"console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    r
}
#[test]
fn suffix_slicing_preserves_order_and_input() {
    let r=run(r#"let numbers be a list of 1, 2, 3, 4.
print take last 2 from numbers.
print drop last 2 from numbers.
print numbers.
print take last 0 from numbers.
print drop last 0 from numbers.
print take last 10 from numbers.
print drop last 10 from numbers."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[3, 4]\n[1, 2]\n[1, 2, 3, 4]\n[]\n[1, 2, 3, 4]\n[1, 2, 3, 4]\n[]\n");
}
#[test]
fn unicode_scalars_and_typed_values() {
    let r=run(r#"print take last 2 from "ñ🌍a".
print drop last 2 from "ñ🌍a".
print take last 10 from "a🍵".
print drop last 10 from "a🍵".
print take last 1 from (a list of true, false, true).
print drop last 1 from (a list of red, blue, red)."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"🌍a\nñ\na🍵\n\n[true]\n[red, blue]\n");
}
#[test]
fn rejects_invalid_counts_and_source_types() {
    let compiler=Compiler::new();
    for src in [
        "print take last true from (a list of 1, 2).",
        "print drop last \"2\" from \"abc\".",
    ] {
        let err=compiler.hir(src).unwrap_err();
        assert_eq!(err.code,"S363","{src}: {err}");
        assert!(err.span.is_some());
    }
    for src in ["print take last 1 from true.",
                "print drop last 1 from 14."] {
        let err=compiler.hir(src).unwrap_err();
        assert_eq!(err.code,"S352","{src}: {err}");
    }
    for src in [
        "print take last minus 1 from (a list of 1, 2).",
        "print drop last 1.5 from \"abc\".",
    ] {
        let r=run(src);
        assert_eq!(r.status,65,"{src}: {}",r.stderr);
        assert!(r.stderr.contains("S364"),"{src}: {}",r.stderr);
    }
}
#[test]
fn contextual_identifiers_and_old_take_drop_still_work() {
    let r=run(r#"let last be 2.
let numbers be a list of 5, 6, 7, 8.
print take last from numbers.
print drop last from numbers.
print last of numbers."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[5, 6]\n[7, 8]\n8\n");
}
#[test]
fn formatting_ast_optimizer_and_effect_analysis() {
    let c=Compiler::new();
    for (source,expected) in [
        ("PRINT TAKE LAST 2 FROM \"ñ🌍a\".","print take last 2 from \"ñ🌍a\"."),
        ("PRINT DROP LAST 1 FROM (A LIST OF 1, 2).",
         "print drop last 1 from (a list of 1, 2)."),
    ] {assert_eq!(c.format_source(source).unwrap(),expected);}
    assert_eq!(c.format_source("LET Last BE 2.").unwrap(),"let Last be 2.");
    for source in [
        "print take last 2 from (a list of 1, 2, 3).",
        "print drop last 2 from \"abc\".",
    ] {
        let optimized=c.optimized_hir(source).unwrap();
        assert!(!optimized.to_json().to_string().is_empty());
        assert_eq!(c.effect_analysis(source).unwrap().effects(),
            vec!["console_write".to_string()]);
    }
    let ast=c.parse("print take last 2 from (a list of 1, 2, 3).")
        .unwrap().to_json().to_string();
    assert!(ast.contains("take") && ast.contains("reverse"),"{ast}");
}
#[test]
fn example_runs() {
    let r=run(include_str!("../examples/suffix_slicing.suri"));
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"Suffix slicing OK\n");
}
