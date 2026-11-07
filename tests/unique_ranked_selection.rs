// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path=std::env::temp_dir().join(format!("semauri-unique-rank-{}-{}.suri",
        std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
    fs::write(&path,source).unwrap();
    let r=run_cli(&["run".into(),"--allow".into(),"console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    r
}
#[test]
fn four_spellings_select_distinct_extremes() {
    let r=run(r#"let values be a list of 5, 9, 5, 3, 9, 1.
print top unique 2 from values.
print top distinct 3 from values.
print bottom unique 2 from values.
print bottom distinct 3 from values.
print values."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,
        "[9, 5]\n[9, 5, 3]\n[1, 3]\n[1, 3, 5]\n[5, 9, 5, 3, 9, 1]\n");
}
#[test]
fn numeric_precision_booleans_strings_and_count_bounds() {
    let r=run(r#"print top distinct 2 from (a list of "é", "a", "é", "z").
print bottom unique 2 from (a list of true, false, true).
print top unique 2 from (a list of 9007199254740995, 9007199254740993, 9007199254740995).
print bottom distinct 0 from (a list of 3, 3, 1).
print top unique 99 from (a list of 4, 4, 2)."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[é, z]\n[false, true]\n[9007199254740995, 9007199254740993]\n[]\n[4, 2]\n");
}
#[test]
fn typed_diagnostics_match_distinct_and_take() {
    let c=Compiler::new();
    for src in ["print top unique 1 from 9.",
                "print bottom distinct 1 from \"abc\"."] {
        let err=c.hir(src).unwrap_err();
        assert_eq!(err.code,"S360","{src}: {err}");
        assert!(err.span.is_some());
    }
    for src in ["print top unique true from (a list of 1, 2).",
                "print bottom distinct \"2\" from (a list of 1, 2)."] {
        assert_eq!(c.hir(src).unwrap_err().code,"S363","{src}");
    }
    for src in ["print top unique minus 1 from (a list of 1, 2).",
                "print bottom distinct 1.5 from (a list of 1, 2)."] {
        let r=run(src);
        assert_eq!(r.status,65,"{src}: {}",r.stderr);
        assert!(r.stderr.contains("S364"),"{src}: {}",r.stderr);
    }
}
#[test]
fn formatter_optimizer_ast_effects_and_contextual_names() {
    let c=Compiler::new();
    for (src,expected) in [
        ("PRINT TOP UNIQUE 2 FROM (A LIST OF 3, 1).",
         "print top unique 2 from (a list of 3, 1)."),
        ("PRINT BOTTOM DISTINCT 2 FROM (A LIST OF 3, 1).",
         "print bottom distinct 2 from (a list of 3, 1)."),
    ] { assert_eq!(c.format_source(src).unwrap(),expected); }
    c.compile(r#"let unique be 2.
let xs be a list of 5, 9, 1.
assert (top unique from xs) is equal to (a list of 9, 5).
assert (bottom unique from xs) is equal to (a list of 1, 5)."#,None).unwrap();
    for src in ["print top unique 2 from (a list of 3, 1, 3).",
                "print bottom distinct 2 from (a list of 3, 1, 3)."] {
        assert!(c.optimized_hir(src).is_ok());
        assert_eq!(c.effect_analysis(src).unwrap().effects(),
            vec!["console_write".to_string()]);
    }
    let ast=c.parse("print top unique 2 from (a list of 3, 1, 3).")
        .unwrap().to_json().to_string();
    for op in ["distinct","sorted","reverse","take"] {
        assert!(ast.contains(op),"missing {op}: {ast}");
    }
}
#[test]
fn example_runs() {
    let r=run(include_str!("../examples/unique_ranked_selection.suri"));
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"Unique ranked selection OK\n");
}
