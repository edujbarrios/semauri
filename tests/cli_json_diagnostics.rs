// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::run_cli;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

fn fixture(source: &str) -> PathBuf {
    let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    let file = std::env::temp_dir().join(format!(
        "semauri-json-cli-{}-{id}.suri",
        std::process::id()
    ));
    fs::write(&file, source).unwrap();
    file
}

fn json_result(args: Vec<String>) -> (i32, Value, String) {
    let result = run_cli(&args);
    let json = serde_json::from_str(&result.stdout)
        .expect("JSON mode must emit parseable JSON on both success and failure");
    (result.status, json, result.stderr)
}

#[test]
fn check_json_returns_a_stable_success_envelope() {
    let path = fixture("Create a web called Example.");
    let (status, body, stderr) = json_result(vec![
        "check".into(), "--json".into(), path.to_string_lossy().to_string()
    ]);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(body["ok"], true);
    assert_eq!(body["status"], 0);
    assert_eq!(body["diagnostics"].as_array().unwrap().len(), 0);
    assert!(stderr.is_empty());
    fs::remove_file(path).unwrap();
}

#[test]
fn check_json_emits_source_spans_for_lexical_errors() {
    let path = fixture("Let text be \"unterminated");
    let (status, body, stderr) = json_result(vec![
        "check".into(), path.to_string_lossy().to_string(), "--json".into()
    ]);
    assert_eq!(status, 65, "{stderr}");
    assert_eq!(body["ok"], false);
    assert_eq!(body["diagnostics"][0]["kind"], "lex");
    assert!(body["diagnostics"][0]["code"].as_str().unwrap().starts_with('S'));
    assert!(body["diagnostics"][0]["span"]["start"]["line"].as_u64().is_some());
    assert!(stderr.is_empty());
    fs::remove_file(path).unwrap();
}

#[test]
fn capability_errors_are_machine_readable_and_keep_exit_status() {
    let path = fixture("Write \"hello\" to \"notes.txt\".");
    let (status, body, stderr) = json_result(vec![
        "check".into(), "--json".into(), "--allow-none".into(),
        path.to_string_lossy().to_string()
    ]);
    assert_eq!(status, 65);
    assert_eq!(body["status"], 65);
    assert_eq!(body["diagnostics"][0]["code"], "S334");
    assert!(stderr.is_empty());
    fs::remove_file(path).unwrap();
}

#[test]
fn existing_json_inspection_commands_gain_json_errors_without_wrapping_success() {
    let path = fixture("Create a web.");
    let (_, body, _) = json_result(vec![
        "effects".into(), path.to_string_lossy().to_string(), "--json".into()
    ]);
    assert!(body.is_object());
    assert!(body.get("diagnostics").is_none());
    fs::remove_file(path).unwrap();

    let bad = fixture("Let text be \"unterminated");
    let (status, body, _) = json_result(vec![
        "plan".into(), "--json".into(), bad.to_string_lossy().to_string()
    ]);
    assert_eq!(status, 65);
    assert_eq!(body["diagnostics"][0]["kind"], "lex");
    fs::remove_file(bad).unwrap();
}

#[test]
fn default_text_diagnostics_remain_unchanged() {
    let path = fixture("Write \"hello\" to \"notes.txt\".");
    let result = run_cli(&[
        "check".into(), "--allow-none".into(), path.to_string_lossy().to_string()
    ]);
    assert_eq!(result.status, 65);
    assert!(result.stdout.is_empty());
    assert!(result.stderr.contains("S334"));
    fs::remove_file(path).unwrap();
}
