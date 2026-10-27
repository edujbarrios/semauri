// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::run_cli;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn suite() -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "semauri-test-discovery-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&directory).unwrap();
    directory
}

fn run(directory: &PathBuf) -> semauri::CliResult {
    run_cli(&["test".into(), directory.to_string_lossy().into_owned()])
}

#[test]
fn ignores_generated_directories_and_configuration_files() {
    let directory = suite();
    fs::create_dir(directory.join("target")).unwrap();
    fs::create_dir(directory.join(".git")).unwrap();
    fs::write(directory.join("pass.suri"), "assert true.").unwrap();
    fs::write(directory.join("target").join("fail.suri"), "assert false.").unwrap();
    fs::write(directory.join(".git").join("fail.suri"), "assert false.").unwrap();
    fs::write(directory.join(".config.suri"), "").unwrap();

    let result = run(&directory);
    assert_eq!(result.status, 0, "{} {}", result.stdout, result.stderr);
    assert!(result.stdout.contains("1 passed; 0 failed"), "{}", result.stdout);
    assert!(!result.stdout.contains("fail.suri"));
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn refuses_unbounded_suites_before_execution() {
    let directory = suite();
    for number in 0..257 {
        fs::write(directory.join(format!("test_{number:03}.suri")), "assert true.").unwrap();
    }
    let result = run(&directory);
    assert_eq!(result.status, 64);
    assert!(result.stderr.contains("256 .suri files"), "{}", result.stderr);
    assert!(result.stdout.is_empty());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn rejects_excessively_nested_suite_directories() {
    let directory = suite();
    let mut current = directory.clone();
    for _ in 0..33 {
        current.push("d");
        fs::create_dir(&current).unwrap();
    }
    fs::write(current.join("case.suri"), "assert true.").unwrap();
    let result = run(&directory);
    assert_eq!(result.status, 64);
    assert!(result.stderr.contains("32 levels"), "{}", result.stderr);
    fs::remove_dir_all(directory).unwrap();
}

#[cfg(unix)]
#[test]
fn never_follows_symlinked_test_directories() {
    use std::os::unix::fs::symlink;
    let directory = suite();
    let outside = suite();
    fs::write(outside.join("bad.suri"), "assert false.").unwrap();
    fs::write(directory.join("good.suri"), "assert true.").unwrap();
    symlink(&outside, directory.join("external")).unwrap();
    let result = run(&directory);
    assert_eq!(result.status, 0, "{} {}", result.stdout, result.stderr);
    assert!(result.stdout.contains("1 passed; 0 failed"));
    fs::remove_dir_all(directory).unwrap();
    fs::remove_dir_all(outside).unwrap();
}
