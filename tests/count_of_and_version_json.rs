// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use serde_json::Value;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-count-of-{}-{}.suri",
        std::process::id(), NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&[
        "run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()
    ]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn count_of_matches_typed_list_and_unicode_string_length() {
    let result = run(r#"let count be 9.
let values be a list of 4, 5, 6.
let total be count of values.
assert total is equal to 3.
assert (count of "ñ🌍") is equal to 2.
assert count is equal to 9.
print total.
print count of "ñ🌍"."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "3\n2\n");
}

#[test]
fn count_of_preserves_length_type_errors_and_optimizer() {
    let compiler = Compiler::new();
    for invalid in ["42", "true", "red"] {
        let alias = compiler.hir(&format!("print count of {invalid}.")).unwrap_err();
        let canonical = compiler.hir(&format!("print length of {invalid}.")).unwrap_err();
        assert_eq!(alias.code, canonical.code);
        assert_eq!(alias.code, "S343");
        assert!(alias.span.is_some());
    }
    let result = compiler.optimized_hir(r#"print count of "ñ🌍"."#)
        .unwrap().to_json().to_string();
    assert!(result.contains("2"), "{result}");
    assert_eq!(
        compiler.effect_analysis(r#"print count of "ñ🌍"."#).unwrap().effects(),
        vec!["console_write".to_owned()]
    );
}

#[test]
fn contextual_count_and_formatter_preserve_existing_syntax() {
    let compiler = Compiler::new();
    assert_eq!(
        compiler.format_source("LET Count BE 9. PRINT Count. PRINT COUNT OF \"abc\".").unwrap(),
        "let Count be 9. print Count. print count of \"abc\"."
    );
    let result = run(r#"let count be 2.
let xs be a list of 1, 2, 3.
assert (count of xs) is equal to 3.
assert (count every x in xs if (x is greater than count)) is equal to 1.
print count."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "2\n");
}

#[test]
fn version_json_is_stable_and_rejects_invalid_arguments() {
    let result = run_cli(&["version".into(), "--json".into()]);
    assert_eq!(result.status, 0, "{}", result.stderr);
    let body: Value = serde_json::from_str(&result.stdout).unwrap();
    assert_eq!(body["name"], "semauri");
    assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
    assert!(result.stderr.is_empty());

    for alias in ["--version", "-v"] {
        let result = run_cli(&[alias.into(), "--json".into()]);
        assert_eq!(result.status, 0, "{alias}: {}", result.stderr);
        let body: Value = serde_json::from_str(&result.stdout).unwrap();
        assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
    }

    let plain = run_cli(&["version".into()]);
    assert_eq!(plain.stdout, format!("Semauri {}\n", env!("CARGO_PKG_VERSION")));

    for invalid in [
        vec!["version".into(), "--unknown".into()],
        vec!["version".into(), "--json".into(), "extra".into()],
    ] {
        let result = run_cli(&invalid);
        assert_ne!(result.status, 0);
        assert!(result.stderr.contains("Usage: semauri version [--json]"));
    }
}

#[test]
fn documented_example_runs() {
    let result = run(include_str!("../examples/count_of.suri"));
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Count operators OK\n");
}
