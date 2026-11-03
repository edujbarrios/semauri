// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-numeric-predicates-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn numeric_predicates_work_for_ints_decimals_and_negations() {
    let result = run(r#"print 0 is zero.
print 0.0 is zero.
print 1 is not zero.
print (minus 1) is negative.
print (minus 0.5) is negative.
print 0.0 is not positive.
print 0 is not negative.
print 1.5 is positive.
print 2 is even.
print 3 is odd.
print 3 is not even.
print 4 is not odd."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "true\n".repeat(12));
}

#[test]
fn numeric_predicates_preserve_typed_diagnostics() {
    let compiler = Compiler::new();
    for src in [r#"print "text" is zero."#, "print true is positive.",
                "print (a list of 1, 2) is not negative."] {
        let err = compiler.hir(src).unwrap_err();
        assert_eq!(err.code, "S417", "{src}: {err}");
    }
    for src in ["print true is even.", r#"print "a" is not odd."#] {
        let err = compiler.hir(src).unwrap_err();
        assert_eq!(err.code, "S425", "{src}: {err}");
    }
    for src in ["print 2.5 is even.", "print 1.5 is not odd."] {
        let err = compiler.compile(src, None).unwrap_err();
        assert_eq!(err.code, "S426", "{src}: {err}");
    }
}

#[test]
fn contextual_names_and_existing_comparisons_remain_compatible() {
    let compiler = Compiler::new();
    compiler.compile(r#"let zero be 4.
let positive be 6.
let even be 8.
assert zero is equal to 4.
assert positive is greater than 2.
assert even is even.
assert zero is not zero."#, None).unwrap();
    for (raw, formatted) in [
        ("PRINT 0 IS ZERO.", "print 0 is zero."),
        ("PRINT 2 IS EVEN.", "print 2 is even."),
        ("PRINT 2 IS NOT ODD.", "print 2 is not odd."),
        ("PRINT (MINUS 1) IS NEGATIVE.", "print (minus 1) is negative."),
    ] {
        assert_eq!(compiler.format_source(raw).unwrap(), formatted);
    }
    let ast = compiler.parse("print 3 is not even.").unwrap().to_json().to_string();
    assert!(ast.contains("not") && ast.contains("even"), "{ast}");
    assert!(compiler.optimized_hir("print 0.0 is zero.").is_ok());
    assert_eq!(compiler.effect_analysis("print 2 is even.").unwrap().effects(),
        vec!["console_write".to_owned()]);
}

#[test]
fn numeric_predicates_example_compiles_and_runs() {
    let result = run(include_str!("../examples/natural_numeric_predicates.suri"));
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Natural numeric predicates OK\n");
}
