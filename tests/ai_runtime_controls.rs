// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::run_cli;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-ai-controls-{}-{}.suri",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&[
        "run".into(),
        path.to_string_lossy().into_owned(),
        "--allow".into(),
        "model_load".into(),
        "--allow".into(),
        "network".into(),
        "--allow".into(),
        "model_inference".into(),
        "--allow".into(),
        "console_write".into(),
    ]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn request_budget_stops_additional_inference() {
    let result = run(
        r#"cap ai requests to 1.
let model be connect language model "local/test".
let first be prompt model with "one".
let second be prompt model with "two".
print second."#,
    );
    assert_eq!(result.status, 70);
    assert!(result.stderr.contains("AI request budget exhausted"));
}

#[test]
fn response_cache_does_not_spend_budget_twice() {
    let result = run(
        r#"cap ai requests to 1.
memoize ai response cache.
let model be connect language model "local/test".
let first be prompt model with "same".
let second be prompt model with "same".
print first.
print second."#,
    );
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[local/test] same\n[local/test] same\n");
}

#[test]
fn control_counts_must_be_nonnegative_integers() {
    for value in ["minus 1", "1.5"] {
        let result = run(&format!(
            "cap ai requests to {value}. let model be connect language model \"local/test\". let answer be prompt model with \"hello\". print answer."
        ));
        assert_eq!(result.status, 70);
        assert!(result.stderr.contains("non-negative integer"));
    }
}
