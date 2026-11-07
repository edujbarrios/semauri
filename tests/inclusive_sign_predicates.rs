// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-sign-predicates-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let output = run_cli(&["run".into(), "--allow".into(),
        "console_write".into(), path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    output
}
#[test]
fn positive_negative_and_zero_boundaries() {
    let output=run(r#"print 0 is nonzero.
print 0 is nonnegative.
print 0 is nonpositive.
print 0.0 is not nonzero.
print (minus 4) is nonzero.
print (minus 4) is nonnegative.
print (minus 4) is nonpositive.
print 4 is nonzero.
print 4 is nonnegative.
print 4 is nonpositive.
print 4 is not nonnegative.
print 4 is not nonpositive.
print (minus 4) is not nonnegative.
print (minus 4) is not nonpositive."#);
    assert_eq!(output.status,0,"{}",output.stderr);
    assert_eq!(output.stdout,
        "false\ntrue\ntrue\ntrue\ntrue\nfalse\ntrue\ntrue\ntrue\nfalse\nfalse\ntrue\ntrue\nfalse\n");
}
#[test]
fn exact_i64_boundaries_and_fractional_values() {
    let output=run(r#"print 9007199254740993 is nonzero.
print 9007199254740993 is nonnegative.
print (minus 9007199254740993) is nonpositive.
print 0.5 is not nonpositive.
print (minus 0.5) is not nonnegative."#);
    assert_eq!(output.status,0,"{}",output.stderr);
    assert_eq!(output.stdout,"true\n".repeat(5));
}
#[test]
fn type_errors_preserve_sign_diagnostics() {
    let compiler=Compiler::new();
    for expression in [r#"print "oops" is nonzero."#,
                       "print true is nonnegative.",
                       "print (a list of 1, 2) is not nonpositive."] {
        let err=compiler.hir(expression).unwrap_err();
        assert_eq!(err.code,"S417","{expression}: {err}");
        assert!(err.span.is_some());
    }
}
#[test]
fn canonical_format_identifiers_and_inspection() {
    let compiler=Compiler::new();
    for (source, expected) in [
        ("PRINT 0 IS NONZERO.","print 0 is nonzero."),
        ("PRINT 0 IS NONNEGATIVE.","print 0 is nonnegative."),
        ("PRINT 0 IS NOT NONPOSITIVE.","print 0 is not nonpositive."),
    ] { assert_eq!(compiler.format_source(source).unwrap(),expected); }
    compiler.compile(r#"let nonzero be 9.
let nonnegative be 6.
let nonpositive be 3.
assert nonzero is nonzero.
assert nonnegative is nonnegative.
assert nonpositive is not nonpositive."#,None).unwrap();
    for expr in ["print 2 is nonzero.","print 0 is nonnegative.",
                 "print 0 is not nonpositive."] {
        assert!(compiler.optimized_hir(expr).is_ok());
        assert_eq!(compiler.effect_analysis(expr).unwrap().effects(),
            vec!["console_write".to_string()]);
    }
    let ast=compiler.parse("print 0 is not nonzero.").unwrap().to_json().to_string();
    assert!(ast.contains("not_equal") || ast.contains("equal"),"{ast}");
}
#[test]
fn official_example_runs() {
    let output=run(include_str!("../examples/inclusive_sign_predicates.suri"));
    assert_eq!(output.status,0,"{}",output.stderr);
    assert_eq!(output.stdout,"Inclusive sign predicates OK\n");
}
#[test]
fn textmate_operator_matches_predicates() {
    let grammar:serde_json::Value=serde_json::from_str(include_str!(
        "../editors/vscode/syntaxes/semauri.tmLanguage.json")).unwrap();
    let operators=grammar["repository"]["operators"]["match"].as_str().unwrap();
    for word in ["nonzero","nonnegative","nonpositive"] {
        assert!(operators.contains(word),"missing {word}");
    }
}
