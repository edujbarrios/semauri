// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::run_cli;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

fn fixture() -> PathBuf {
    let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "semauri-file-only-delete-{}-{id}",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
    root
}

fn run(root: &std::path::Path, source: &str) -> semauri::CliResult {
    let file = root.join("program.suri");
    fs::write(&file, source).unwrap();
    run_cli(&[
        "run".into(),
        "--allow".into(),
        "filesystem_write".into(),
        "--cwd".into(),
        root.to_string_lossy().to_string(),
        file.to_string_lossy().to_string(),
    ])
}

#[test]
fn delete_rejects_directories_without_recursive_removal() {
    let root = fixture();
    let directory = root.join("data");
    fs::create_dir(&directory).unwrap();
    fs::write(directory.join("keep.txt"), "important").unwrap();

    let result = run(&root, "Delete \"data\".");
    assert_eq!(result.status, 70, "{}", result.stderr);
    assert!(result.stderr.contains("S500"), "{}", result.stderr);
    assert!(result.stderr.contains("Refusing to recursively delete"), "{}", result.stderr);
    assert_eq!(fs::read_to_string(directory.join("keep.txt")).unwrap(), "important");

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn delete_removes_regular_files_and_ignores_missing_files() {
    let root = fixture();
    fs::write(root.join("temporary.txt"), "discard").unwrap();

    let result = run(&root, "Delete \"temporary.txt\". Delete \"missing.txt\".");
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert!(!root.join("temporary.txt").exists());

    fs::remove_dir_all(root).unwrap();
}
