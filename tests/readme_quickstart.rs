// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

//! Keep the public quick start concise, runnable and faithful to Semauri.
use semauri::{run_cli, Compiler};

const README: &str = include_str!("../README.md");
const GUIDE: &str = include_str!("../docs/language-guide.md");
const HELLO: &str = include_str!("../examples/quickstart.suri");

#[test]
fn front_page_is_short_but_links_to_complete_reference() {
    let lines = README.lines().count();
    assert!(lines <= 110, "README grew to {lines} lines; keep details in docs/language-guide.md");
    for required in [
        "## Quick Start",
        "## Commands",
        "## Real ML demo: linear regression",
        "## Citation",
        "[complete language guide](docs/language-guide.md)",
        "[CITATION.cff](CITATION.cff)",
    ] {
        assert!(README.contains(required), "README missing: {required}");
    }
    assert!(GUIDE.lines().count() > 900, "the full guide must not be discarded");
    assert!(GUIDE.contains("### Types"));
    assert!(GUIDE.contains("## Compiler commands"));
}

#[test]
fn quickstart_code_in_readme_is_real_executable_semauri() {
    assert!(README.contains(HELLO.trim()), "README snippet differs from executable example");
    Compiler::new().analyze(HELLO).expect("Quick Start must type-check");
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/quickstart.suri");
    let result = run_cli(&[
        "run".into(), "--allow".into(), "console_write".into(), path.into(),
    ]);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[2, 4, 6]\n");
}

#[test]
fn benchmark_section_is_linked_to_verifiable_execution() {
    assert!(README.contains("1.802 ms median"));
    assert!(README.contains("2.159 ms"));
    assert!(README.contains("https://github.com/edujbarrios/semauri/actions/runs/41132112869"));
    assert!(README.contains("Prediction for x=6:\n13"));
    assert!(README.contains("excludes compilation"));
}
