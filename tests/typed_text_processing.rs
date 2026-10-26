// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-text-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
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
fn unicode_text_cleaning_and_tokenization() {
    let source = r#"let cleaned be trim of "  \t ¡Hola, Ñandú!  \n".
let tokens be words of "uno  \t dos\n三".
let rows be lines of "primera\r\n\r\ntercera\n".
assert cleaned is equal to "¡Hola, Ñandú!".
assert length of tokens is equal to 3.
assert (tokens at 0) is equal to "uno".
assert (tokens at 2) is equal to "三".
assert length of rows is equal to 3.
assert (rows at 0) is equal to "primera".
assert (rows at 1) is equal to "".
assert (rows at 2) is equal to "tercera".
print cleaned."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "¡Hola, Ñandú!\n");
}

#[test]
fn handles_empty_strings_and_whitespace() {
    let source = r#"let empty_trim be trim of "".
let empty_words be words of " \t\n ".
let empty_lines be lines of "".
assert empty_trim is equal to "".
assert length of empty_words is equal to 0.
assert length of empty_lines is equal to 0.
assert length of (words of "é") is equal to 1."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn operator_names_remain_ordinary_identifiers() {
    let source = r#"let words be "  ok ".
let lines be words.
let trim be lines.
assert (trim of trim) is equal to "ok".
assert ((words of lines) at 0) is equal to "ok"."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn text_operations_reject_non_string_types() {
    let compiler = Compiler::new();
    for source in [
        "print trim of 5.",
        "print words of true.",
        "print lines of a list of 1, 2.",
    ] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S351", "{source}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn constant_folding_and_formatter_support_contextual_words() {
    let compiler = Compiler::new();
    let optimized = compiler.optimized_hir(
        "let values be words of \"  uno dos  \". print length of values."
    ).unwrap();
    assert!(optimized.changes >= 1);
    let optimized_json = optimized.to_json().to_string();
    assert!(optimized_json.contains(r#""value":2"#), "{optimized_json}");
    let text = "PRINT TRIM OF \" Mixed Case \". PRINT WORDS OF \"A B\". PRINT LINES OF \"a\\nb\".";
    let formatted = compiler.format_source(text).unwrap();
    assert_eq!(formatted, "print trim of \" Mixed Case \". print words of \"A B\". print lines of \"a\\nb\".");
    assert_eq!(
        compiler.effect_analysis("print trim of \" yes \".").unwrap().effects(),
        vec!["console_write".to_string()]
    );
}
