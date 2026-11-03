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

#[test]
fn natural_for_loops_accept_optional_every_and_each() {
    let compiler = Compiler::new();
    for quantifier in ["", "every ", "each "] {
        let source = format!("let values be a list of 1, 2, 3. for {quantifier}n in values: assert n is greater than 0. end.");
        compiler.compile(&source, None).expect("natural iteration should compile");
    }
}

#[test]
fn extended_scientific_math_checks_safe_values_and_typed_lists() {
    let c = Compiler::new();
    let source = r#"
assert (asinh of 0) is equal to 0.0.
assert (acosh of 1) is equal to 0.0.
assert (atanh of 0) is equal to 0.0.
assert (log1p of 0) is equal to 0.0.
assert (expm1 of 0) is equal to 0.0.
assert (asinh of (minus 2)) is less than (minus 1).
assert (acosh of 2) is greater than 1.
assert (atanh of 0.5) is greater than 0.5.
let xs be a list of 0, 0.1, 0.2.
let ys be map every x in xs to (log1p of x).
let zs be map every x in xs to (expm1 of x).
assert length of ys is equal to 3.
assert length of zs is equal to 3.
"#;
    c.compile(source, None).unwrap();
    assert!(c.optimized_hir(source).is_ok());
    assert!(c.hir(source).is_ok());
}

#[test]
fn extended_scientific_math_checks_errors_and_contextual_formatting() {
    let c = Compiler::new();
    for name in ["asinh", "acosh", "atanh", "log1p", "expm1"] {
        let source = format!("print {name} of true.");
        let error = c.hir(&source).unwrap_err();
        assert_eq!(error.code, "S465", "{name}");
        assert!(error.span.is_some());
        let uppercase = format!("PRINT {} OF 0.", name.to_uppercase());
        assert_eq!(c.format_source(&uppercase).unwrap(), format!("print {name} of 0."));
        let valid = if name == "acosh" { 1 } else { 0 };
        assert_eq!(c.effect_analysis(&format!("print {name} of {valid}.")).unwrap().effects(),
            vec!["console_write".to_string()]);
    }
    for expr in [
        "acosh of 0",
        "acosh of (minus 2)",
        "atanh of 1",
        "atanh of (minus 1)",
        "atanh of 2",
        "log1p of (minus 1)",
        "log1p of (minus 2)",
        "expm1 of 1000",
    ] {
        let error = c.compile(&format!("print {expr}."), None).unwrap_err();
        assert_eq!(error.code, "S466", "{expr}");
    }
    c.compile("let atanh be 7. let log1p be 8. assert atanh is equal to 7. assert log1p is equal to 8.", None).unwrap();
}
