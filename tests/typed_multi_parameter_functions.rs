// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-multiparam-{}-{}.sema",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn functions_accept_two_and_three_typed_arguments() {
    let r = run(r#"define function sum_two taking left as number, right as number to (left plus right).
define function weighted taking x as number, weight as number, bias as number to ((x times weight) plus bias).
print invoke sum_two to (9, 4).
print invoke weighted to (3, 5, 2)."#);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "13\n17\n");
}

#[test]
fn functions_compose_with_existing_single_parameter_calls_and_lists() {
    let r = run(r#"define function square taking n as number to (n times n).
define function shift taking value as number, bias as number to (value plus bias).
let numbers be range from 1 to 4.
let values be map every n in numbers to (invoke shift to (invoke square to n, 10)).
print values.
print invoke square to 5."#);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "[11, 14, 19, 26]\n25\n");
}

#[test]
fn nested_list_and_heterogeneous_parameter_types_work() {
    let bad = Compiler::new().hir(r#"define function pick taking xs as list of number, enabled as boolean to (first of xs).
let xs be a list of 7, true.
print invoke pick to (xs, true)."#).unwrap_err();
    assert_eq!(bad.code, "S321");
    let ok = run(r#"define function pick taking xs as list of number, enabled as boolean to (first of xs).
define function concatenate taking left as string, right as string to (left plus right).
let xs be a list of 7, 8.
print invoke pick to (xs, true).
print invoke concatenate to ("semi", "uri")."#);
    assert_eq!(ok.status, 0, "{}", ok.stderr);
    assert_eq!(ok.stdout, "7\nsemiuri\n");
}

#[test]
fn variable_capture_and_parameter_shadowing_are_lexically_safe() {
    let r = run(r#"let bias be 10.
let value be 100.
define function add_bias taking value as number, offset as number to ((value plus offset) plus bias).
print invoke add_bias to (1, 2).
print value."#);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "13\n100\n");
}

#[test]
fn detects_arity_errors_and_argument_types_at_compile_time() {
    for (src, code) in [
        ("define function sum_two taking x as number, y as number to (x plus y). print invoke sum_two to 1.", "S374"),
        ("define function sum_two taking x as number, y as number to (x plus y). print invoke sum_two to (1, 2, 3).", "S374"),
        ("define function sum_two taking x as number, y as number to (x plus y). print invoke sum_two to ().", "S374"),
        ("define function sum_two taking x as number, y as number to (x plus y). print invoke sum_two to (true, 2).", "S370"),
        ("define function sum_two taking x as number, y as number to (x plus y). print invoke sum_two to (1, true).", "S370"),
        ("define function sum_two taking x as number, y as number to (x plus y). print invoke sum_two to (1, 2", "S247"),
    ] {
        let e = Compiler::new().hir(src).unwrap_err();
        assert_eq!(e.code, code, "{src}: {e}");
        assert!(e.span.is_some());
    }
}

#[test]
fn duplicate_parameter_names_rejected_and_scope_does_not_leak() {
    let err = Compiler::new().hir(
        "define function broken taking x as number, x as number to x. print 1."
    ).unwrap_err();
    assert_eq!(err.code, "S311");
    let err = Compiler::new().hir(
        "define function combine taking x as number, y as number to (x plus y). print y."
    ).unwrap_err();
    assert_eq!(err.code, "S312");
}

#[test]
fn supports_parenthesized_single_arguments_and_nested_calls() {
    let r = run(r#"define function double taking x as number to (x times 2).
define function combine taking x as number, y as number to (x plus y).
print invoke double to (3 plus 4).
print invoke combine to (invoke double to 2, invoke double to 3)."#);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "14\n10\n");
}

#[test]
fn compiler_apis_preserve_multi_argument_structure() {
    let c=Compiler::new();
    let src="define function combine taking x as number, y as number to (x plus y). print invoke combine to (2, 3).";
    let ast=c.parse(src).unwrap().to_json().to_string();
    assert!(ast.contains("parameters"), "{ast}");
    let hir=c.hir(src).unwrap().to_json().to_string();
    assert!(hir.contains("arguments"), "{hir}");
    let optimized=c.optimized_hir(src).unwrap().to_json().to_string();
    assert!(optimized.contains("function_call"), "{optimized}");
    assert_eq!(c.effect_analysis(src).unwrap().effects(), vec!["console_write".to_owned()]);
    assert_eq!(c.format_source("PRINT INVOKE combine TO (2, 3).").unwrap(), "print invoke combine to (2, 3).");
}
