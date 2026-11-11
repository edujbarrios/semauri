// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
//! Immutable dictionaries with uniform value types and string keys.
use semauri::{run_cli, Compiler};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn fixture(source: &str) -> PathBuf {
    let file = std::env::temp_dir().join(format!(
        "semauri-dicts-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&file, source).unwrap();
    file
}

fn command(cmd: &str, source: &str) -> semauri::CliResult {
    let file = fixture(source);
    let result = if cmd == "run" {
        run_cli(&["run".into(), "--allow".into(), "console_write".into(),
            file.to_string_lossy().into_owned()])
    } else {
        run_cli(&[cmd.into(), file.to_string_lossy().into_owned()])
    };
    fs::remove_file(file).unwrap();
    result
}

#[test]
fn dictionaries_are_typed_and_print_stable_json() {
    let source = r#"let scores be dictionary of ("two": 2, "one": 1).
assert (entry "one" of scores) is equal to 1.
print entry "two" of scores.
print scores.
print json of scores.
"#;
    for cmd in ["check", "ast", "hir", "effects"] {
        let result = command(cmd, source);
        assert_eq!(result.status, 0, "{cmd}: {}", result.stderr);
    }
    let result = command("run", source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout,
        "2\n{\"one\":1,\"two\":2}\n{\"one\":1,\"two\":2}\n");
    let hir = Compiler::new().hir(source).unwrap().to_json().to_string();
    assert!(hir.contains("dictionary"), "{hir}");
}

#[test]
fn nested_dictionaries_and_lists_are_composable() {
    let source = r#"let data be a dictionary of ("first": dictionary of ("city": "Barcelona"), "second": dictionary of ("city": "Madrid")).
assert (entry "city" of entry "first" of data) is equal to "Barcelona".
let batches be dictionary of ("left": (a list of 1, 2), "right": (a list of 3, 4)).
print entry "city" of entry "second" of data.
print json of batches.
"#;
    let result = command("run", source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout,
        "Madrid\n{\"left\":[1,2],\"right\":[3,4]}\n");
}

#[test]
fn dictionary_types_are_first_class_function_parameter_types() {
    let source = r#"define function get_score taking scores as dictionary of number to entry "primary" of scores.
let scores be dictionary of ("primary": 42, "secondary": 18).
print invoke get_score to scores.
"#;
    let result = command("run", source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "42\n");
    let wrong = r#"define function get_score taking scores as dictionary of number to entry "primary" of scores.
let scores be dictionary of ("primary": "forty-two").
print invoke get_score to scores.
"#;
    let result = command("check", wrong);
    assert_eq!(result.status, 65, "{}", result.stderr);
    assert!(result.stderr.contains("S370"), "{}", result.stderr);
}

#[test]
fn invalid_dictionary_literals_report_errors() {
    for (source, code) in [
        (r#"let data be dictionary of (). print data."#, "S420"),
        (r#"let data be dictionary of ("x": 1, "x": 2). print data."#, "S420"),
        (r#"let data be dictionary of ("": 1). print data."#, "S420"),
        (r#"let data be dictionary of ("x" 1). print data."#, "S420"),
        (r#"let data be dictionary of ("x": 1, "y": "string"). print data."#, "S421"),
        (r#"let data be record of ("x": 1). print entry "x" of data."#, "S421"),
    ] {
        let result = command("check", source);
        assert_eq!(result.status, 65, "{source}: {}", result.stderr);
        assert!(result.stderr.contains(code), "{source}: {}", result.stderr);
    }
}

#[test]
fn missing_dictionary_keys_are_reported_not_silently_defaulted() {
    let source = r#"let data be dictionary of ("present": 1).
print entry "missing" of data.
"#;
    // The compiler evaluates known immutable dictionaries while checking,
    // so a missing literal key is rejected before the executable starts.
    for verb in ["check", "run"] {
        let result = command(verb, source);
        assert_eq!(result.status, 65, "{}", result.stderr);
        assert!(result.stderr.contains("S421"), "{}", result.stderr);
        assert!(result.stderr.contains("missing"), "{}", result.stderr);
    }
}

#[test]
fn dictionary_keys_are_case_sensitive_and_only_strings() {
    let source = r#"let data be dictionary of ("Name": "Ada", "name": "Grace",).
print entry "Name" of data.
print entry "name" of data.
"#;
    let result = command("run", source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Ada\nGrace\n");
}

#[test]
fn dictionary_entries_have_explicit_limits() {
    let entries = (0..65).map(|i| format!("\"k{i}\": {i}")).collect::<Vec<_>>().join(", ");
    let source = format!("let data be dictionary of ({entries}).\nprint data.\n");
    let result = command("check", &source);
    assert_eq!(result.status, 65, "{}", result.stderr);
    assert!(result.stderr.contains("S420"), "{}", result.stderr);
}

#[test]
fn contextual_dictionary_word_does_not_break_existing_identifiers() {
    let source = "let dictionary be 3.\nlet item be dictionary plus 2.\nprint item.\n";
    assert!(Compiler::new().analyze(source).is_ok());
}

#[test]
fn dictionaries_preserve_effect_permissions_and_need_no_network() {
    let source = r#"let settings be dictionary of ("timeout": 30, "retries": 2).
assert (entry "timeout" of settings) is equal to 30.
"#;
    let effects = command("effects", source);
    assert_eq!(effects.status, 0, "{}", effects.stderr);
    assert!(!effects.stdout.contains("network"), "{}", effects.stdout);
    assert!(!effects.stdout.contains("filesystem_write"), "{}", effects.stdout);
}

#[test]
fn formatter_recognizes_dictionary_grammar_without_mutating_keys() {
    let source = "LET values BE DICTIONARY OF (\"Key\": 4).\nPRINT ENTRY \"Key\" OF values.\n";
    let formatted = command("fmt", source);
    assert_eq!(formatted.status, 0, "{}", formatted.stderr);
    assert!(formatted.stdout.contains("dictionary of (\"Key\": 4)"),
        "{}", formatted.stdout);
    assert!(formatted.stdout.contains("entry \"Key\" of values"),
        "{}", formatted.stdout);
}
