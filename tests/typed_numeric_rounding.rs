// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-rounding-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
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
fn rounds_toward_negative_and_positive_infinity() {
    let result = run(r#"assert (floor of 2.9) is equal to 2.
assert (ceiling of 2.1) is equal to 3.
assert (floor of (minus 2.1)) is equal to (minus 3).
assert (ceiling of (minus 2.9)) is equal to (minus 2).
assert (floor of 3) is equal to 3.
assert (ceiling of 3.0) is equal to 3.
assert (floor of 0.0) is equal to 0.
assert (ceiling of (minus 0.2)) is equal to 0.
print "rounded"."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "rounded\n");
}

#[test]
fn preserves_exact_large_and_boundary_integers() {
    let result = run(r#"let large be 9007199254740993.
let lowest be (minus 9223372036854775807) minus 1.
assert (floor of large) is equal to large.
assert (ceiling of large) is equal to large.
assert (floor of lowest) is equal to lowest.
assert (ceiling of lowest) is equal to lowest."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn composes_with_list_transformations_and_functions() {
    let result = run(r#"define function round_down taking value as number to (floor of value).
let values be a list of 1.2, minus 0.4, 5.9.
let floors be map every n in values to (invoke round_down to n).
assert (floors at 0) is equal to 1.
assert (floors at 1) is equal to (minus 1).
assert (floors at 2) is equal to 5."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn rejects_nonnumeric_operands_with_located_diagnostics() {
    let compiler = Compiler::new();
    for source in [
        "print floor of true.",
        "print ceiling of \"text\".",
        "print floor of a list of 1, 2.",
    ] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S419", "{source}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn contextual_identifiers_formatter_optimizer_and_effects_work() {
    let compiler = Compiler::new();
    let result = run("let floor be 9. let ceiling be 3. print floor. print ceiling.");
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "9\n3\n");
    assert_eq!(
        compiler.format_source("PRINT FLOOR OF (MINUS 2.1).").unwrap(),
        "print floor of (minus 2.1)."
    );
    assert_eq!(
        compiler.format_source("PRINT CEILING OF 4.2.").unwrap(),
        "print ceiling of 4.2."
    );
    assert_eq!(
        compiler.format_source("LET Floor BE 4. PRINT Floor.").unwrap(),
        "let Floor be 4. print Floor."
    );
    for source in ["print floor of 2.3.", "print ceiling of (minus 2.3)."] {
        assert!(compiler.hir(source).is_ok());
        assert!(compiler.optimized_hir(source).is_ok());
        assert_eq!(
            compiler.effect_analysis(source).unwrap().effects(),
            vec!["console_write".to_string()]
        );
    }
}
