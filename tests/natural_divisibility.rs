// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-divisibility-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn divisible_by_and_multiple_of_execute_and_negate() {
    let source = r#"print 12 is divisible by 3.
print 12 is not divisible by 5.
print 12 is a multiple of 4.
print 12 is not a multiple of 5.
print (minus 12) is divisible by 3.
print 0 is a multiple of 9.
print 13 is divisible by 2.
print 13 is a multiple of 2."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "true\ntrue\ntrue\ntrue\ntrue\ntrue\nfalse\nfalse\n");
}

#[test]
fn modulo_of_large_integers_is_exact_in_optimizer_and_runtime() {
    let source = r#"print 9007199254740993 modulo 2.
print 9007199254740993 is not divisible by 2.
print 9007199254740993 is a multiple of 3.
print 9223372036854775807 modulo 2.
let lowest be (minus 9223372036854775807) minus 1.
print lowest modulo (minus 1).
print lowest is divisible by (minus 1)."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "1\ntrue\ntrue\n1\n0\ntrue\n");
    let c = Compiler::new();
    assert!(c.optimized_hir("print 9007199254740993 modulo 2.").unwrap().to_json().to_string().contains("1"));
}

#[test]
fn type_checking_and_zero_divisors_keep_existing_errors() {
    let c = Compiler::new();
    for src in [r#"print "12" is divisible by 3."#,
                "print true is divisible by 2.",
                "print 3 is a multiple of false."] {
        let e = c.hir(src).unwrap_err();
        assert_eq!(e.code, "S314", "{src}: {e}");
    }
    for src in ["print 12 is divisible by 0.",
                "print 12 is not a multiple of 0.",
                "print 12 modulo 0."] {
        let r = run(src);
        assert_ne!(r.status, 0, "{src}");
        assert!(r.stderr.contains("S317"), "{src}: {}", r.stderr);
    }
}

#[test]
fn compiler_passes_formatter_and_contextual_identifiers_work() {
    let c = Compiler::new();
    c.compile(r#"let divisible be 3.
let multiple be 6.
assert multiple is divisible by divisible.
assert multiple is a multiple of divisible.
assert divisible is equal to 3."#, None).unwrap();
    for (source, expect) in [
        ("PRINT 8 IS DIVISIBLE BY 4.", "print 8 is divisible by 4."),
        ("PRINT 8 IS NOT DIVISIBLE BY 3.", "print 8 is not divisible by 3."),
        ("PRINT 8 IS A MULTIPLE OF 4.", "print 8 is a multiple of 4."),
        ("PRINT 8 IS NOT A MULTIPLE OF 3.", "print 8 is not a multiple of 3."),
    ] { assert_eq!(c.format_source(source).unwrap(), expect); }
    let ast = c.parse("print 8 is divisible by 4.").unwrap().to_json().to_string();
    assert!(ast.contains("modulo") && ast.contains("equal"), "{ast}");
    assert_eq!(c.effect_analysis("print 8 is a multiple of 4.").unwrap().effects(),
        vec!["console_write".to_string()]);
}

#[test]
fn example_runs_without_new_capabilities() {
    let r = run(include_str!("../examples/natural_divisibility.suri"));
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "Natural divisibility OK\n");
}
