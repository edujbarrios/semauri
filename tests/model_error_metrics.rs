// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-model-errors-{}-{}.suri",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
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
fn runnable_model_example_passes_check_and_prints_expected_output() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/model_error_metrics.suri");
    let checked = run_cli(&["check".into(), path.into()]);
    assert_eq!(checked.status, 0, "{}", checked.stderr);
    let executed = run(include_str!("../examples/model_error_metrics.suri"));
    assert_eq!(executed.status, 0, "{}", executed.stderr);
    assert_eq!(executed.stdout, "Model error metrics OK\n");
}

#[test]
fn aliases_are_numerically_equivalent_and_bias_keeps_sign() {
    let source = r#"let residuals be a list of 2, (minus 3), 1.
assert (mse of residuals) is equal to (mean square of residuals).
assert (mean squared error of residuals) is equal to (14 divided by 3).
assert (rmse of residuals) is equal to (rms of residuals).
assert (root mean squared error of residuals) is equal to (sqrt of (14 divided by 3)).
assert (mbe of residuals) is equal to (average of residuals).
assert (mean bias error of residuals) is equal to 0.
assert (mean error of residuals) is equal to 0.
let negative be a list of (minus 3), (minus 1).
assert (mbe of negative) is equal to (minus 2).
assert (mean bias error of negative) is equal to (minus 2).
let fractional be a list of 1.5, (minus 0.5).
assert (mse of fractional) is equal to 1.25.
assert (rmse of fractional) is equal to (sqrt of 1.25).
assert (mean error of fractional) is equal to 0.5."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn respects_large_integer_precision_in_squared_and_bias_aggregates() {
    let source = r#"let large be a list of 134217729.
assert (mse of large) is equal to 18014398777917440.
assert (mean squared error of large) is equal to (mean square of large).
assert (rmse of large) is equal to (root mean square of large).
let signed be a list of 9007199254740993.
assert (mbe of signed) is equal to (average of signed).
assert (mean bias error of signed) is equal to (mean error of signed)."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn malformed_types_empty_lists_and_overflows_produce_existing_errors() {
    let compiler = Compiler::new();
    for source in [
        "print mse of 2.",
        "print rmse of true.",
        "print mean squared error of 5.",
        "print root mean squared error of false.",
    ] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S365", "{source}: {error}");
        assert!(error.span.is_some());
    }
    for source in [
        "print mse of (a list of true).",
        "print root mean squared error of (a list of \"wrong\").",
    ] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S421", "{source}: {error}");
    }
    for source in [
        "print mean error of false.",
        "print mbe of (a list of \"wrong\").",
        "print mean bias error of 1.",
    ] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S353", "{source}: {error}");
    }
    for (source, code) in [
        ("print mse of (drop 1 from (a list of 1)).", "S354"),
        ("print rmse of (drop 1 from (a list of 1)).", "S354"),
        ("print mbe of (drop 1 from (a list of 1)).", "S354"),
        ("print mean squared error of (a list of 3037000500).", "S422"),
        ("print mean bias error of (a list of 9223372036854775807, 1).", "S354"),
    ] {
        let result = run(source);
        assert_eq!(result.status, 65, "{source}: {}", result.stderr);
        assert!(result.stderr.contains(code), "{source}: {}", result.stderr);
    }
}

#[test]
fn formatter_recognizes_contextual_phrases_but_preserves_identifiers() {
    let compiler = Compiler::new();
    let raw = "PRINT MSE OF (A LIST OF 3, 4).\nPRINT MEAN SQUARED ERROR OF (A LIST OF 3, 4).\nPRINT RMSE OF (A LIST OF 3, 4).\nPRINT ROOT MEAN SQUARED ERROR OF (A LIST OF 3, 4).\nPRINT MBE OF (A LIST OF 3, 4).\nPRINT MEAN BIAS ERROR OF (A LIST OF 3, 4).\nPRINT MEAN ERROR OF (A LIST OF 3, 4).";
    let formatted = "print mse of (a list of 3, 4).\nprint mean squared error of (a list of 3, 4).\nprint rmse of (a list of 3, 4).\nprint root mean squared error of (a list of 3, 4).\nprint mbe of (a list of 3, 4).\nprint mean bias error of (a list of 3, 4).\nprint mean error of (a list of 3, 4).";
    assert_eq!(compiler.format_source(raw).unwrap(), formatted);
    assert_eq!(compiler.format_source(formatted).unwrap(), formatted);
    let source = r#"let mse be 1.
let rmse be 2.
let mbe be 3.
let bias be 4.
let error be 5.
let squared be 6.
assert mse is equal to 1.
assert rmse is equal to 2.
assert mbe is equal to 3.
assert bias is equal to 4.
assert error is equal to 5.
assert squared is equal to 6."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    let unchanged = "LET Mse BE 1. PRINT Mse.";
    assert_eq!(compiler.format_source(unchanged).unwrap(), "let Mse be 1. print Mse.");
}

#[test]
fn desugaring_is_typed_optimizable_and_preserves_effects() {
    let compiler = Compiler::new();
    for (source, root, mapped) in [
        ("print mse of (a list of 1, 2).", "average", true),
        ("print mean squared error of (a list of 1, 2).", "average", true),
        ("print rmse of (a list of 1, 2).", "sqrt", true),
        ("print root mean squared error of (a list of 1, 2).", "sqrt", true),
        ("print mbe of (a list of 1, 2).", "average", false),
        ("print mean error of (a list of 1, 2).", "average", false),
        ("print mean bias error of (a list of 1, 2).", "average", false),
    ] {
        let ast = compiler.parse(source).unwrap().to_json().to_string();
        assert!(ast.contains(root), "{source}: {ast}");
        assert_eq!(ast.contains("map"), mapped, "{source}: {ast}");
        if mapped { assert!(ast.contains("square"), "{source}: {ast}"); }
        assert!(compiler.optimized_hir(source).is_ok(), "{source}");
        assert_eq!(compiler.effect_analysis(source).unwrap().effects(),
            vec!["console_write".to_string()]);
    }
}
