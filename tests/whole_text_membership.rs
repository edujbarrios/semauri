// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-whole-text-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn exact_whole_words_and_lines_are_unicode_aware() {
    let result = run(r#"let message be "Ada  ñ 🌍!\r\nAda Lovelace\n".
assert message has word "Ada".
assert message has word "ñ".
assert message has word "🌍!".
assert message does not have word "Ad".
assert message does not have word "🌍".
assert message has line "Ada  ñ 🌍!".
assert message has line "Ada Lovelace".
assert message does not have line "Lovelace".
assert "foobar" contains "foo".
assert "foobar" does not have word "foo".
assert "" does not have word "".
assert "" does not have line "".
print "ok"."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "ok\n");
}

#[test]
fn matching_reuses_typed_pure_operators_and_rejects_bad_operands() {
    let compiler = Compiler::new();
    let tree = compiler.parse(
        r#"print "hello world" has word "hello".
print "a\nb" does not have line "c"."#)
        .unwrap().to_json().to_string();
    assert!(tree.contains("words"), "{tree}");
    assert!(tree.contains("lines"), "{tree}");
    assert!(tree.contains("contains"), "{tree}");
    assert!(tree.contains("not"), "{tree}");
    compiler.optimized_hir(r#"assert "hello" has word "hello"."#).unwrap();
    let effects = compiler.effect_analysis(
        r#"print "hello world" has word "hello"."#).unwrap();
    assert_eq!(effects.effects(), vec!["console_write".to_owned()]);
    for source in [
        r#"assert 42 has word "42"."#,
        r#"assert true has line "true"."#,
        r#"assert "hello" has word 7."#,
        r#"assert "a\nb" does not have line false."#,
    ] {
        assert!(compiler.hir(source).is_err(), "{source}");
    }
}

#[test]
fn formatter_canonicalizes_predicates_but_preserves_contextual_names() {
    let compiler = Compiler::new();
    let raw = r#"LET has BE "value".
LET word BE "Ada".
LET line BE "row".
ASSERT has IS EQUAL TO "value".
ASSERT word IS EQUAL TO "Ada".
ASSERT line IS EQUAL TO "row".
ASSERT "Ada Lovelace" HAS WORD "Ada".
ASSERT "Ada Lovelace" DOES NOT HAVE LINE "Ada"."#;
    let expected = r#"let has be "value".
let word be "Ada".
let line be "row".
assert has is equal to "value".
assert word is equal to "Ada".
assert line is equal to "row".
assert "Ada Lovelace" has word "Ada".
assert "Ada Lovelace" does not have line "Ada"."#;
    let formatted = compiler.format_source(raw).unwrap();
    assert_eq!(formatted, expected);
    assert_eq!(compiler.format_source(&formatted).unwrap(), expected);
    compiler.compile(expected, None).unwrap();
}

#[test]
fn checked_in_example_checks_and_runs() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/whole_text_membership.suri");
    let checked = run_cli(&["check".into(), path.into()]);
    assert_eq!(checked.status, 0, "{}", checked.stderr);
    let output = run_cli(&["run".into(), path.into()]);
    assert_eq!(output.status, 0, "{}", output.stderr);
    assert_eq!(output.stdout, "Whole-text matching OK\n");
}
