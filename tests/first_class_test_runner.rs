// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::run_cli;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn temp_dir() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "semauri-suite-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
    ));
    fs::create_dir_all(&root).unwrap();
    root
}

fn test_run(dir: &PathBuf, allow_console: bool) -> semauri::CliResult {
    let mut args = vec!["test".into(), dir.to_string_lossy().to_string()];
    if allow_console {
        args.push("--allow-console".into());
    }
    run_cli(&args)
}

#[test]
fn runs_pure_assertion_suites_in_deterministic_filename_order() {
    let dir = temp_dir();
    fs::write(dir.join("z_last.suri"), "Assert 2 is greater than 1.").unwrap();
    fs::write(dir.join("a_first.suri"), "Let answer be 6 times 7. Assert answer is equal to 42.").unwrap();

    let result = test_run(&dir, false);
    assert_eq!(result.status, 0, "{}", result.stdout);
    assert!(result.stdout.contains("2 passed; 0 failed"));
    assert!(result.stdout.find("a_first.suri").unwrap() < result.stdout.find("z_last.suri").unwrap());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn failed_assertions_fail_the_suite_and_report_source_location() {
    let dir = temp_dir();
    fs::write(dir.join("pass.suri"), "Assert true.").unwrap();
    fs::write(dir.join("fail.suri"), "Assert false.").unwrap();

    let result = test_run(&dir, false);
    assert_eq!(result.status, 1);
    assert!(result.stdout.contains("FAIL"));
    assert!(result.stdout.contains("S502"), "{}", result.stdout);
    assert!(result.stdout.contains("1 passed; 1 failed"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn requires_assertions_and_rejects_effectful_filesystem_programs() {
    let dir = temp_dir();
    fs::write(dir.join("no_assert.suri"), "Print \"hello\".").unwrap();
    fs::write(dir.join("unsafe.suri"), "Assert true. Write \"data\" to \"unexpected.txt\".").unwrap();

    let result = test_run(&dir, true);
    assert_eq!(result.status, 1);
    assert!(result.stdout.contains("needs at least one executable Assert"));
    assert!(result.stdout.contains("Tests must not generate build artifacts"));
    assert!(!dir.join("unexpected.txt").exists());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn printing_requires_an_explicit_optional_console_grant() {
    let dir = temp_dir();
    fs::write(dir.join("visible.suri"), "Assert true. Print \"ok\".").unwrap();

    let denied = test_run(&dir, false);
    assert_eq!(denied.status, 1);
    assert!(denied.stdout.contains("S334"));

    let allowed = test_run(&dir, true);
    assert_eq!(allowed.status, 0, "{}", allowed.stdout);
    assert!(allowed.stdout.contains("1 passed; 0 failed"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn directory_discovery_is_recursive_and_empty_suites_are_usage_errors() {
    let dir = temp_dir();
    fs::create_dir(dir.join("nested")).unwrap();
    fs::write(dir.join("nested").join("case.suri"), "Assert true.").unwrap();

    let result = test_run(&dir, false);
    assert_eq!(result.status, 0, "{}", result.stdout);

    fs::remove_file(dir.join("nested").join("case.suri")).unwrap();
    let empty = test_run(&dir, false);
    assert_eq!(empty.status, 64);
    assert!(empty.stderr.contains("No .suri test files"));
    fs::remove_dir_all(&dir).unwrap();
}
