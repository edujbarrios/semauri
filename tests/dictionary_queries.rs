// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
//! Safe dictionary membership checks and sorted key enumeration.
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn command(cmd: &str, source: &str) -> semauri::CliResult {
    let file = std::env::temp_dir().join(format!("semauri-dict-query-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&file, source).unwrap();
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
fn membership_is_safe_for_present_and_absent_keys() {
    let source = r#"let scores be dictionary of ("Ada": 8, "Bob": 10).
assert has entry "Ada" in scores.
assert not (has entry "Carol" in scores).
print has entry "Bob" in scores.
"#;
    for cmd in ["check", "hir", "ast", "effects"] {
        let result = command(cmd, source);
        assert_eq!(result.status, 0, "{cmd}: {}", result.stderr);
    }
    let run = command("run", source);
    assert_eq!(run.status, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "true\n");
}

#[test]
fn keys_of_dictionary_are_sorted_and_serializable() {
    let source = r#"let scores be dictionary of ("zeta": 2, "alpha": 1).
print json of keys of scores.
"#;
    let run = command("run", source);
    assert_eq!(run.status, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "[\"alpha\",\"zeta\"]\n");
    let hir = Compiler::new().hir(source).unwrap().to_json().to_string();
    assert!(hir.contains("dictionary_keys"), "{hir}");
}

#[test]
fn queries_work_inside_pure_functions_and_nested_dictionaries() {
    let source = r#"define function has_primary taking scores as dictionary of number to has entry "primary" in scores.
define function get_keys taking scores as dictionary of number to keys of scores.
let scores be dictionary of ("primary": 5, "other": 8).
let entries be dictionary of ("scores": scores, "backup": scores).
assert invoke has_primary to entry "scores" of entries.
assert not (has entry "missing" in entry "backup" of entries).
print json of (invoke get_keys to scores).
"#;
    let result = command("run", source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[\"other\",\"primary\"]\n");
}

#[test]
fn non_dictionary_lookups_are_rejected_during_type_checking() {
    for source in [
        "print keys of 5.\n",
        "print has entry \"x\" in 5.\n",
        "let person be record of (\"x\": 1).\nprint keys of person.\n",
        "let person be record of (\"x\": 1).\nprint has entry \"x\" in person.\n",
    ] {
        let checked = command("check", source);
        assert_eq!(checked.status, 65, "{source}: {}", checked.stderr);
        assert!(checked.stderr.contains("S422"), "{source}: {}", checked.stderr);
    }
}

#[test]
fn existing_missing_entry_errors_remain_explicit() {
    let source = r#"let d be dictionary of ("x": 1).
assert not (has entry "y" in d).
print entry "y" of d.
"#;
    let run = command("run", source);
    assert_eq!(run.status, 65, "{}", run.stderr);
    assert!(run.stderr.contains("S421"), "{}", run.stderr);
}

#[test]
fn formatter_recognizes_new_dictionary_phrases_without_changing_keys() {
    let source = "LET data BE DICTIONARY OF (\"KEYS OF\": 2).\n\
                  PRINT HAS ENTRY \"KEYS OF\" IN data.\n\
                  PRINT KEYS OF data.\n";
    let fmt = command("fmt", source);
    assert_eq!(fmt.status, 0, "{}", fmt.stderr);
    assert!(fmt.stdout.contains("has entry \"KEYS OF\" in data"), "{}", fmt.stdout);
    assert!(fmt.stdout.contains("keys of data"), "{}", fmt.stdout);
    assert!(fmt.stdout.contains("dictionary of (\"KEYS OF\": 2)"), "{}", fmt.stdout);
}

#[test]
fn context_words_remain_valid_user_identifiers() {
    let source = "let has be 2.\nlet keys be 3.\nprint has plus keys.\n";
    let checked = command("check", source);
    assert_eq!(checked.status, 0, "{}", checked.stderr);
    let run = command("run", source);
    assert_eq!(run.status, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "5\n");
}

#[test]
fn existing_keys_of_json_keeps_its_meaning() {
    let source = r#"let doc be "{\"b\":2,\"a\":1}".
print json of keys of json doc.
"#;
    let checked = command("check", source);
    assert_eq!(checked.status, 0, "{}", checked.stderr);
    let run = command("run", source);
    assert_eq!(run.status, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "[\"a\",\"b\"]\n");
}
