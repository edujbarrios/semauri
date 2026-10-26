// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-extrema-{}-{}.suri",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&[
        "run".into(),
        "--allow".into(),
        "console_write".into(),
        path.to_string_lossy().into_owned(),
    ]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn extrema_are_pure_typed_and_composable() {
    let result = run(r#"let scores be a list of 3, minus 4, 7, 1.
let smallest be minimum of scores.
let largest be maximum of scores.
assert smallest is equal to minus 4.
assert largest is equal to 7.
assert minimum of (reverse of scores) is equal to minus 4.
assert (scores at 0) is equal to 3.
print smallest.
print largest."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "-4\n7\n");
}

#[test]
fn integer_comparisons_above_f64_precision_are_exact() {
    let result = run("print minimum of a list of 9007199254740993, 9007199254740992. print maximum of a list of 9007199254740993, 9007199254740992.");
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "9007199254740992\n9007199254740993\n");
}

#[test]
fn decimal_extrema_and_duplicate_values() {
    let result = run("print minimum of a list of 3.5, minus 0.5, 3.5. print maximum of a list of minus 2.5, 1.25, 1.25.");
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "-0.5\n1.25\n");
}

#[test]
fn rejects_incompatible_operands_with_source_spans() {
    let compiler = Compiler::new();
    for source in [
        "print minimum of 10.",
        "print maximum of true.",
        "print minimum of a list of \"a\", \"b\".",
        "print maximum of a list of true, false.",
    ] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S353", "{source}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn formatter_hir_and_effect_analysis_support_extrema() {
    let compiler = Compiler::new();
    assert_eq!(
        compiler.format_source("PRINT MINIMUM OF a LIST OF 1, 2.").unwrap(),
        "print minimum of a list of 1, 2."
    );
    assert!(compiler.hir("let low be maximum of a list of 3, 8.").unwrap()
        .to_json().to_string().contains("maximum"));
    assert_eq!(
        compiler.effect_analysis("print minimum of a list of 1, 2.")
            .unwrap().effects(),
        vec!["console_write".to_string()],
    );
}
