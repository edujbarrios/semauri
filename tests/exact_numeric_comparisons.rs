// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-exact-numbers-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn adjacent_large_integers_compare_exactly() {
    let src = r#"print 9007199254740993 is greater than 9007199254740992.
print 9007199254740993 is not equal to 9007199254740992.
print 9007199254740992 is less than 9007199254740993.
print 9223372036854775807 is greater than 9223372036854775806.
print 9223372036854775807 is greater than or equal to 9223372036854775807.
print 9223372036854775806 is less than or equal to 9223372036854775807.
print 9007199254740993 is equal to 9007199254740992.
print 9007199254740993 is less than 9007199254740992."#;
    let r = run(src);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "true\ntrue\ntrue\ntrue\ntrue\ntrue\nfalse\nfalse\n");
}

#[test]
fn number_equality_is_numerical_across_int_and_float() {
    let src = r#"print 1 is equal to 1.0.
print 1.0 is equal to 1.
print 1 is not equal to 1.0.
print 9007199254740993 is equal to 9007199254740992.0.
print 9007199254740993 is greater than 9007199254740992.0.
print 0 is equal to (minus 0.0).
print (minus 1) is less than (minus 0.5).
print 0 is greater than (minus 0.5).
print (minus 9223372036854775807) is greater than (minus 9223372036854775807) minus 1."#;
    let r = run(src);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "true\ntrue\nfalse\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\n");
}

#[test]
fn list_membership_and_distinct_use_numeric_value_equality() {
    let src = r#"let data be a list of 1, 1.0, 2, 2.0.
let deduped be distinct of data.
print length of deduped.
print 1.0 in (a list of 1, 2, 3).
print 9007199254740992.0 in (a list of 9007199254740993, 7).
print 9007199254740993 in (a list of 9007199254740992.0, 7)."#;
    let r=run(src);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "2\ntrue\nfalse\nfalse\n");
}

#[test]
fn compiler_optimizer_and_runtime_agree_without_new_effects() {
    let c=Compiler::new();
    let src="print 9007199254740993 is greater than 9007199254740992.";
    assert!(c.optimized_hir(src).unwrap().to_json().to_string().contains("true"));
    assert!(c.hir(src).is_ok());
    assert_eq!(c.effect_analysis(src).unwrap().effects(),
        vec!["console_write".to_string()]);
    for src in [r#"print 1 is equal to "1"."#, "print true is equal to 1."] {
        assert_eq!(c.hir(src).unwrap_err().code, "S315");
    }
}
