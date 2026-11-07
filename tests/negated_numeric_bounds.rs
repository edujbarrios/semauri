// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-negative-bounds-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&[
        "run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()
    ]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn negated_bounds_preserve_inclusive_and_strict_edges() {
    let output = run(r#"print 3 is not above 3.
print 3 is not below 3.
print 3 is not at least 3.
print 3 is not at most 3.
print 3 is not above 2.
print 3 is not below 4.
print 3 is not at least 4.
print 3 is not at most 2.
print (minus 0.5) is not below 0.
print 0.5 is not above 0."#);
    assert_eq!(output.status, 0, "{}", output.stderr);
    assert_eq!(output.stdout, "true\ntrue\nfalse\nfalse\nfalse\nfalse\ntrue\ntrue\nfalse\nfalse\n");
}

#[test]
fn large_integers_keep_exact_ordering() {
    let output = run(r#"print 9007199254740993 is not above 9007199254740992.
print 9007199254740993 is not below 9007199254740994.
print 9007199254740993 is not at least 9007199254740994.
print 9007199254740993 is not at most 9007199254740992."#);
    assert_eq!(output.status, 0, "{}", output.stderr);
    assert_eq!(output.stdout, "false\nfalse\ntrue\ntrue\n");
}

#[test]
fn type_diagnostics_and_optimized_hir_are_preserved() {
    let compiler = Compiler::new();
    for (expression, original) in [
        (r#"print 1 is not above "1"."#, r#"print 1 is at most "1"."#),
        ("print true is not below 2.", "print true is at least 2."),
        (r#"print "x" is not at least 2."#, r#"print "x" is below 2."#),
        ("print 2 is not at most false.", "print 2 is above false."),
    ] {
        let actual = compiler.hir(expression).unwrap_err();
        let expected = compiler.hir(original).unwrap_err();
        assert_eq!(actual.code, "S314", "{expression}: {actual}");
        assert_eq!(actual.code, expected.code);
        assert!(actual.span.is_some());
    }
    for expression in [
        "print 3 is not above 4.",
        "print 3 is not below 2.",
        "print 3 is not at least 4.",
        "print 3 is not at most 2.",
    ] {
        assert!(compiler.optimized_hir(expression).is_ok(), "{expression}");
        assert_eq!(compiler.effect_analysis(expression).unwrap().effects(),
            vec!["console_write".to_string()]);
    }
}

#[test]
fn formatter_and_contextual_identifiers_are_preserved() {
    let compiler = Compiler::new();
    for (source, expected) in [
        ("PRINT 3 IS NOT ABOVE 4.", "print 3 is not above 4."),
        ("PRINT 3 IS NOT BELOW 2.", "print 3 is not below 2."),
        ("PRINT 3 IS NOT AT LEAST 4.", "print 3 is not at least 4."),
        ("PRINT 3 IS NOT AT MOST 2.", "print 3 is not at most 2."),
    ] {
        assert_eq!(compiler.format_source(source).unwrap(), expected);
    }
    compiler.compile(r#"let least be 2.
let most be 4.
let above be 5.
let below be 1.
assert least is not above most.
assert above is not below most.
assert below is not at least least.
assert most is not at most least."#, None).unwrap();
}

#[test]
fn ast_uses_existing_checked_comparison_operators() {
    let compiler = Compiler::new();
    for (expression, operator) in [
        ("print 3 is not above 4.", "less_than_or_equal"),
        ("print 3 is not below 2.", "greater_than_or_equal"),
        ("print 3 is not at least 4.", "less_than"),
        ("print 3 is not at most 2.", "greater_than"),
    ] {
        let ast = compiler.parse(expression).unwrap().to_json().to_string();
        assert!(ast.contains(operator), "{expression}: {ast}");
    }
}

#[test]
fn executable_example() {
    let output = run(include_str!("../examples/negated_numeric_bounds.suri"));
    assert_eq!(output.status, 0, "{}", output.stderr);
    assert_eq!(output.stdout, "Negated numeric bounds OK\n");
}
