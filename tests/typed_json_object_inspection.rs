// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-json-object-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn sorts_keys_and_distinguishes_null_from_missing() {
    let code = r#"let document be "{\"zeta\":null,\"alpha\":2,\"ñandú\":true}".
let names be keys of json document.
assert length of names is equal to 3.
assert (names at 0) is equal to "alpha".
assert (names at 1) is equal to "zeta".
assert (names at 2) is equal to "ñandú".
assert has json key "zeta" in document.
assert not (has json key "missing" in document).
print "valid"."#;
    let result = run(code);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "valid\n");
}

#[test]
fn empty_json_object_produces_an_empty_key_list() {
    let result = run(r#"let names be keys of json "{}".
assert length of names is equal to 0.
assert not (has json key "anything" in "{}")."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn rejects_non_object_and_malformed_documents() {
    for source in [
        "print keys of json \"[]\".",
        "print keys of json \"null\".",
        "print has json key \"x\" in \"[]\".",
        "print keys of json \"{\".",
    ] {
        let result = run(source);
        assert_eq!(result.status, 65, "{source}: {}", result.stderr);
        assert!(result.stderr.contains("S399"), "{source}: {}", result.stderr);
    }
}

#[test]
fn rejects_nonstring_operands_at_type_check() {
    let compiler = Compiler::new();
    for code in [
        "print keys of json 23.",
        "print has json key 7 in \"{}\".",
        "print has json key \"x\" in true.",
    ] {
        let error = compiler.hir(code).unwrap_err();
        assert_eq!(error.code, "S398", "{code}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn supports_contextual_names_and_constant_folding() {
    let source = r#"let keys be "{}".
let has be keys of json keys.
assert length of has is equal to 0."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    let compiler = Compiler::new();
    let formatted = compiler.format_source(
        "PRINT KEYS OF JSON \"{}\"."
    ).unwrap();
    assert_eq!(formatted, "print keys of json \"{}\".");
    assert_eq!(compiler.format_source("PRINT HAS JSON KEY \"x\" IN \"{}\".").unwrap(),
        "print has json key \"x\" in \"{}\".");
    let optimized = compiler.optimized_hir(
        "print has json key \"x\" in \"{}\"."
    ).unwrap().to_json().to_string();
    assert!(optimized.contains("false"), "{optimized}");
    assert_eq!(compiler.effect_analysis("print keys of json \"{}\".")
        .unwrap().effects(), vec!["console_write".to_string()]);
}
