// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static CASE: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-contains-{}-{}.suri",
        std::process::id(),
        CASE.fetch_add(1, Ordering::Relaxed),
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&[
        "run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().to_string(),
    ]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn contains_checks_unicode_substrings_and_empty_needle() {
    let result = run(
        "Print \"semauri\" contains \"mau\". Print \"ño🌍\" contains \"🌍\". Print \"hi\" contains \"\"."
    );
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "true\ntrue\ntrue\n");
}

#[test]
fn contains_checks_typed_list_membership() {
    let source = r#"Let values be a list of 10, 20, 30.
Print values contains 20.
Print values contains 21.
Let labels be a list of "red", "green".
Print labels contains "green"."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "true\nfalse\ntrue\n");
}

#[test]
fn contains_can_drive_conditionals_and_assertions() {
    let source = r#"Let values be a list of red, blue.
If values contains blue:
  Print "yes".
Otherwise:
  Print "no".
End.
Assert "Semauri" contains "mau"."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "yes\n");
}

#[test]
fn contains_rejects_mismatched_element_types() {
    for source in [
        "Print 123 contains 2.",
        "Print \"hi\" contains 2.",
        "Let values be a list of 1, 2. Print values contains \"1\".",
    ] {
        let err = Compiler::new().hir(source).unwrap_err();
        assert_eq!(err.code, "S344", "{source}");
    }
}

#[test]
fn contains_is_an_explicit_ast_operator_and_safe_to_fold() {
    let compiler = Compiler::new();
    let ast = compiler.parse("Print \"hello\" contains \"ell\".").unwrap().to_json();
    assert!(ast.to_string().contains("\"contains\""));
    let optimized = compiler
        .optimized_hir("Print \"hello\" contains \"ell\".")
        .unwrap()
        .to_json();
    assert!(optimized.to_string().contains("true"));
    assert_eq!(
        compiler.effect_analysis("Print \"hello\" contains \"ell\".")
            .unwrap().effects(),
        vec!["console_write".to_string()]
    );
}

#[test]
fn natural_membership_supports_lists_strings_and_negative_forms() {
    let source = r#"let nums be a list of 2, 3, 5.
print 3 in nums.
print 4 not in nums.
print 2 is in nums.
print 6 is not in nums.
print "aur" in "Semauri".
print "xyz" not in "Semauri".
when 5 in nums and "abc" not in "hello":
    print "matched".
end."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "true\ntrue\ntrue\ntrue\ntrue\ntrue\nmatched\n");
}

#[test]
fn natural_membership_preserves_type_errors_and_compiler_passes() {
    let compiler = Compiler::new();
    for source in [r#"print "3" in (a list of 3, 4)."#, "print 3 in 42.",
                   "print 3 not in true.", r#"print 1 in "123"."#] {
        let err = compiler.hir(source).unwrap_err();
        assert_eq!(err.code, "S344", "{source}: {err}");
    }
    let ast = compiler.parse("print 2 not in (a list of 1, 2).").unwrap().to_json().to_string();
    assert!(ast.contains("contains") && ast.contains("not"), "{ast}");
    let optimized = compiler.optimized_hir(r#"print "a" in "cat"."#).unwrap().to_json().to_string();
    assert!(optimized.contains("true"), "{optimized}");
    assert_eq!(compiler.effect_analysis("print 3 in (a list of 3, 4).").unwrap().effects(),
        vec!["console_write".to_string()]);
    assert_eq!(compiler.format_source("PRINT 3 NOT IN (a list of 3, 4).").unwrap(),
        "print 3 not in (a list of 3, 4).");
}
