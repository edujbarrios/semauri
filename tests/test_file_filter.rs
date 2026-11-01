// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::run_cli;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn suite() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "semauri-filter-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(root.join("module")).unwrap();
    fs::create_dir_all(root.join("other")).unwrap();
    fs::write(root.join("module").join("success.suri"), "assert true.").unwrap();
    fs::write(root.join("other").join("failing.suri"), "assert false.").unwrap();
    root
}

fn command(root: &std::path::Path, flags: &[&str]) -> semauri::CliResult {
    let mut args = vec!["test".to_string(), root.to_string_lossy().into_owned()];
    args.extend(flags.iter().map(|flag| flag.to_string()));
    run_cli(&args)
}

#[test]
fn selects_only_matching_relative_paths() {
    let root = suite();
    let result = command(&root, &["--filter", "module/success"]);
    assert_eq!(result.status, 0, "{} {}", result.stdout, result.stderr);
    assert!(result.stdout.contains("1 passed; 0 failed"));
    assert!(!result.stdout.contains("failing.suri"));
    let all = command(&root, &[]);
    assert_eq!(all.status, 1);
    assert!(all.stdout.contains("1 passed; 1 failed"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn json_reports_only_selected_files_and_preserves_failures() {
    let root = suite();
    let result = command(&root, &["--filter", "failing", "--json"]);
    assert_eq!(result.status, 1);
    let report: serde_json::Value = serde_json::from_str(&result.stdout).unwrap();
    assert_eq!(report["failed"], 1);
    assert_eq!(report["passed"], 0);
    assert_eq!(report["tests"].as_array().unwrap().len(), 1);
    assert!(report["tests"][0]["path"].as_str().unwrap().ends_with("failing.suri"));
    assert_eq!(report["tests"][0]["ok"], false);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn nonexistent_filter_and_invalid_options_fail_without_running_tests() {
    let root = suite();
    for flags in [
        vec!["--filter", "not-present", "--json"],
        vec!["--filter", "--json"],
        vec!["--filter", ""],
        vec!["--filter", "success", "--filter", "failing"],
    ] {
        let result = command(&root, &flags);
        assert_eq!(result.status, 64, "flags: {flags:?}: {}", result.stdout);
    }
    let missing = command(&root, &["--filter", "not-present", "--json"]);
    let report: serde_json::Value = serde_json::from_str(&missing.stdout).unwrap();
    assert_eq!(report["ok"], false);
    assert_eq!(report["status"], 64);
    assert!(report["error"].as_str().unwrap().contains("No test files matched"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn file_inputs_match_only_the_basename() {
    let root = suite();
    let file = root.join("module").join("success.suri");
    let args = ["test".to_string(), file.to_string_lossy().into_owned(),
        "--filter".to_string(), "success".to_string()];
    let result = run_cli(&args);
    assert_eq!(result.status, 0, "{} {}", result.stdout, result.stderr);
    fs::remove_dir_all(root).unwrap();
}
