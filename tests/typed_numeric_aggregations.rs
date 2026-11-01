// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-aggregate-{}-{}.suri", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&[
        "run".into(), "--allow".into(), "console_write".into(), path.to_string_lossy().into_owned(),
    ]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn sums_and_averages_numeric_lists_with_types_preserved() {
    let result = run(r#"let values be a list of 2, 4, 6.
let total be sum of values.
let mean be average of values.
assert total is equal to 12.
assert mean is equal to 4.
assert sum of (reverse of values) is equal to 12.
print total.
print mean."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "12\n4\n");
}

#[test]
fn aggregates_decimal_lists_and_fractional_averages() {
    let result = run("print sum of a list of 1.5, 2.5. print average of a list of 1, 2.");
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "4\n1.5\n");
}

#[test]
fn integer_sums_do_not_round_through_f64() {
    let result = run("print sum of a list of 9007199254740993, 1.");
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "9007199254740994\n");
}

#[test]
fn checked_integer_overflow_returns_located_diagnostic() {
    let source = "print sum of a list of 9223372036854775807, 1.";
    let result = run(source);
    assert_eq!(result.status, 65, "{}", result.stderr);
    assert!(result.stderr.contains("S354"), "{}", result.stderr);
}

#[test]
fn rejects_non_numeric_lists_at_compile_time() {
    let compiler = Compiler::new();
    for source in [
        "print sum of 42.",
        "print average of true.",
        "print sum of a list of \"a\", \"b\".",
        "print average of a list of false, true.",
    ] {
        let err = compiler.hir(source).unwrap_err();
        assert_eq!(err.code, "S353", "{source}: {err}");
        assert!(err.span.is_some());
    }
}

#[test]
fn aggregates_can_be_formatted_and_require_no_extra_effects() {
    let compiler = Compiler::new();
    assert_eq!(
        compiler.format_source("PRINT SUM OF a LIST OF 1, 2.").unwrap(),
        "print sum of a list of 1, 2."
    );
    assert_eq!(
        compiler.effect_analysis("print average of a list of 1, 3.")
            .unwrap().effects(),
        vec!["console_write".to_string()],
    );
    let hir = compiler.hir("let total be sum of a list of 1, 2.").unwrap();
    assert!(hir.to_json().to_string().contains("sum"));
}
