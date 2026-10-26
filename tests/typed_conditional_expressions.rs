// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!("semauri-conditional-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result=run_cli(&["run".into(),"--allow".into(),"console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}
#[test]
fn selects_typed_branches_and_composes_with_functions() {
    let r=run(r#"define function magnitude taking n as number to (if (n is less than 0) then (minus n) otherwise n).
let numbers be a list of minus 4, 1, 0, minus 8.
let magnitudes be map every n in numbers to (invoke magnitude to n).
assert (magnitudes at 0) is equal to 4.
assert (magnitudes at 3) is equal to 8.
print magnitudes."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[4, 1, 0, 8]\n");
}
#[test]
fn short_circuits_unevaluated_branch() {
    let r=run(r#"let x be if true then 42 otherwise (1 divided by 0).
let y be if false then (1 divided by 0) otherwise 7.
assert x is equal to 42.
print y."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"7\n");
}
#[test]
fn supports_strings_and_lists_and_nested_branches() {
    let r=run(r#"let text be if true then "pass" otherwise "fail".
let xs be if false then (a list of 2, 1) otherwise (a list of 7, 9).
let nested be if false then 1 otherwise (if true then 6 otherwise 7).
print text.
print xs.
print nested."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"pass\n[7, 9]\n6\n");
}
#[test]
fn rejects_nonboolean_predicate_or_mismatched_branches() {
    for (src,code) in [
        ("let x be if 3 then 1 otherwise 2.","S375"),
        ("let x be if true then 1 otherwise false.","S376"),
        ("let x be if true then (a list of 1) otherwise (a list of true).","S376"),
    ] {
        let e=Compiler::new().hir(src).unwrap_err();
        assert_eq!(e.code,code,"{src}: {e}");
        assert!(e.span.is_some());
    }
}
#[test]
fn statement_if_syntax_remains_compatible() {
    let r=run("if true: print 5. otherwise: print 9. end.");
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"5\n");
}
#[test]
fn formatter_ast_hir_effects_and_optimizer_support_conditional() {
    let c=Compiler::new();
    let src="let x be if true then 2 otherwise 3. print x.";
    assert!(c.parse(src).unwrap().to_json().to_string().contains("conditional_expression"));
    assert!(c.hir(src).unwrap().to_json().to_string().contains("conditional_expression"));
    assert_eq!(c.effect_analysis(src).unwrap().effects(),vec!["console_write".to_owned()]);
    assert_eq!(c.format_source("LET x BE IF true THEN 2 OTHERWISE 3.").unwrap(),
        "let x be if true then 2 otherwise 3.");
    assert!(c.optimized_hir(src).unwrap().to_json().to_string().contains("literal"));
}
