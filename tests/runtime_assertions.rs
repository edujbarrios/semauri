// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

fn program(source: &str) -> PathBuf {
    let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    let file = std::env::temp_dir().join(format!(
        "semauri-assert-{}-{id}.suri",
        std::process::id()
    ));
    fs::write(&file, source).unwrap();
    file
}

fn run(source: &str, allowed: bool) -> semauri::CliResult {
    let file = program(source);
    let mut args = vec!["run".into()];
    if allowed {
        args.extend(["--allow".into(), "console_write".into()]);
    }
    args.push(file.to_string_lossy().to_string());
    let result = run_cli(&args);
    fs::remove_file(file).unwrap();
    result
}

#[test]
fn passing_assertions_execute_without_any_effect_permission() {
    let source = "Let expected be 6 times 7. Assert expected is equal to 42. Assert not false.";
    assert!(Compiler::new().effect_analysis(source).unwrap().pure());
    let result = run(source, false);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "");
}

#[test]
fn failed_assertion_returns_runtime_status_and_source_location() {
    let result = run("Let value be 10.\nAssert value is less than 5.", false);
    assert_eq!(result.status, 70);
    assert!(result.stdout.is_empty());
    assert!(result.stderr.contains("S502"), "{}", result.stderr);
    assert!(result.stderr.contains("at 2:1"), "{}", result.stderr);
    assert!(result.stderr.contains("Assertion failed"), "{}", result.stderr);
}

#[test]
fn assertions_accept_only_boolean_expressions() {
    let error = Compiler::new().hir("Assert 42.").unwrap_err();
    assert_eq!(error.code, "S329");
    assert!(error.message.contains("expects boolean"), "{error}");

    let source = "Assert 1 is equal to 1.";
    let plan = Compiler::new().runtime_plan(source).unwrap();
    assert_eq!(plan.operations[0].name, "assert");
    assert!(plan.operations[0].effects.is_empty());
}

#[test]
fn assertions_and_prints_keep_order_and_effect_gates() {
    let source = "Assert true. Print \"verified\". Assert 2 is greater than 1.";
    let denied = run(source, false);
    assert_eq!(denied.status, 65);
    assert!(denied.stderr.contains("S334"));
    let allowed = run(source, true);
    assert_eq!(allowed.status, 0, "{}", allowed.stderr);
    assert_eq!(allowed.stdout, "verified\n");
}

#[test]
fn assertions_do_not_turn_a_runtime_failure_into_a_compile_error() {
    let source = "Assert false.";
    let compiler = Compiler::new();
    assert!(compiler.compile(source, None).is_ok());
    assert_eq!(run(source, false).status, 70);
}
