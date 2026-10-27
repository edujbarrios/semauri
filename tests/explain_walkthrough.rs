// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::run_cli;
use std::path::PathBuf;

fn example_path() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/explain_walkthrough.suri")
        .to_string_lossy()
        .into_owned()
}

#[test]
fn readme_explain_output_matches_the_real_semantic_trace() {
    let path = example_path();
    let checked = run_cli(&["check".to_string(), path.clone()]);
    assert_eq!(checked.status, 0, "{}", checked.stderr);
    assert_eq!(checked.stdout, "OK\n");

    let result = run_cli(&["explain".to_string(), path]);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(
        result.stdout,
        concat!(
            "1. Entered semantic domain scope 'web'.\n",
            "2. 'web' resolved to an HTML web document (default web backend).\n",
            "3. Title explicitly set to 'Example'.\n",
            "4. Added button 'Start' as button-1.\n",
        )
    );
}

#[test]
fn explain_does_not_execute_the_program_or_write_build_artifacts() {
    let path = example_path();
    let result = run_cli(&["explain".to_string(), path]);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert!(result.stderr.is_empty());
    assert!(!result.stdout.contains("Built "));
    assert!(!result.stdout.contains("Planned runtime operation"));
}
