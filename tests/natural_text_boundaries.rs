// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path=std::env::temp_dir().join(format!(
        "semauri-text-boundaries-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result=run_cli(&["run".into(),"--allow".into(),"console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}
#[test]
fn positive_and_negative_unicode_text_boundaries_work() {
    let r=run(r#"let word be "Café Ñandú".
print word begins with "Café".
print word finishes with "Ñandú".
print word begins with "café".
print word finishes with "ñandú".
print word does not begin with "café".
print word does not finish with "ñandú".
print word does not begin with "Café".
print word does not finish with "Ñandú".
print "" begins with "".
print "" finishes with "".
print "" does not begin with "".
print "" does not finish with ""."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,
        "true\ntrue\nfalse\nfalse\ntrue\ntrue\nfalse\nfalse\ntrue\ntrue\nfalse\nfalse\n");
}
#[test]
fn invalid_operands_keep_checked_prefix_suffix_diagnostics() {
    let compiler=Compiler::new();
    for src in [
        r#"print 5 begins with "5"."#,
        r#"print "alpha" finishes with 4."#,
        r#"print (a list of 1, 2) does not begin with "one"."#,
        r#"print false does not finish with "semauri"."#,
    ] {
        let error=compiler.hir(src).unwrap_err();
        assert_eq!(error.code,"S391","{src}: {error}");
        assert!(error.span.is_some());
    }
    for src in [
        r#"print "alpha" does not begin "a"."#,
        r#"print "alpha" does not finish "a"."#,
    ] {
        let error=compiler.parse(src).unwrap_err();
        assert_eq!(error.code,"S225","{src}: {error}");
    }
}
#[test]
fn formatter_identifiers_ast_optimizer_and_effects() {
    let compiler=Compiler::new();
    for (source, expected) in [
        (r#"PRINT "abc" BEGINS WITH "a"."#,r#"print "abc" begins with "a"."#),
        (r#"PRINT "abc" FINISHES WITH "c"."#,r#"print "abc" finishes with "c"."#),
        (r#"PRINT "abc" DOES NOT BEGIN WITH "z"."#,r#"print "abc" does not begin with "z"."#),
        (r#"PRINT "abc" DOES NOT FINISH WITH "z"."#,r#"print "abc" does not finish with "z"."#),
    ] { assert_eq!(compiler.format_source(source).unwrap(),expected); }
    compiler.compile(r#"let begins be "report.txt".
let finishes be ".txt".
assert begins finishes with finishes.
assert begins begins with "report".
let begin be 1.
let finish be 2.
assert begin is less than finish."#,None).unwrap();
    let ast=compiler.parse(r#"print "abc" does not finish with "z"."#)
        .unwrap().to_json().to_string();
    assert!(ast.contains("ends_with") && ast.contains("not"),"{ast}");
    assert!(compiler.optimized_hir(r#"print "abc" begins with "a"."#).is_ok());
    assert_eq!(compiler.effect_analysis(r#"print "abc" does not begin with "z"."#)
        .unwrap().effects(),vec!["console_write".to_string()]);
}
#[test]
fn example_runs_without_new_capabilities() {
    let result=run(include_str!("../examples/natural_text_boundaries.suri"));
    assert_eq!(result.status,0,"{}",result.stderr);
    assert_eq!(result.stdout,"Natural text boundaries OK\n");
}
#[test]
fn vscode_grammar_highlights_boundary_words() {
    let grammar:serde_json::Value=serde_json::from_str(include_str!(
        "../editors/vscode/syntaxes/semauri.tmLanguage.json")).unwrap();
    let keywords=grammar["repository"]["operators"]["match"].as_str().unwrap();
    for word in ["begins","finishes","begin","finish"] {
        assert!(keywords.contains(word),"missing {word}");
    }
}
