// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-json-decode-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn decodes_typed_scalars_and_homogeneous_lists() {
    let source = r#"let text be decode json as string from "\"hello\"".
let parsed_number be decode json as number from "12.5".
let enabled be decode json as boolean from "true".
let names be decode json as strings from "[\"Ada\",\"Grace\"]".
let amounts be decode json as numbers from "[1,2,3]".
let flags be decode json as booleans from "[true,false]".
assert text is equal to "hello".
assert parsed_number is greater than 12.
assert enabled.
assert length of names is equal to 2.
assert (names at 1) is equal to "Grace".
assert sum of amounts is equal to 6.
assert (flags at 0) is equal to true.
assert (flags at 1) is equal to false.
print text."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "hello\n");
}

#[test]
fn supports_json_encoding_round_trip_and_empty_lists() {
    let source = r#"let original be a list of "alpha", "beta".
let decoded be decode json as strings from (json of original).
assert decoded is equal to original.
let empty be decode json as numbers from "[]".
assert length of empty is equal to 0."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn rejects_invalid_json_and_mismatched_shapes() {
    for input in [
        r#"print decode json as number from "\"42\""."#,
        r#"print decode json as number from "true"."#,
        r#"print decode json as strings from "[1,2]"."#,
        r#"print decode json as booleans from "[true,1]"."#,
        r#"print decode json as number from "null"."#,
        r#"print decode json as numbers from "{}"."#,
        r#"print decode json as number from "9223372036854775808"."#,
        r#"print decode json as number from "-9223372036854775809"."#,
        r#"print decode json as numbers from "[-9223372036854775809]"."#,
        r#"print decode json as strings from "[\"unclosed"."#,
    ] {
        let result = run(input);
        assert_eq!(result.status, 65, "{input}: {}", result.stderr);
        assert!(result.stderr.contains("S397"), "{input}: {}", result.stderr);
    }
}

#[test]
fn reports_static_type_error_for_non_string_json_inputs() {
    let compiler = Compiler::new();
    for code in [
        "print decode json as number from 7.",
        "print decode json as strings from (a list of \"one\", \"two\").",
    ] {
        let err = compiler.hir(code).unwrap_err();
        assert_eq!(err.code, "S396", "{code}: {err}");
        assert!(err.span.is_some());
    }
}

#[test]
fn preserves_contextual_identifiers_formatter_and_pure_effects() {
    let source = r#"let decode be "42".
assert (decode json as number from decode) is equal to 42."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    let compiler = Compiler::new();
    assert_eq!(compiler.format_source("PRINT DECODE JSON AS NUMBER FROM \"42\".").unwrap(),
        "print decode json as number from \"42\".");
    assert_eq!(compiler.effect_analysis("print decode json as boolean from \"true\".")
        .unwrap().effects(), vec!["console_write".to_string()]);
    let hir = compiler.optimized_hir("print decode json as number from \"25\".")
        .unwrap().to_json().to_string();
    assert!(hir.contains("25"), "{hir}");
    let list_hir = compiler.optimized_hir("print decode json as strings from \"[]\".")
        .unwrap().to_json().to_string();
    assert!(list_hir.contains("list"), "{list_hir}");
}
