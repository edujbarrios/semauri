// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-boolean-membership-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn boolean_tests_use_strict_equality_and_support_negation() {
    let result = run(r#"print true is true.
print true is not false.
print false is false.
print false is not true.
print true is false.
print false is not false."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "true\ntrue\ntrue\ntrue\nfalse\nfalse\n");
    let c = Compiler::new();
    for s in ["print 1 is true.", r#"print "yes" is not false."#] {
        let e = c.hir(s).unwrap_err();
        assert_eq!(e.code, "S315", "{s}: {e}");
    }
}

#[test]
fn natural_membership_aliases_preserve_typed_contains_and_negation() {
    let result = run(r#"let xs be a list of 2, 4, 6.
print 4 is one of xs.
print 3 is not one of xs.
print "aur" is one of "Semauri".
print "x" is not one of "Semauri"."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "true\ntrue\ntrue\ntrue\n");
    let c = Compiler::new();
    for s in [r#"print "2" is one of (a list of 2, 4)."#,
              "print 2 is one of false.", "print 1 is not one of 42."] {
        let e = c.hir(s).unwrap_err();
        assert_eq!(e.code, "S344", "{s}: {e}");
    }
}

#[test]
fn both_predicates_support_compiler_passes_and_contextual_names() {
    let c = Compiler::new();
    c.compile(r#"let one be "one".
let names be a list of "Ada", "Grace".
assert "Ada" is one of names.
assert one is equal to "one"."#, None).unwrap();
    for (source, expected) in [
        ("PRINT TRUE IS NOT FALSE.", "print true is not false."),
        ("PRINT 2 IS ONE OF (a list of 2, 4).", "print 2 is one of (a list of 2, 4)."),
        ("PRINT 2 IS NOT ONE OF (a list of 1, 3).", "print 2 is not one of (a list of 1, 3)."),
    ] { assert_eq!(c.format_source(source).unwrap(), expected); }
    let ast = c.parse("print 2 is not one of (a list of 1, 3).").unwrap().to_json().to_string();
    assert!(ast.contains("contains") && ast.contains("not"), "{ast}");
    assert!(c.optimized_hir("print true is false.").is_ok());
    assert_eq!(c.effect_analysis("print true is true.").unwrap().effects(),
        vec!["console_write".to_string()]);
}

#[test]
fn natural_boolean_example_executes() {
    let result = run(include_str!("../examples/boolean_membership.suri"));
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Boolean membership OK\n");
}
