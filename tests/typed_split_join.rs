// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-split-join-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(),
        "console_write".into(), path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn split_and_join_preserve_unicode_and_empty_fields() {
    let source = r#"let fields be split of "é,,三," by ",".
assert length of fields is equal to 4.
assert (fields at 0) is equal to "é".
assert (fields at 1) is equal to "".
assert (fields at 2) is equal to "三".
assert (fields at 3) is equal to "".
let rebuilt be join of fields with ",".
assert rebuilt is equal to "é,,三,".
assert (join of (words of "") with "-") is equal to "".
print rebuilt."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "é,,三,\n");
}

#[test]
fn rejects_wrong_static_types() {
    let compiler = Compiler::new();
    for source in [
        "print split of 1 by \",\".",
        "print split of \"one\" by 1.",
        "print join of (a list of 1, 2) with \",\".",
        "print join of \"abc\" with \",\".",
    ] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S392", "{source}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn empty_delimiter_returns_source_aware_error() {
    let result = run("print split of \"text\" by \"\".");
    assert_eq!(result.status, 65, "{}", result.stderr);
    assert!(result.stderr.contains("S393"), "{}", result.stderr);
}

#[test]
fn contextual_words_are_valid_identifiers() {
    let result = run(r#"let source_text be "a,b".
let parts be split of source_text by ",".
let join be parts.
assert (join of join with ",") is equal to source_text."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn parser_formatter_optimizer_and_effects_are_consistent() {
    let compiler = Compiler::new();
    let formatted = compiler.format_source(
        "PRINT JOIN OF (SPLIT OF \"a,b\" BY \",\" ) WITH \"|\"."
    ).unwrap();
    assert!(formatted.contains("join of"), "{formatted}");
    assert!(formatted.contains("split of"), "{formatted}");
    let optimized = compiler.optimized_hir("print join of (a list of \"a\", \"b\") with \".\".")
        .unwrap().to_json().to_string();
    assert!(optimized.contains("a.b"), "{optimized}");
    assert_eq!(compiler.effect_analysis("print join of (words of \"a b\") with \",\".")
        .unwrap().effects(), vec!["console_write".to_string()]);
}
