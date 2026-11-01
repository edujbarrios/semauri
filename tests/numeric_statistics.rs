// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};

#[test]
fn variance_and_stdev_execute_with_numeric_lists() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/statistics.suri");
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(), path.into()]);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "variance OK\n");
}

#[test]
fn statistics_reject_non_numeric_and_overflowing_input() {
    let compiler = Compiler::new();
    let bad = compiler.compile("print variance of \"bad\".", None).unwrap_err();
    assert_eq!(bad.code, "S353");
    let overflow = compiler.compile("print stdev of (a list of 1e308, 0, 1e308).", None).unwrap_err();
    assert_eq!(overflow.code, "S469");
}
