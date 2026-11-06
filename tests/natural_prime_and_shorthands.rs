// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-prime-shorthands-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn natural_prime_and_not_prime_reuse_checked_primality() {
    let result = run(r#"print 2 is prime.
print 13 is prime.
print 21 is not prime.
print 1 is not prime.
print (minus 7) is not prime.
print 8 is prime.
print 13 is not prime."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "true\ntrue\ntrue\ntrue\ntrue\nfalse\nfalse\n");
}

#[test]
fn concise_function_aliases_work_with_typed_lists_and_stats() {
    let result = run(r#"let nums be a list of 3, 5, 5, 7.
print size of nums.
print total of nums.
print size of (unique of nums).
print head of nums.
print final of nums.
print (avg of nums) is greater than 4.
print (var of nums) is greater than 0.
print (stddev of nums) is greater than 0.
let names be a list of "Ada", "Grace".
print size of names."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "4\n20\n3\n3\n7\ntrue\ntrue\ntrue\n2\n");
}

#[test]
fn prime_and_alias_type_errors_match_existing_diagnostics() {
    let c=Compiler::new();
    for src in [r#"print "13" is prime."#, "print true is not prime.",
                "print (a list of 2, 3) is prime."] {
        let e=c.hir(src).unwrap_err();
        assert_eq!(e.code, "S435", "{src}: {e}");
    }
    for src in ["print 1.5 is prime.", "print 2.5 is not prime."] {
        let r=run(src);
        assert_eq!(r.status, 65, "{src}: {}", r.stderr);
        assert!(r.stderr.contains("S436"), "{src}: {}", r.stderr);
    }
    for (alias, original, input) in [
        ("avg", "average", "42"), ("var", "variance", "true"),
        ("stddev", "stdev", r#""hi""#), ("total", "sum", "true"),
        ("size", "length", "42"), ("unique", "distinct", "42"),
        ("head", "first", "true"), ("final", "last", "true"),
    ] {
        let a=c.hir(&format!("print {alias} of {input}.")).unwrap_err();
        let b=c.hir(&format!("print {original} of {input}.")).unwrap_err();
        assert_eq!(a.code, b.code, "{alias} should share checks with {original}");
    }
}

#[test]
fn source_formatting_contextual_names_and_inspection() {
    let c=Compiler::new();
    for name in ["avg", "var", "stddev", "total", "size", "unique", "head", "final"] {
        let src=format!("PRINT {} OF (a list of 2, 4).", name.to_uppercase());
        let expected=format!("print {name} of (a list of 2, 4).");
        assert_eq!(c.format_source(&src).unwrap(), expected);
    }
    for (src, expected) in [
        ("PRINT 13 IS PRIME.", "print 13 is prime."),
        ("PRINT 21 IS NOT PRIME.", "print 21 is not prime."),
    ] { assert_eq!(c.format_source(src).unwrap(), expected); }
    c.compile(r#"let prime be 7.
let avg be 5.
let unique be 3.
assert prime is prime.
assert avg is equal to 5.
assert unique is equal to 3."#, None).unwrap();
    let ast=c.parse("print 17 is not prime.").unwrap().to_json().to_string();
    assert!(ast.contains("prime") && ast.contains("not"), "{ast}");
    assert!(c.optimized_hir("print 13 is prime.").is_ok());
    assert_eq!(c.effect_analysis("print 13 is prime.").unwrap().effects(),
        vec!["console_write".to_string()]);
}

#[test]
fn official_example_executes() {
    let r=run(include_str!("../examples/natural_prime_and_shortcuts.suri"));
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "Prime and shorthand functions OK\n");
}
