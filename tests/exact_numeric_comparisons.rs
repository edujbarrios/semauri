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
fn ordering_handles_mixed_integer_and_decimal_without_rounding() {
    let src = r#"print 9007199254740993 is greater than 9007199254740992.0.
print 9007199254740993 is at least 9007199254740992.0.
print 9223372036854775807 is below 9223372036854775807.0.
print 0 is greater than (minus 0.5).
print 1 is less than 1.5.
print (minus 1) is less than (minus 0.5).
print (minus 9223372036854775807) is greater than (minus 9223372036854775807) minus 1.
print 9007199254740993 is less than 9007199254740992.0."#;
    let r=run(src);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "true\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\nfalse\n");
}

#[test]
fn existing_storage_sensitive_equality_and_distinct_stay_compatible() {
    let src = r#"print 1 is equal to 1.0.
print 1 is not equal to 1.0.
print 9007199254740993 is not equal to 9007199254740992.
let data be a list of 1, 1.0, 2, 2.0.
print length of (distinct of data).
print 1.0 in (a list of 1, 2, 3).
print 9007199254740993 in (a list of 9007199254740992, 9007199254740993)."#;
    let r=run(src);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "false\ntrue\ntrue\n4\nfalse\ntrue\n");
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
