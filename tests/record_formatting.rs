// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn fixture(source: &str) -> PathBuf {
    let file = std::env::temp_dir().join(format!(
        "semauri-record-format-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&file, source).unwrap();
    file
}

#[test]
fn formatter_normalizes_contextual_record_words_without_touching_strings() {
    let source = "LET person BE RECORD OF (\"Name\": \"RECORD OF\", \"Age\": 37).\n\
                  PRINT FIELD \"Name\" OF person.\n";
    let file = fixture(source);
    let formatted = run_cli(&["fmt".into(), file.to_string_lossy().into_owned()]);
    assert_eq!(formatted.status, 0, "{}", formatted.stderr);
    assert!(formatted.stdout.contains("record of (\"Name\": \"RECORD OF\""),
        "{}", formatted.stdout);
    assert!(formatted.stdout.contains("field \"Name\" of person"),
        "{}", formatted.stdout);
    let checked = run_cli(&["check".into(), file.to_string_lossy().into_owned()]);
    assert_eq!(checked.status, 0, "{}", checked.stderr);
    fs::remove_file(file).unwrap();
}

#[test]
fn contextual_record_words_remain_ordinary_identifiers_outside_records() {
    let source = "let record be 2.\nlet field be 3.\nprint record plus field.\n";
    let result = Compiler::new().analyze(source);
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn checked_in_example_works_as_a_real_program() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/typed_records.suri");
    let run = run_cli(&["run".into(),"--allow".into(),"console_write".into(),
        path.to_string_lossy().into_owned()]);
    assert_eq!(run.status,0,"{}",run.stderr);
    assert_eq!(run.stdout,"Ada\n");
}
