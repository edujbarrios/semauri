// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-reverse-{}-{}.sema", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&[
        "run".into(), "--allow".into(), "console_write".into(), path.to_string_lossy().into_owned(),
    ]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn reverses_strings_and_lists_without_mutating_inputs() {
    let output = run(r#"let word be "Semauri".
let numbers be a list of 1, 2, 3.
let reversed be reverse of numbers.
assert (numbers at 0) is equal to 1.
assert (reversed at 0) is equal to 3.
assert ((reverse of reversed) at 0) is equal to 1.
print reverse of word.
print reversed.
print numbers."#);
    assert_eq!(output.status, 0, "{}", output.stderr);
    assert_eq!(output.stdout, "irua meS\n[3, 2, 1]\n[1, 2, 3]\n".replace("irua meS", "iruameS"));
}

#[test]
fn reverses_unicode_scalars_and_compounds_with_case_conversion() {
    let output = run("print reverse of \"ñ🌍a\". print uppercase of (reverse of \"abc\").");
    assert_eq!(output.status, 0, "{}", output.stderr);
    assert_eq!(output.stdout, "a🌍ñ\nCBA\n");
}

#[test]
fn statically_rejects_non_string_non_list() {
    for source in ["print reverse of 42.", "print reverse of true.", "print reverse of red."] {
        let e = Compiler::new().hir(source).unwrap_err();
        assert_eq!(e.code, "S352", "{source}");
        assert!(e.span.is_some());
    }
}

#[test]
fn formatter_optimizer_and_effects_integrate() {
    let compiler = Compiler::new();
    assert_eq!(
        compiler.format_source("PRINT REVERSE OF \"ABC\".").unwrap(),
        "print reverse of \"ABC\"."
    );
    let optimized = compiler.optimized_hir("print reverse of \"abc\".")
        .unwrap().to_json().to_string();
    assert!(optimized.contains("cba"));
    assert_eq!(
        compiler.effect_analysis("print reverse of \"abc\".").unwrap().effects(),
        vec!["console_write".to_string()]
    );
}
