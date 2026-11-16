// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

//! Regression tests for the README's real, executable machine-learning example.
use semauri::{run_cli, Compiler};

const SOURCE: &str = include_str!("../examples/linear_regression.suri");
const EXPECTED_OUTPUT: &str = concat!(
    "Linear regression (ordinary least squares)\n",
    "Training observations:\n5\n",
    "Learned slope:\n2.0\n",
    "Learned intercept:\n1\n",
    "Prediction for x=6:\n13\n",
);

#[test]
fn showcase_is_compilable_and_emits_the_documented_prediction() {
    Compiler::new().analyze(SOURCE).expect("OLS demo must type-check");
    let output = run_cli(&[
        "run".into(),
        "--allow".into(),
        "console_write".into(),
        concat!(env!("CARGO_MANIFEST_DIR"), "/examples/linear_regression.suri").into(),
    ]);
    assert_eq!(output.status, 0, "{}", output.stderr);
    assert_eq!(output.stdout, EXPECTED_OUTPUT);
}

#[test]
fn showcase_prints_by_default_and_supports_strict_denial() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/linear_regression.suri");
    let allowed = run_cli(&["run".into(), path.into()]);
    assert_eq!(allowed.status, 0, "{}", allowed.stderr);
    assert_eq!(allowed.stdout, EXPECTED_OUTPUT);
    let denied = run_cli(&["run".into(), "--deny-console".into(), path.into()]);
    assert_eq!(denied.status, 65);
    assert!(denied.stderr.contains("S334"), "{}", denied.stderr);
}
