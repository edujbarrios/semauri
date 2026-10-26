// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64=AtomicU64::new(1);
fn run(source:&str)->semauri::CliResult{
    let path=std::env::temp_dir().join(format!("semauri-repeat-{}-{}.suri",
        std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
    fs::write(&path,source).unwrap();
    let r=run_cli(&["run".into(),"--allow".into(),"console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    r
}
#[test]
fn typed_repetition_creates_new_immutable_list(){
    let r=run(r#"let input be 7.
let data be repeat 3 of input.
assert length of data is equal to 3.
assert last of data is equal to 7.
assert input is equal to 7.
print data."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[7, 7, 7]\n");
}
#[test]
fn zero_creates_typed_empty_and_nested_lists(){
    let r=run(r#"let empty be repeat 0 of true.
let base be a list of 1, 2.
let copies be repeat 2 of base.
assert length of empty is equal to 0.
assert length of copies is equal to 2.
assert (first of copies at 0) is equal to 1.
print copies.
print empty."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[[1, 2], [1, 2]]\n[]\n");
}
#[test]
fn repeat_composes_with_maps_and_function_results(){
    let r=run(r#"define function double taking x as number to (x times 2).
let data be repeat (1 plus 2) of (invoke double to 5).
let result be map every n in data to (n plus 1).
print result."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[11, 11, 11]\n");
}
#[test]
fn reject_invalid_count_types(){
    let e=Compiler::new().hir("print repeat true of 4.").unwrap_err();
    assert_eq!(e.code,"S377");
    assert!(e.span.is_some());
}
#[test]
fn enforce_nonnegative_integer_and_bounded_count(){
    for src in [
        "print repeat minus 1 of 5.",
        "print repeat 1.25 of 5.",
        "print repeat 10001 of 5.",
    ] {
        let r=run(src);
        assert_ne!(r.status,0,"{src}");
        assert!(r.stderr.contains("S378"),"{src}: {}",r.stderr);
    }
}
#[test]
fn repeat_keyword_remains_usable_as_variable(){
    let r=run("let repeat be 6. print repeat.");
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"6\n");
}
#[test]
fn formatter_ast_hir_optimizer_and_effects_integrate_repeat(){
    let c=Compiler::new();
    let src="let xs be repeat 2 of 3. print xs.";
    assert!(c.parse(src).unwrap().to_json().to_string().contains("repeat_expression"));
    assert!(c.hir(src).unwrap().to_json().to_string().contains("repeat_expression"));
    assert!(c.optimized_hir(src).unwrap().to_json().to_string().contains("repeat_expression"));
    assert_eq!(c.effect_analysis(src).unwrap().effects(),vec!["console_write".to_owned()]);
    assert_eq!(c.format_source("PRINT REPEAT 2 OF 3.").unwrap(),"print repeat 2 of 3.");
}
