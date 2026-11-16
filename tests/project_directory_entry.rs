// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::run_cli;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn project() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "semauri-directory-entry-{}-{}", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let result = run_cli(&["init".into(), root.to_string_lossy().into_owned()]);
    assert_eq!(result.status, 0, "{}", result.stderr);
    root
}

fn call(command: &str, path: &Path) -> semauri::CliResult {
    run_cli(&[command.into(), path.to_string_lossy().into_owned()])
}

#[test]
fn run_and_check_accept_a_project_directory() {
    let root = project();
    let checked = call("check", &root);
    assert_eq!(checked.status, 0, "{}", checked.stderr);
    assert_eq!(checked.stdout, "OK\n");

    let ran = call("run", &root);
    assert_eq!(ran.status, 0, "{}", ran.stderr);
    assert_eq!(ran.stdout, "49\n");

    let shorthand = run_cli(&[root.join("main.suri").to_string_lossy().into_owned()]);
    assert_eq!(shorthand.status, 0, "{}", shorthand.stderr);
    assert_eq!(shorthand.stdout, ran.stdout);

    // Project path resolution is also used by inspection commands.
    let effects = call("effects", &root);
    assert_eq!(effects.status, 0, "{}", effects.stderr);
    assert!(effects.stdout.contains("console_write"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn no_main_file_fails_with_a_helpful_error_not_an_arbitrary_entry() {
    let root = project();
    fs::remove_file(root.join("main.suri")).unwrap();
    fs::write(root.join("other.suri"), "print \"wrong\".").unwrap();
    for name in ["run", "check"] {
        let result = call(name, &root);
        assert_eq!(result.status, 64, "{}", result.stderr);
        assert!(result.stderr.contains("main.suri"), "{}", result.stderr);
        assert!(!result.stdout.contains("wrong"));
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn directory_execution_cannot_bypass_permissions() {
    let root = project();
    fs::write(root.join("main.suri"), "write \"data\" to \"new-file.txt\".").unwrap();
    let result = call("run", &root);
    assert_eq!(result.status, 65, "{}", result.stderr);
    assert!(result.stderr.contains("S334"), "{}", result.stderr);
    assert!(result.stderr.contains("filesystem_write"), "{}", result.stderr);
    assert!(!root.join("new-file.txt").exists());

    fs::write(root.join("main.suri"), "print \"message\".").unwrap();
    let strict = run_cli(&["run".into(), "--deny-console".into(),
        root.to_string_lossy().into_owned()]);
    assert_eq!(strict.status, 65);
    assert!(strict.stderr.contains("console_write"));
    fs::remove_dir_all(root).unwrap();
}
