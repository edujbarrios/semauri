// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{Compiler, run_cli};
const SOURCE: &str = include_str!("../examples/scientific_math.suri");

#[test]
fn scientific_functions_execute_with_typed_lists() {
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
    for expression in [
        "ln of 0", "ln of (0 minus 1)", "exp of 1000",
        "log2 of 0", "log10 of (minus 1)",
        "asin of 1.1", "asin of (0 minus 1.1)",
        "acos of 2", "acos of (0 minus 2)",
    ] {
        let source = format!("print {expression}.");
        let error = compiler.compile(&source, None).unwrap_err();
        assert_eq!(error.code, "S466", "{}", expression);
    }
}

#[test]
fn expanded_scientific_operators_typecheck_and_format_contextually() {
    let compiler = Compiler::new();
    for operation in ["log2", "log10", "asin", "acos", "atan"] {
        let wrong = compiler.compile(&format!("print {operation} of true."), None)
            .unwrap_err();
        assert_eq!(wrong.code, "S465", "{operation}");
        assert!(wrong.span.is_some(), "{operation}");
        let source = format!("print {operation} of 0.");
        assert!(compiler.hir(&source).is_ok(), "{operation}");
        assert!(compiler.optimized_hir(&source).is_ok(), "{operation}");
        assert_eq!(compiler.effect_analysis(&source).unwrap().effects(),
            vec!["console_write".to_string()]);
        let upper = format!("PRINT {} OF 1.", operation.to_ascii_uppercase());
        let lower = format!("print {operation} of 1.");
        assert_eq!(compiler.format_source(&upper).unwrap(), lower);
    }

    // Contextual operators do not reserve otherwise usable variable names.
    let result = compiler.compile("let log10 be 7. let asin be 2. assert log10 is equal to 7. assert asin is equal to 2.", None);
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn inverse_trigonometry_supports_boundary_values_and_negative_arguments() {
    let compiler = Compiler::new();
    compiler.compile(r#"
let left be asin of (minus 1).
let right be asin of 1.
let upper be acos of (minus 1).
assert left is less than (minus 1.5).
assert right is greater than 1.5.
assert upper is greater than 3.
assert (atan of (minus 1)) is less than 0.
"#, None).unwrap();
}

#[test]
fn hyperbolic_math_checks_domains_and_types() {
    let compiler = Compiler::new();
    for name in ["sinh", "cosh", "tanh"] {
        assert_eq!(compiler.compile(&format!("print {name} of true."), None).unwrap_err().code, "S465");
        if name != "tanh" { assert_eq!(compiler.compile(&format!("print {name} of 1000."), None).unwrap_err().code, "S466"); }
    }
    compiler.compile("assert sinh of 0 is equal to 0.0. assert cosh of 0 is equal to 1.0. assert tanh of 0 is equal to 0.0.", None).unwrap();
}
