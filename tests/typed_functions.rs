// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-functions-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn functions_are_typed_reusable_and_composable_with_map_filter() {
    let result = run(r#"define function square taking n as number to (n times n).
define function large taking n as number to (n is greater than 5).
let xs be range from 1 to 4.
let squares be map every x in xs to (invoke square to x).
let selected be filter every x in squares if (invoke large to x).
assert length of selected is equal to 2.
print squares.
print selected."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[1, 4, 9, 16]\n[9, 16]\n");
}

#[test]
fn supports_string_boolean_list_parameters_and_nested_calls() {
    let result = run(r#"define function shout taking s as string to (uppercase of s).
define function keep taking yes as boolean to (not yes).
define function size taking xs as list of number to (length of xs).
define function increment taking n as number to (n plus 1).
define function twice taking n as number to (invoke increment to (invoke increment to n)).
let data be a list of 3, 4.
assert invoke size to data is equal to 2.
assert invoke keep to false is equal to true.
print invoke shout to "hello".
print invoke twice to 5."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "HELLO\n7\n");
}

#[test]
fn parameter_shadows_outer_binding_without_leaking() {
    let result = run(r#"let x be 50.
let offset be 3.
define function add_offset taking x as number to (x plus offset).
print invoke add_offset to 2.
print x."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "5\n50\n");
}

#[test]
fn rejects_unknown_duplicate_and_wrong_argument_types() {
    for (src, code) in [
        ("print invoke missing to 2.", "S369"),
        ("define function f taking x as number to x. print invoke f to true.", "S370"),
        ("define function f taking x as number to x. define function f taking y as number to y. print 1.", "S371"),
        ("define function f taking x as number to (invoke f to x). print 1.", "S369"),
    ] {
        let err = Compiler::new().hir(src).unwrap_err();
        assert_eq!(err.code, code, "{src}: {err}");
        assert!(err.span.is_some());
    }
}

#[test]
fn refuses_effectful_function_bodies_and_nested_definitions() {
    for src in [
        "define function bad taking x as number to (print x). print 1.",
        "if true: define function bad taking x as number to x. end. print 1.",
    ] {
        let error = Compiler::new().hir(src).unwrap_err();
        assert!(matches!(error.code.as_str(), "S201" | "S217" | "S241" | "S372"), "{src}: {error}");
    }
}

#[test]
fn functions_remain_contextual_without_renaming_legacy_variables() {
    let result = run(r#"let define be 2.
let invoke be 3.
let function be 4.
print define plus invoke plus function.
define function double taking x as number to (x times 2).
print invoke double to 5."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "9\n10\n");
}

#[test]
fn formatter_ast_hir_optimizer_and_effect_analysis() {
    let compiler = Compiler::new();
    let source = "define function double taking n as number to (n times 2). print invoke double to 4.";
    assert!(compiler.parse(source).unwrap().to_json().to_string().contains("function_definition"));
    assert!(compiler.hir(source).unwrap().to_json().to_string().contains("function_call"));
    assert!(compiler.optimized_hir(source).unwrap().to_json().to_string().contains("function_call"));
    assert_eq!(compiler.effect_analysis(source).unwrap().effects(), vec!["console_write".to_owned()]);
    assert_eq!(
        compiler.format_source("PRINT INVOKE double TO 4.").unwrap(),
        "print invoke double to 4."
    );
}
