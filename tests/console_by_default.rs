// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::run_cli;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn fixture(source: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "semauri-console-default-{}-{}.suri",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    path
}

fn invoke(arguments: &[&str]) -> semauri::CliResult {
    run_cli(&arguments.iter().map(|value| value.to_string()).collect::<Vec<_>>())
}

#[test]
fn prints_to_terminal_without_an_explicit_permission() {
    let path = fixture("print \"Hi from Semauri\".");
    let path = path.to_str().unwrap().to_string();
    for args in [
        vec!["run", path.as_str()],
        vec![path.as_str()],
        vec![path.as_str(), "--allow", "console_write"],
    ] {
        let output = invoke(&args);
        assert_eq!(output.status, 0, "{args:?}: {}", output.stderr);
        assert_eq!(output.stdout, "Hi from Semauri\n");
    }
    fs::remove_file(path).unwrap();
}

#[test]
fn deny_console_is_an_explicit_override_even_with_legacy_allow() {
    let path = fixture("assert true. print 42.");
    let path = path.to_str().unwrap().to_string();
    for args in [
        vec!["run", path.as_str(), "--deny-console"],
        vec![path.as_str(), "--deny-console", "--allow", "console_write"],
        vec!["run", "--allow", "console_write", "--deny-console", path.as_str()],
    ] {
        let output = invoke(&args);
        assert_eq!(output.status, 65, "{args:?}: {}", output.stderr);
        assert!(output.stderr.contains("S334"), "{}", output.stderr);
        assert!(output.stderr.contains("console_write"), "{}", output.stderr);
    }
    fs::remove_file(path).unwrap();
}

#[test]
fn extra_effects_remain_denied_even_when_console_is_allowed() {
    let source = fixture("write \"secret\" to \"no-autowrite.txt\".");
    let path = source.to_str().unwrap().to_string();
    for args in [vec!["run", path.as_str()], vec![path.as_str()]] {
        let result = invoke(&args);
        assert_eq!(result.status, 65, "{}", result.stderr);
        assert!(result.stderr.contains("S334"));
        assert!(result.stderr.contains("filesystem_write"));
    }
    fs::remove_file(source).unwrap();
}

#[test]
fn secure_random_still_requires_a_grant_and_check_policy_is_strict() {
    let path = fixture("let secret be password of 20 characters. print \"OK\".");
    let path_str = path.to_str().unwrap().to_string();
    let denied = invoke(&["run", path_str.as_str()]);
    assert_eq!(denied.status, 65);
    assert!(denied.stderr.contains("secure_random"));

    let allowed = invoke(&[path_str.as_str(), "--allow", "secure_random"]);
    assert_eq!(allowed.status, 0, "{}", allowed.stderr);
    assert_eq!(allowed.stdout, "OK\n");

    let static_check = invoke(&["check", "--allow-none", path_str.as_str()]);
    assert_eq!(static_check.status, 65);
    assert!(static_check.stderr.contains("S334"));
    fs::remove_file(path).unwrap();
}
