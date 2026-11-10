// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::run_cli;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn fixture(source: &str) -> PathBuf {
    let file = std::env::temp_dir().join(format!(
        "semauri-formatter-json-{}-{}.suri",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&file, source).unwrap();
    file
}

fn json_fmt(args: Vec<String>) -> (i32, Value, String) {
    let result = run_cli(&args);
    let body = serde_json::from_str(&result.stdout)
        .expect("fmt --json should return parseable JSON");
    (result.status, body, result.stderr)
}

#[test]
fn file_preview_check_and_write_have_consistent_reports() {
    let file = fixture("LET Answer BE 42.\nPRINT Answer.\n");
    let path = file.to_string_lossy().to_string();

    let (status, preview, stderr) = json_fmt(vec![
        "fmt".into(), path.clone(), "--json".into()
    ]);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(preview["ok"], true);
    assert_eq!(preview["mode"], "preview");
    assert_eq!(preview["files_checked"], 1);
    assert_eq!(preview["files_changed"], 1);
    assert_eq!(preview["formatted"], "let Answer be 42.\nprint Answer.\n");
    assert_eq!(preview["changed_files"][0].as_str(), Some(path.as_str()));
    assert!(fs::read_to_string(&file).unwrap().starts_with("LET"));

    let (status, check, stderr) = json_fmt(vec![
        "fmt".into(), "--json".into(), "--check".into(), path.clone()
    ]);
    assert_eq!(status, 1, "{stderr}");
    assert_eq!(check["ok"], false);
    assert_eq!(check["status"], 1);
    assert_eq!(check["mode"], "check");
    assert!(check.get("formatted").is_none());

    let (status, write, stderr) = json_fmt(vec![
        "fmt".into(), path.clone(), "--write".into(), "--json".into()
    ]);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(write["mode"], "write");
    assert_eq!(write["files_changed"], 1);
    assert_eq!(fs::read_to_string(&file).unwrap(), "let Answer be 42.\nprint Answer.\n");

    let (status, clean, stderr) = json_fmt(vec![
        "fmt".into(), "--check".into(), path, "--json".into()
    ]);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(clean["ok"], true);
    assert_eq!(clean["files_changed"], 0);
    assert!(clean["changed_files"].as_array().unwrap().is_empty());
    fs::remove_file(file).unwrap();
}

#[test]
fn project_reports_sorted_changes_and_respects_preflight() {
    let parent = std::env::temp_dir().join(format!(
        "semauri-format-project-json-{}-{}",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&parent).unwrap();
    let root = parent.join("app");
    let init = run_cli(&["init".into(), root.to_string_lossy().into_owned()]);
    assert_eq!(init.status, 0, "{}", init.stderr);
    let main = root.join("main.suri");
    let library = root.join("lib/math.suri");
    let original = fs::read_to_string(&main).unwrap();
    fs::write(&main, original.replace("import", "IMPORT")).unwrap();
    let code = fs::read_to_string(&library).unwrap();
    fs::write(&library, code.replace("define", "DEFINE")).unwrap();
    let path = root.to_string_lossy().into_owned();

    let (status, check, stderr) = json_fmt(vec![
        "fmt".into(), "--json".into(), path.clone(), "--check".into()
    ]);
    assert_eq!(status, 1, "{stderr}");
    assert_eq!(check["files_checked"], 3);
    assert_eq!(check["files_changed"], 2);
    let files = check["changed_files"].as_array().unwrap();
    assert!(files[0].as_str().unwrap().ends_with("lib/math.suri"));
    assert!(files[1].as_str().unwrap().ends_with("main.suri"));
    assert!(fs::read_to_string(&main).unwrap().contains("IMPORT"));

    let (status, write, stderr) = json_fmt(vec![
        "fmt".into(), "--write".into(), "--json".into(), path.clone()
    ]);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(write["files_changed"], 2);
    let (status, clean, _) = json_fmt(vec![
        "fmt".into(), path, "--check".into(), "--json".into()
    ]);
    assert_eq!(status, 0);
    assert_eq!(clean["files_changed"], 0);
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn invalid_source_uses_json_diagnostics_and_preserves_bytes() {
    let file = fixture("print \"unterminated");
    let path = file.to_string_lossy().into_owned();
    let (status, body, stderr) = json_fmt(vec![
        "fmt".into(), "--json".into(), "--write".into(), path
    ]);
    assert_eq!(status, 65, "{stderr}");
    assert_eq!(body["ok"], false);
    assert_eq!(body["diagnostics"][0]["code"], "S103");
    assert!(stderr.is_empty());
    assert_eq!(fs::read_to_string(&file).unwrap(), "print \"unterminated");
    fs::remove_file(file).unwrap();
}
