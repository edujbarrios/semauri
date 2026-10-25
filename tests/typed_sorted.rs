// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-sorted-{}-{}.sema",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)
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
fn sorts_numbers_without_mutating_the_original_list() {
    let result = run(r#"let input be a list of 4, minus 2, 3, 1.
let output be sorted of input.
assert (input at 0) is equal to 4.
assert (output at 0) is equal to minus 2.
assert (output at 3) is equal to 4.
assert (sorted of (reverse of output) at 0) is equal to minus 2.
print output.
print input."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[-2, 1, 3, 4]\n[4, -2, 3, 1]\n");
}

#[test]
fn sorts_string_and_boolean_lists() {
    let result = run(r#"print sorted of a list of "zebra", "Apple", "ant".
print sorted of a list of true, false, true."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[Apple, ant, zebra]\n[false, true, true]\n");
}

#[test]
fn numeric_sort_preserves_i64_and_mixed_float_order() {
    let result = run(
        "print sorted of a list of 9007199254740995, 9007199254740992.0, 9007199254740993."
    );
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(
        result.stdout,
        "[9007199254740992.0, 9007199254740993, 9007199254740995]\n"
    );
    let result = run(
        "print median of a list of 9007199254740995, 9007199254740992.0, 9007199254740993."
    );
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "9007199254740993\n");
}

#[test]
fn rejects_unsupported_operands_with_located_diagnostic() {
    for source in [
        "print sorted of 9.",
        "print sorted of \"hello\".",
        "print sorted of a list of red, blue.",
    ] {
        let error = Compiler::new().hir(source).unwrap_err();
        assert_eq!(error.code, "S359", "{source}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn sorted_is_recognized_by_formatter_hir_and_effect_analysis() {
    let compiler = Compiler::new();
    assert_eq!(
        compiler.format_source("PRINT SORTED OF a LIST OF 2, 1.").unwrap(),
        "print sorted of a list of 2, 1."
    );
    assert!(compiler.hir("let x be sorted of a list of true, false.")
        .unwrap().to_json().to_string().contains("\"sorted\""));
    assert_eq!(
        compiler.effect_analysis("print sorted of a list of 2, 1.")
            .unwrap().effects(),
        vec!["console_write".to_owned()]
    );
}
