// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::run_cli;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn suite() -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "semauri-json-tests-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&directory).unwrap();
    directory
}

fn json_test(directory: &PathBuf, additional: &[&str]) -> (i32, Value, String) {
    let mut args = vec!["test".to_string(), "--json".to_string(),
        directory.to_string_lossy().into_owned()];
    args.extend(additional.iter().map(|s| s.to_string()));
    let output = run_cli(&args);
    let report: Value = serde_json::from_str(&output.stdout)
        .expect("JSON test command must return JSON");
    (output.status, report, output.stderr)
}

#[test]
fn successful_suites_have_structured_results_in_filename_order() {
    let directory = suite();
    fs::write(directory.join("z_second.suri"), "assert true.").unwrap();
    fs::write(directory.join("a_first.suri"), "assert 1 is equal to 1.").unwrap();
    let (status, report, stderr) = json_test(&directory, &[]);
    assert_eq!(status, 0, "{stderr}");
    assert_eq!(report["ok"], true);
    assert_eq!(report["status"], 0);
    assert_eq!(report["passed"], 2);
    assert_eq!(report["failed"], 0);
    let tests = report["tests"].as_array().unwrap();
    assert_eq!(tests.len(), 2);
    assert!(tests[0]["path"].as_str().unwrap().ends_with("a_first.suri"));
    assert!(tests[1]["path"].as_str().unwrap().ends_with("z_second.suri"));
    assert_eq!(tests[0]["ok"], true);
    assert!(tests[0].get("error").is_none());
    assert!(stderr.is_empty());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn failed_assertions_remain_failures_in_json_mode() {
    let directory = suite();
    fs::write(directory.join("fail.suri"), "assert false.").unwrap();
    let (status, report, stderr) = json_test(&directory, &[]);
    assert_eq!(status, 1);
    assert_eq!(report["ok"], false);
    assert_eq!(report["status"], 1);
    assert_eq!(report["passed"], 0);
    assert_eq!(report["failed"], 1);
    assert_eq!(report["tests"][0]["ok"], false);
    assert!(report["tests"][0]["error"].as_str().unwrap().contains("S502"));
    assert!(stderr.is_empty());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn json_mode_preserves_explicit_console_permissions() {
    let directory = suite();
    fs::write(directory.join("prints.suri"), "assert true. print \"ok\".").unwrap();
    let (denied_status, denied, _) = json_test(&directory, &[]);
    assert_eq!(denied_status, 1);
    assert!(denied["tests"][0]["error"].as_str().unwrap().contains("S334"));
    let (allowed_status, allowed, _) = json_test(&directory, &["--allow-console"]);
    assert_eq!(allowed_status, 0);
    assert_eq!(allowed["passed"], 1);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn json_usage_errors_have_valid_machine_readable_envelopes() {
    let directory = suite();
    let (status, body, stderr) = json_test(&directory, &[]);
    assert_eq!(status, 64);
    assert_eq!(body["ok"], false);
    assert_eq!(body["status"], 64);
    assert_eq!(body["tests"].as_array().unwrap().len(), 0);
    assert!(body["error"].as_str().unwrap().contains("No .suri test files"));
    assert!(stderr.is_empty());
    fs::remove_dir_all(directory).unwrap();
}
