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
    // Keep the front page concise while showing the new secure-password use case.
    assert!(lines <= 226, "README grew to {lines} lines; keep details in docs/language-guide.md");
    for required in [
        "## 1. Install",
        "## 2. Your first program",
        "## 3. Learn by example",
        "## 4. Command reference",
        "Create your own Semauri-compatible library",
        "[step-by-step guide](docs/external-libraries.md)",
        "## Citation",
        "[Language reference](docs/language-guide.md)",
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
    assert!(README.contains("https://github.com/edujbarrios/semauri/actions/runs/41132112869"));
    assert!(README.contains("[complete example](examples/linear_regression.suri)"));
    assert!(README.contains("semauri run --allow console_write examples/linear_regression.suri"));
    assert!(README.contains("predicts `13` for `x = 6`"));
    let baseline = include_str!("../benches/BASELINE.md");
    assert!(baseline.contains("bash scripts/bench.sh 50"));
    assert!(baseline.contains("same hosted runner"));
}

#[test]
fn prediction_errors_readme_example_runs_and_matches_documented_output() {
    const EXAMPLE: &str = include_str!("../examples/prediction_errors.suri");
    assert!(README.contains(EXAMPLE.trim()), "README snippet must match the runnable .suri file");
    assert!(README.contains("semauri run --allow console_write examples/prediction_errors.suri"));
    assert!(README.contains("Output (residual, absolute error, squared error):"));
    assert!(README.contains("```text\n3\n3\n9\n```"));

    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/prediction_errors.suri");
    let checked = run_cli(&["check".into(), path.into()]);
    assert_eq!(checked.status, 0, "{}", checked.stderr);
    let result = run_cli(&[
        "run".into(), "--allow".into(), "console_write".into(), path.into()
    ]);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "3\n3\n9\n");
}
