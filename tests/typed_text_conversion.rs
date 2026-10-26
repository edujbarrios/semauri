// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-conversion-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn parses_numeric_and_boolean_text_without_implicit_coercion() {
    let source = r#"let integer be parse number from "-42".
let decimal be parse number from " 12.5 ".
let exponent be parse number from "1e3".
let enabled be parse boolean from " TRUE ".
let disabled be parse boolean from "false".
assert integer is less than 0.
assert decimal is greater than 12.
assert exponent is greater than 999.
assert enabled.
assert not disabled.
assert (parse number from "9223372036854775807") is greater than 0.
print decimal."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "12.5\n");
}

#[test]
fn malformed_nonfinite_and_oversized_text_has_located_errors() {
    for source in [
        "print parse number from \"\".",
        "print parse number from \"NaN\".",
        "print parse number from \"1e999\".",
        "print parse number from \"9223372036854775808\".",
        "print parse number from \"hello\".",
        "print parse boolean from \"maybe\".",
    ] {
        let result = run(source);
        assert_eq!(result.status, 65, "{source}: {}", result.stderr);
        assert!(result.stderr.contains("S395"), "{source}: {}", result.stderr);
    }
}

#[test]
fn nonstring_operands_are_rejected_before_execution() {
    let compiler = Compiler::new();
    for source in [
        "print parse number from 123.",
        "print parse boolean from true.",
        "print parse number from (a list of \"1\", \"2\").",
    ] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S394", "{source}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn parse_is_still_a_usable_identifier() {
    let result = run(r#"let parse be " 18 ".
let number_value be parse number from parse.
assert number_value is equal to 18."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn formatter_optimizer_and_effects_stay_consistent() {
    let compiler = Compiler::new();
    assert_eq!(compiler.format_source("PRINT PARSE NUMBER FROM \"12\".").unwrap(),
        "print parse number from \"12\".");
    assert_eq!(compiler.format_source("PRINT PARSE BOOLEAN FROM \"TRUE\".").unwrap(),
        "print parse boolean from \"TRUE\".");
    let folded = compiler.optimized_hir("print parse number from \"25\".")
        .unwrap().to_json().to_string();
    assert!(folded.contains("25"), "{folded}");
    assert_eq!(compiler.effect_analysis("print parse boolean from \"true\".")
        .unwrap().effects(), vec!["console_write".to_string()]);
}
