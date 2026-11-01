// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::run_cli;
use serde_json::Value;
use std::{fs, path::PathBuf, sync::atomic::{AtomicU64, Ordering}};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn walkthrough() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/explain_walkthrough.suri")
        .to_string_lossy()
        .into_owned()
}

#[test]
fn explain_json_matches_numbered_text_trace_without_executing() {
    let path = walkthrough();
    let plain = run_cli(&["explain".into(), path.clone()]);
    let json = run_cli(&["explain".into(), "--json".into(), path.clone()]);
    let reordered = run_cli(&["explain".into(), path, "--json".into()]);
    assert_eq!(plain.status, 0, "{}", plain.stderr);
    assert_eq!(json.status, 0, "{}", json.stderr);
    assert_eq!(json.stdout, reordered.stdout);
    assert!(json.stderr.is_empty());
    let data: Value = serde_json::from_str(&json.stdout).unwrap();
    assert_eq!(data["ok"], true);
    assert_eq!(data["status"], 0);
    let lines = data["explanations"].as_array().unwrap();
    assert_eq!(lines.len(), 4);
    let rendered = lines.iter().enumerate().map(|(i, line)| {
        format!("{}. {}\n", i + 1, line.as_str().unwrap())
    }).collect::<String>();
    assert_eq!(rendered, plain.stdout);
}

#[test]
fn explain_json_provides_diagnostics_for_invalid_source() {
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "semauri-explain-json-{}-{id}.suri", std::process::id()));
    fs::write(&path, "let n be missing.").unwrap();
    let result = run_cli(&[
        "explain".into(), "--json".into(), path.to_string_lossy().into_owned()
    ]);
    assert_eq!(result.status, 65, "{} {}", result.stdout, result.stderr);
    assert!(result.stderr.is_empty());
    let data: Value = serde_json::from_str(&result.stdout).unwrap();
    assert_eq!(data["ok"], false);
    assert_eq!(data["status"], 65);
    assert!(data["diagnostics"][0]["code"].as_str().unwrap().starts_with('S'));
    assert!(data["diagnostics"][0]["span"]["start"]["line"].is_number());
    fs::remove_file(path).unwrap();
}

#[test]
fn explain_without_json_retains_original_human_readable_format() {
    let result = run_cli(&["explain".into(), walkthrough()]);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert!(result.stdout.starts_with("1. Entered semantic domain"));
    assert!(!result.stdout.starts_with('{'));
}
