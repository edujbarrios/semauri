use semauri::{run_cli, Compiler};
use std::{fs,sync::atomic::{AtomicU64,Ordering}};
static NEXT:AtomicU64=AtomicU64::new(1);
fn run(source:&str)->semauri::CliResult {
    let path=std::env::temp_dir().join(format!("semauri-quantifiers-{}-{}.suri",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
    fs::write(&path,source).unwrap();
    let r=run_cli(&["run".into(),"--allow".into(),"console_write".into(),path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();r
}
#[test]
fn checks_any_and_all_over_numeric_predicates() {
    let r=run(r#"let values be a list of 2, 4, 6.
print any every n in values if (n is greater than 5).
print all every n in values if ((n modulo 2) is equal to 0).
print any every n in values if (n is less than 0).
print all every n in values if (n is greater than 3)."#);
    assert_eq!(r.status,0,"{}",r.stderr); assert_eq!(r.stdout,"true\ntrue\nfalse\nfalse\n");
}
#[test]
fn quantifiers_are_vacuous_on_typed_empty_lists() {
    let r=run(r#"let empty be take 0 from a list of 3, 4.
print any every n in empty if true.
print all every n in empty if false.
print any of (a list of false, false).
print all of (a list of true, true)."#);
    assert_eq!(r.status,0,"{}",r.stderr);assert_eq!(r.stdout,"false\ntrue\nfalse\ntrue\n");
}
#[test]
fn quantified_predicates_short_circuit_unevaluated_tail() {
    let r=run(r#"let numbers be a list of 1, 2.
print any every n in numbers if (if n is equal to 1 then true otherwise ((1 divided by 0) is equal to 7)).
print all every n in numbers if (if n is equal to 1 then false otherwise ((1 divided by 0) is equal to 7))."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"true\nfalse\n");
}
#[test]
fn quantifiers_use_pure_functions_and_preserve_lexical_shadowing() {
    let r=run(r#"let threshold be 2.
define function passes taking value as number, limit as number to (value is greater than limit).
let samples be a list of 1, 3, 5.
print any every x in samples if (invoke passes to (x, threshold)).
print all every x in samples if (invoke passes to (x, threshold)).
print threshold."#);
    assert_eq!(r.status,0,"{}",r.stderr);assert_eq!(r.stdout,"true\nfalse\n2\n");
}
#[test]
fn rejects_invalid_input_and_predicate_types() {
    for (source,code) in [
        ("print any every n in 2 if true.","S365"),
        ("print all every n in (a list of 1, 2) if (n plus 2).","S366")
    ] {
        let e=Compiler::new().hir(source).unwrap_err();
        assert_eq!(e.code,code,"{source}: {e}");
        assert!(e.span.is_some());
    }
}
#[test]
fn scopes_do_not_leak_and_compile_apis_see_boolean_reductions() {
    let err=Compiler::new().hir("let aresult be all every n in (a list of 1, 2) if true. print n.").unwrap_err();
    assert_eq!(err.code,"S312");
    let c=Compiler::new();
    let src="print any every n in (a list of 1, 2) if (n is equal to 1).";
    let hir=c.hir(src).unwrap().to_json().to_string();
    assert!(hir.contains("\"any\""),"{hir}");
    assert!(c.optimized_hir(src).unwrap().to_json().to_string().contains("list_transform"));
    assert_eq!(c.effect_analysis(src).unwrap().effects(),vec!["console_write".to_owned()]);
    assert_eq!(c.format_source("PRINT ANY EVERY n IN (a LIST OF 1, 2) IF (n IS EQUAL TO 1).").unwrap(),
        "print any every n in (a list of 1, 2) if (n is equal to 1).");
}
