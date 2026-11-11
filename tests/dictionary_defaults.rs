// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
//! Recoverable dictionary entry lookup with a statically checked fallback.
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(1);

fn cli(command: &str, source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-dictionary-defaults-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let args = if command == "run" {
        vec!["run".into(), "--allow".into(), "console_write".into(),
            path.to_string_lossy().into_owned()]
    } else {
        vec![command.to_string(), path.to_string_lossy().into_owned()]
    };
    let result = run_cli(&args);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn default_lookup_handles_missing_entries_without_error() {
    let source = r#"let scores be dictionary of ("ada": 7, "grace": 9).
assert (entry "absent" of scores with default 0) is equal to 0.
assert (entry "ada" of scores with default 0) is equal to 7.
print entry "unknown" of scores with default 5.
"#;
    for cmd in ["check", "ast", "hir", "effects"] {
        let result = cli(cmd, source);
        assert_eq!(result.status, 0, "{cmd}: {}", result.stderr);
    }
    let result = cli("run", source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "5\n");
    let ast = cli("ast", source);
    assert_eq!(ast.status, 0, "{}", ast.stderr);
    assert!(ast.stdout.contains("dictionary_get_or"), "{}", ast.stdout);
    let hir = Compiler::new().hir(source).unwrap().to_json().to_string();
    assert!(hir.contains("dictionary_get_or"), "{hir}");
}

#[test]
fn default_branch_is_only_evaluated_if_key_is_absent() {
    let source = r#"let values be dictionary of ("exists": 8).
print entry "exists" of values with default (1 divided by 0).
"#;
    let result = cli("run", source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "8\n");

    let missing = r#"let values be dictionary of ("exists": 8).
print entry "missing" of values with default (1 divided by 0).
"#;
    let result = cli("run", missing);
    assert_eq!(result.status, 65, "{}", result.stderr);
}

#[test]
fn fallback_must_match_dictionary_element_type() {
    for source in [
        r#"let values be dictionary of ("n": 8).
print entry "missing" of values with default "wrong".
"#,
        r#"let values be dictionary of ("n": 8).
print entry "n" of values with default true.
"#,
        r#"print entry "missing" of 42 with default 0.
"#,
    ] {
        let result = cli("check", source);
        assert_eq!(result.status, 65, "{source}: {}", result.stderr);
        assert!(result.stderr.contains("S423"), "{source}: {}", result.stderr);
    }
}

#[test]
fn record_and_list_valued_dictionaries_can_provide_typed_defaults() {
    let source = r#"let people be dictionary of (
    "ada": record of ("name": "Ada", "age": 37),
    "grace": record of ("name": "Grace", "age": 34)
).
let unknown be entry "absent" of people with default record of ("name": "Unknown", "age": 0).
print field "name" of unknown.
"#;
    let result = cli("run", source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Unknown\n");
}

#[test]
fn defaulted_lookups_work_in_pure_functions() {
    let source = r#"define function score taking data as dictionary of number to entry "missing" of data with default 99.
let scores be dictionary of ("primary": 25).
print invoke score to scores.
"#;
    let result = cli("run", source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "99\n");
}

#[test]
fn formatter_understands_contextual_with_default_clause() {
    let source = "LET values BE DICTIONARY OF (\"X\": 4).\n\
                  PRINT ENTRY \"Y\" OF values WITH DEFAULT 0.\n";
    let formatted = cli("fmt", source);
    assert_eq!(formatted.status, 0, "{}", formatted.stderr);
    assert!(formatted.stdout.contains("entry \"Y\" of values with default 0"),
        "{}", formatted.stdout);
}

#[test]
fn defaults_are_contextual_words_not_reserved_identifiers() {
    let source = "let default be 3.\nlet with be 2.\nprint default plus with.\n";
    let result = cli("run", source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "5\n");
}

#[test]
fn plain_lookup_still_reports_missing_key() {
    let source = r#"let data be dictionary of ("known": 1).
print entry "missing" of data.
"#;
    let result = cli("run", source);
    assert_eq!(result.status, 65, "{}", result.stderr);
    assert!(result.stderr.contains("S421"), "{}", result.stderr);
}
