use semauri::{run_cli, Compiler};
use std::{fs, sync::atomic::{AtomicU64, Ordering}};
static N: AtomicU64=AtomicU64::new(1);
fn run(source:&str)->semauri::CliResult {
    let path=std::env::temp_dir().join(format!("semauri-reduce-{}-{}.suri",std::process::id(),N.fetch_add(1,Ordering::Relaxed)));
    fs::write(&path,source).unwrap();
    let result=run_cli(&["run".into(),"--allow".into(),"console_write".into(),path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}
#[test]
fn reduce_sums_and_multiplies_with_typed_immutable_accumulators() {
    let r=run(r#"let values be a list of 2, 3, 4.
let total be reduce every item in values starting acc as 0 to (acc plus item).
let product be reduce every item in values starting acc as 1 to (acc times item).
assert total is equal to 9.
assert product is equal to 24.
print total.
print product.
print values."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"9\n24\n[2, 3, 4]\n");
}
#[test]
fn reduce_empty_list_returns_seed_and_single_item_executes_once() {
    let r=run(r#"let empty be take 0 from a list of 5, 6.
print reduce every x in empty starting total as 42 to (total plus x).
print reduce every x in (a list of 9) starting total as 1 to (total plus x)."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"42\n10\n");
}
#[test]
fn reduce_supports_boolean_strings_and_list_accumulators() {
    let r=run(r#"let values be a list of true, false, true.
print reduce every flag in values starting accumulated as true to (accumulated and flag).
let words be a list of "a", "b", "c".
print reduce every word in words starting text as "" to (text plus word).
let numbers be a list of 1, 2, 3.
let result be reduce every n in numbers starting kept as (take 0 from numbers) to (kept plus (a list of n)).
print result."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"false\nabc\n[1, 2, 3]\n");
}
#[test]
fn reduction_can_use_pure_functions_and_outer_lexical_bindings() {
    let r=run(r#"let offset be 2.
define function combine taking left as number, right as number to (left plus right plus offset).
let data be a list of 1, 2, 3.
let value be reduce every item in data starting acc as 0 to (invoke combine to (acc, item)).
print value.
print offset."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"12\n2\n");
}
#[test]
fn rejects_nonlist_wrong_step_type_and_duplicate_names() {
    for (source,code) in [
        ("let x be reduce every item in 2 starting acc as 0 to (acc plus item).","S379"),
        ("let x be reduce every item in (a list of 1, 2) starting acc as 0 to true.","S380"),
        ("let x be reduce every item in (a list of 1, 2) starting item as 0 to item.","S311"),
    ] {
        let e=Compiler::new().hir(source).unwrap_err();
        assert_eq!(e.code,code,"{source}: {e}");assert!(e.span.is_some());
    }
}
#[test]
fn iterator_and_accumulator_do_not_escape() {
    for name in ["item","acc"] {
        let source=format!("let value be reduce every item in (a list of 1, 2) starting acc as 0 to (acc plus item). print {name}.");
        assert_eq!(Compiler::new().hir(&source).unwrap_err().code,"S312");
    }
}
#[test]
fn keyword_is_contextual_and_optimized_hir_preserves_reduce_scope() {
    let r=run(r#"let reduce be 3.
let xs be a list of 1, 2.
print reduce.
print reduce every item in xs starting total as reduce to (total plus item)."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"3\n6\n");
    let c=Compiler::new();
    let src="let x be reduce every item in (a list of 1, 2) starting acc as 0 to (acc plus item). print x.";
    let hir=c.hir(src).unwrap().to_json().to_string();
    assert!(hir.contains("reduce_expression"),"{hir}");
    assert!(c.optimized_hir(src).unwrap().to_json().to_string().contains("reduce_expression"));
    assert_eq!(c.effect_analysis(src).unwrap().effects(),vec!["console_write".to_owned()]);
    assert_eq!(c.format_source("PRINT REDUCE EVERY item IN (a LIST OF 1, 2) STARTING total AS 0 TO (total PLUS item).").unwrap(),
        "print reduce every item in (a list of 1, 2) STARTING total AS 0 to (total plus item).");
}
