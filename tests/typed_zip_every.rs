// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!("semauri-zip-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let r = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    r
}
#[test]
fn adds_pairwise_and_preserves_source() {
    let r=run(r#"let xs be a list of 1, 2, 3.
let ys be a list of 10, 20, 30.
let out be zip every x in xs with y in ys to (x plus y).
assert length of out is equal to 3.
print out.
print xs."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[11, 22, 33]\n[1, 2, 3]\n");
}
#[test]
fn truncates_to_the_shorter_list_either_way() {
    let r=run(r#"let xs be a list of 1, 2.
let ys be a list of 10, 20, 30.
print zip every x in xs with y in ys to (x plus y).
print zip every x in ys with y in xs to (x plus y)."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[11, 22]\n[11, 22]\n");
}
#[test]
fn supports_different_input_types() {
    let r=run(r#"let xs be a list of 1, 2.
let labels be a list of "one", "two".
let flags be a list of true, false.
print zip every n in xs with label in labels to label.
print zip every n in xs with flag in flags to flag."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[one, two]\n[true, false]\n");
}
#[test]
fn works_with_user_functions_and_map() {
    let r=run(r#"define function weighted taking score as number, weight as number to (score times weight).
let scores be range from 1 to 3.
let weights be a list of 2, 4, 6.
let products be zip every score in scores with weight in weights to (invoke weighted to (score, weight)).
print map every n in products to (n plus 1)."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[3, 9, 19]\n");
}
#[test]
fn handles_typed_empty_input() {
    let r=run(r#"let xs be take 0 from a list of 1, 2.
let ys be a list of 9, 8.
let out be zip every x in xs with y in ys to (x plus y).
assert length of out is equal to 0.
print out."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[]\n");
}
#[test]
fn reports_invalid_types_and_duplicate_names() {
    for (source,code) in [
        ("print zip every x in true with y in (a list of 1, 2) to x.","S383"),
        ("print zip every x in (a list of 1, 2) with y in false to x.","S383"),
        ("print zip every x in (a list of 1, 2) with x in (a list of 3, 4) to x.","S311"),
    ] {
        let err=Compiler::new().hir(source).unwrap_err();
        assert_eq!(err.code,code,"{source}: {err}");
    }
}
#[test]
fn lexical_shadowing_and_no_leak() {
    let r=run(r#"let x be 100.
let xs be a list of 1, 2.
let ys be a list of 10, 20.
print zip every x in xs with y in ys to (x plus y).
print x."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[11, 22]\n100\n");
    let e=Compiler::new().hir("let xs be a list of 1. let ys be a list of 2. let result be zip every x in xs with y in ys to x. print y.").unwrap_err();
    assert_eq!(e.code,"S312");
}
#[test]
fn formatter_hir_optimizer_and_effects_work() {
    let c=Compiler::new();
    let src="let xs be a list of 1, 2. let ys be a list of 3, 4. print zip every x in xs with y in ys to (x plus y).";
    assert!(c.parse(src).unwrap().to_json().to_string().contains("zip_transform"));
    assert!(c.hir(src).unwrap().to_json().to_string().contains("zip_transform"));
    assert!(c.optimized_hir(src).unwrap().to_json().to_string().contains("zip_transform"));
    assert_eq!(c.effect_analysis(src).unwrap().effects(),vec!["console_write".to_owned()]);
    assert_eq!(c.format_source("PRINT ZIP EVERY x IN xs with y IN ys TO (x PLUS y).").unwrap(),
        "print zip every x in xs with y in ys to (x plus y).");
    assert_eq!(c.format_source("LET Zip BE 3. PRINT Zip.").unwrap(),"let Zip be 3. print Zip.");
}
