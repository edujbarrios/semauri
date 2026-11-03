// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-predicates-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn empty_and_blank_handle_typed_lists_unicode_and_whitespace() {
    let result = run(r#"let items be a list of 1, 2, 3.
let none be take 0 from items.
print none is empty.
print items is not empty.
print "" is empty.
print "ñ🌍" is not empty.
print "   " is blank.
print "  text  " is not blank.
print "" is blank.
print "x" is not blank."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "true\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\n");
}

#[test]
fn natural_predicates_retain_exact_type_errors() {
    let compiler = Compiler::new();
    for source in ["print 7 is empty.", "print true is not empty."] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S343", "{source}: {error}");
        assert!(error.span.is_some());
    }
    for source in ["print 4 is blank.", "print false is not blank.",
                   "print (a list of 1, 2) is blank."] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S351", "{source}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn predicates_work_in_conditionals_and_compiler_passes() {
    let compiler = Compiler::new();
    let src = r#"let xs be a list of 2, 4.
when xs is not empty and " \t " is blank then:
    print "match".
end."#;
    let result = run(src);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "match\n");
    let ast = compiler.parse(r#"print "   " is blank."#).unwrap().to_json().to_string();
    assert!(ast.contains("trim") && ast.contains("length") && ast.contains("equal"), "{ast}");
    assert!(compiler.optimized_hir(r#"print "" is empty."#).is_ok());
    assert_eq!(compiler.effect_analysis(src).unwrap().effects(),
        vec!["console_write".to_owned()]);
    assert_eq!(compiler.format_source(r#"PRINT "  " IS NOT BLANK."#).unwrap(),
        r#"print "  " is not blank."#);
}

#[test]
fn contextual_identifiers_and_existing_equality_stay_valid() {
    let source = r#"let empty be "empty".
let blank be "blank".
assert empty is equal to "empty".
assert blank is equal to "blank".
assert empty is not equal to blank."#;
    Compiler::new().compile(source, None).unwrap();
}

#[test]
fn example_runs_with_console_write_only() {
    let result = run(include_str!("../examples/natural_predicates.suri"));
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Natural predicates OK\n");
}
