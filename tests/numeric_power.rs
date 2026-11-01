// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{Compiler, run_cli};

#[test]
fn power_executes_with_lists_and_exact_integer_results() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/power.suri");
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(), path.into()]);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "power OK\n");
}

#[test]
fn power_checks_overflow_domains_and_types() {
    let compiler = Compiler::new();
    for source in [
        "print power of 2 to 63.",
        "print power of (0 minus 1) to 0.5.",
        "print power of 0 to (0 minus 1).",
    ] {
        let error = compiler.compile(source, None).unwrap_err();
        assert_eq!(error.code, "S467", "{source}");
    }
    let error = compiler.compile("print power of \"nope\" to 2.", None).unwrap_err();
    assert_eq!(error.code, "S314");
}
