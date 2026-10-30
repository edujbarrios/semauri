// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{Compiler, run_cli};
const SOURCE: &str = include_str!("../examples/scientific_math.suri");

#[test]
fn exp_ln_sin_cos_execute_with_typed_lists() {
    Compiler::new().analyze(SOURCE).expect("scientific functions should type-check");
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/scientific_math.suri");
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(), path.into()]);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "scientific math OK\n");
}

#[test]
fn scientific_functions_check_types_and_finite_domains() {
    let compiler = Compiler::new();
    let wrong = compiler.compile("print exp of \"text\".", None).unwrap_err();
    assert_eq!(wrong.code, "S465");
    for expression in ["ln of 0", "ln of (0 minus 1)", "exp of 1000"] {
        let source = format!("print {expression}.");
        let error = compiler.compile(&source, None).unwrap_err();
        assert_eq!(error.code, "S466", "{}", expression);
    }
}
