// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-bounds-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn boundary_comparisons_handle_strict_inclusive_and_negative_numbers() {
    let result = run(r#"print 3 is at least 3.
print 3 is at most 3.
print 3 is above 2.
print 3 is below 4.
print 3 is at least 4.
print 3 is at most 2.
print 3 is above 3.
print 3 is below 3.
print (minus 0.5) is below 0.
print 0.5 is above 0."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "true\ntrue\ntrue\ntrue\nfalse\nfalse\nfalse\nfalse\ntrue\ntrue\n");
}

#[test]
fn boundaries_preserve_number_types_and_original_comparators() {
    let compiler = Compiler::new();
    for source in [r#"print 1 is at least "1"."#, "print true is at most 2.",
                   r#"print "hello" is above "abc"."#, "print 2 is below false."] {
        let err = compiler.hir(source).unwrap_err();
        assert_eq!(err.code, "S314", "{source}: {err}");
    }
    let result = run(r#"let least be 2.
let above be 4.
assert least is at least 2.
assert above is above 3.
assert above is greater than least.
assert least is less than above.
print "checked"."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "checked\n");
}

#[test]
fn bounds_support_formatting_ast_optimizer_and_effects() {
    let compiler = Compiler::new();
    for (source, expected) in [
        ("PRINT 3 IS AT LEAST 2.", "print 3 is at least 2."),
        ("PRINT 3 IS AT MOST 4.", "print 3 is at most 4."),
        ("PRINT 3 IS ABOVE 2.", "print 3 is above 2."),
        ("PRINT 3 IS BELOW 4.", "print 3 is below 4."),
    ] { assert_eq!(compiler.format_source(source).unwrap(), expected); }
    let ast = compiler.parse("print 3 is at least 2.").unwrap().to_json().to_string();
    assert!(ast.contains("greater_than_or_equal"), "{ast}");
    assert!(compiler.optimized_hir("print 3 is below 4.").is_ok());
    assert_eq!(compiler.effect_analysis("print 4 is at most 5.").unwrap().effects(),
        vec!["console_write".to_string()]);
}

#[test]
fn natural_bound_example_executes() {
    let result = run(include_str!("../examples/natural_bounds.suri"));
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Natural bounds OK\n");
}
