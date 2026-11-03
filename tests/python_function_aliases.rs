// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-python-aliases-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn aliases_execute_with_typed_lists_unicode_and_text() {
    let source = r#"let xs be a list of 2, 4, 6.
print len of "ñ🌍".
print abs of (minus 3).
print mean of xs.
print min of xs.
print max of xs.
print strip of "  hi  ".
print upper of "hello".
print lower of "WORLD".
let normalized be map every x in xs to (abs of x).
assert len of normalized is equal to 3."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "2\n3\n4\n2\n6\nhi\nHELLO\nworld\n");
}

#[test]
fn alias_errors_match_original_operator_type_checks() {
    let compiler = Compiler::new();
    for (alias, canonical, operand) in [
        ("len", "length", "123"),
        ("abs", "absolute", "true"),
        ("mean", "average", r#""hello""#),
        ("min", "minimum", "true"),
        ("max", "maximum", "3"),
        ("strip", "trim", "9"),
        ("upper", "uppercase", "true"),
        ("lower", "lowercase", "false"),
    ] {
        let old = compiler.hir(&format!("print {canonical} of {operand}.")).unwrap_err();
        let new = compiler.hir(&format!("print {alias} of {operand}.")).unwrap_err();
        assert_eq!(new.code, old.code, "{alias} should share typing with {canonical}");
        assert!(new.span.is_some());
    }
}

#[test]
fn aliases_preserve_contextual_identifiers_effects_and_formatting() {
    let compiler = Compiler::new();
    compiler.compile(r#"let len be 7. let abs be 9. let mean be 4.
assert len is equal to 7. assert abs is equal to 9. assert mean is equal to 4."#, None).unwrap();
    for name in ["len", "abs", "mean", "min", "max", "strip", "upper", "lower"] {
        let upper = format!("PRINT {} OF (a list of 2, 4).", name.to_uppercase());
        let lower = format!("print {name} of (a list of 2, 4).");
        assert_eq!(compiler.format_source(&upper).unwrap(), lower);
    }
    let source = "print abs of (minus 5).";
    assert!(compiler.optimized_hir(source).unwrap().to_json().to_string().contains("5"));
    assert_eq!(compiler.effect_analysis(source).unwrap().effects(),
        vec!["console_write".to_string()]);
}

#[test]
fn example_is_runnable_without_new_capabilities() {
    let result = run(include_str!("../examples/python_function_aliases.suri"));
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Python-style functions OK\n");
}
