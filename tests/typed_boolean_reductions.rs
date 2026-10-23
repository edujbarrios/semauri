// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-bool-reductions-{}-{}.sema",
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
fn any_and_all_are_typed_pure_and_composable() {
    let result = run(r#"let flags be a list of true, false, true.
assert any of flags is equal to true.
assert all of flags is equal to false.
assert any of (reverse of flags) is equal to true.
assert (flags at 0) is equal to true.
print any of flags.
print all of flags."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "true\nfalse\n");
}

#[test]
fn correctly_handles_all_true_and_all_false_inputs() {
    let result = run(
        "assert all of a list of true, true.          assert any of a list of false, false is equal to false.          print all of a list of true, true.          print any of a list of false, false."
    );
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "true\nfalse\n");
}

#[test]
fn rejects_non_boolean_lists_or_scalars() {
    let compiler = Compiler::new();
    for source in [
        "print any of 1.",
        "print all of true.",
        "print any of a list of 1, 2.",
        "print all of a list of \"a\", \"b\".",
    ] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S356", "{source}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn fmt_ast_hir_and_effects_recognize_boolean_reductions() {
    let compiler = Compiler::new();
    assert_eq!(
        compiler.format_source("PRINT ANY OF a LIST OF TRUE, FALSE.").unwrap(),
        "print any of a list of true, false."
    );
    let hir = compiler.hir("let flags be all of a list of true, false.").unwrap();
    assert!(hir.to_json().to_string().contains("\"all\""));
    assert_eq!(
        compiler.effect_analysis("print any of a list of false, true.")
            .unwrap().effects(),
        vec!["console_write".to_owned()]
    );
}
