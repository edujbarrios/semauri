// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-sign-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&[
        "run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned(),
    ]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn computes_sign_for_integers_decimals_zero_and_extreme_i64() {
    let result = run(r#"print sign of (minus 8).
print sign of 0.
print sign of 7.
print sign of (minus 0.25).
print sign of 0.0.
print sign of 8.5.
let lowest be (minus 9223372036854775807) minus 1.
print sign of lowest."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "-1\n0\n1\n-1\n0\n1\n-1\n");
}

#[test]
fn sign_composes_with_functions_and_list_transforms() {
    let result = run(r#"define function polarity taking n as number to (sign of n).
let numbers be a list of minus 4, 0, 9.
print map every n in numbers to (invoke polarity to n)."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[-1, 0, 1]\n");
}

#[test]
fn reports_located_type_errors() {
    let compiler = Compiler::new();
    for source in [
        "print sign of true.",
        "print sign of \"test\".",
        "print sign of a list of 1, 2.",
    ] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S417", "{source}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn sign_is_contextual_and_the_formatter_handles_case() {
    let compiler = Compiler::new();
    let result = run("let sign be 3. print sign. print sign of (minus 10).");
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "3\n-1\n");
    assert_eq!(
        compiler.format_source("PRINT SIGN OF (MINUS 2).").unwrap(),
        "print sign of (minus 2)."
    );
    assert_eq!(
        compiler.format_source("LET Sign BE 4. PRINT Sign.").unwrap(),
        "let Sign be 4. print Sign."
    );
}

#[test]
fn optimizer_and_effect_analysis_understand_sign() {
    let compiler = Compiler::new();
    let src = "print sign of (minus 9).";
    assert!(compiler.hir(src).unwrap().to_json().to_string().contains("sign"));
    assert!(compiler.optimized_hir(src).is_ok());
    assert_eq!(
        compiler.effect_analysis(src).unwrap().effects(),
        vec!["console_write".to_string()]
    );
}
