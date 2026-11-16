// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-text-composition-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn word_count_and_selection_are_unicode_aware() {
    let source = r#"let message be "  Hello   ñ 🌍  ".
assert word count of message is equal to 3.
assert first word of message is equal to "Hello".
assert last word of message is equal to "🌍".
assert word count of "" is equal to 0.
assert word count of "  " is equal to 0.
print word count of message."#;
    let output = run(source);
    assert_eq!(output.status, 0, "{}", output.stderr);
    assert_eq!(output.stdout, "3\n");
}

#[test]
fn line_count_and_selection_follow_existing_lines_semantics() {
    let source = r#"let rows be "alpha\nbeta\nγ".
assert line count of rows is equal to 3.
assert first line of rows is equal to "alpha".
assert last line of rows is equal to "γ".
assert line count of "" is equal to 0.
print last line of rows."#;
    let output = run(source);
    assert_eq!(output.status, 0, "{}", output.stderr);
    assert_eq!(output.stdout, "γ\n");
}

#[test]
fn normalize_whitespace_collapses_unicode_spaces_without_data_loss() {
    let source = "let input be \" \\t Hello\\n  ñ\\t🌍 \".\nassert normalize whitespace of input is equal to \"Hello ñ 🌍\".\nassert normalize whitespace of \"  \" is equal to \"\".\nassert word count of (normalize whitespace of input) is equal to 3.\nprint normalize whitespace of input.";
    let output = run(source);
    assert_eq!(output.status, 0, "{}", output.stderr);
    assert_eq!(output.stdout, "Hello ñ 🌍\n");
}

#[test]
fn new_phrases_reuse_typed_operations_and_reject_wrong_types() {
    let compiler = Compiler::new();
    let tree = compiler.parse(
        r#"print first word of "hello world".
print line count of "one".
print normalize whitespace of "  hello  world "."#)
        .unwrap().to_json().to_string();
    assert!(tree.contains("words"), "{tree}");
    assert!(tree.contains("first"), "{tree}");
    assert!(tree.contains("lines"), "{tree}");
    assert!(tree.contains("length"), "{tree}");
    assert!(tree.contains("join"), "{tree}");
    assert!(compiler.optimized_hir(r#"print word count of "a b"."#).is_ok());
    let effects = compiler.effect_analysis(r#"print normalize whitespace of "  hello "."#).unwrap();
    assert_eq!(effects.effects(), vec!["console_write".to_string()]);
    for source in [
        "print word count of 42.",
        "print first word of true.",
        "print line count of (a list of 1, 2).",
        "print normalize whitespace of 17.",
    ] {
        assert!(compiler.hir(source).is_err(), "{source}");
    }
}

#[test]
fn formatter_canonicalizes_only_complete_phrases_and_preserves_names() {
    let compiler = Compiler::new();
    let raw = "PRINT FIRST WORD OF \"Hi there\".\nPRINT LAST LINE OF \"x\".\nPRINT WORD COUNT OF \"a b\".\nPRINT LINE COUNT OF \"x\".\nPRINT NORMALIZE WHITESPACE OF \" a \".";
    let formatted = compiler.format_source(raw).unwrap();
    assert_eq!(formatted,
        "print first word of \"Hi there\".\nprint last line of \"x\".\nprint word count of \"a b\".\nprint line count of \"x\".\nprint normalize whitespace of \" a \".\n");
    let identifiers = r#"let word be "name".
let line be "another".
let count be 7.
let normalize be "word".
let whitespace be "value".
assert word is equal to "name".
assert line is equal to "another".
assert count is equal to 7.
assert normalize is equal to "word".
assert whitespace is equal to "value"."#;
    compiler.compile(identifiers, None).unwrap();
}

#[test]
fn checked_in_text_composition_example_runs() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/text_composition.suri");
    let checked = run_cli(&["check".into(), path.into()]);
    assert_eq!(checked.status, 0, "{}", checked.stderr);
    let result = run_cli(&["run".into(), path.into()]);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Hello, Semauri!\n");
}
