// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

//! The README is a concise, executable introduction to Semauri.
use semauri::{run_cli, Compiler};

const README: &str = include_str!("../README.md");
const GUIDE: &str = include_str!("../docs/language-guide.md");
const HELLO: &str = include_str!("../examples/quickstart.suri");

#[test]
fn readme_prioritizes_syntax_core_features_and_running_without_flags() {
    assert!(README.lines().count() <= 160, "Keep detailed documentation in the language guide");
    for required in [
        "## Install",
        "## Your first Semauri program",
        "## Language essentials",
        "## Main features",
        "## Commands",
        "## Learn and extend",
        "## Citation and license",
        "semauri hello.suri",
        "semauri run hello.suri",
        "semauri test PATH",
        "semauri effects FILE.suri",
        "[language reference](docs/language-guide.md)",
        "[CITATION.cff](CITATION.cff)",
    ] {
        assert!(README.contains(required), "README missing: {required}");
    }
    assert!(GUIDE.lines().count() > 900, "Full reference must remain available");
    assert!(GUIDE.contains("### Types"));
    assert!(GUIDE.contains("## Compiler commands"));
}

#[test]
fn readme_quickstart_is_real_executable_code_without_grants() {
    assert!(README.contains(HELLO.trim()), "README must reuse the tested quickstart");
    Compiler::new().analyze(HELLO).expect("Quickstart must type-check");
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/quickstart.suri");
    for arguments in [
        vec!["run".into(), path.into()],
        vec![path.into()],
    ] {
        let result = run_cli(&arguments);
        assert_eq!(result.status, 0, "{}", result.stderr);
        assert_eq!(result.stdout, "[2, 4, 6]\n");
    }
}

#[test]
fn readme_explains_default_output_without_removing_sensitive_effect_gates() {
    assert!(README.contains("No `--allow-console` or `--allow console_write` is needed."));
    assert!(README.contains("--allow secure_random"));
    assert!(README.contains("--deny-console"));
    assert!(README.contains("Other effects **still require permission**"));
}

#[test]
fn advanced_prediction_example_still_checks_and_runs_without_console_flags() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/prediction_errors.suri");
    let checked = run_cli(&["check".into(), path.into()]);
    assert_eq!(checked.status, 0, "{}", checked.stderr);
    let result = run_cli(&["run".into(), path.into()]);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "3\n3\n9\n");
}

#[test]
fn benchmarks_are_still_documented_in_their_dedicated_reference() {
    assert!(README.contains("[Benchmarks](benches/BASELINE.md)"));
    let baseline = include_str!("../benches/BASELINE.md");
    assert!(baseline.contains("bash scripts/bench.sh 50"));
    assert!(baseline.contains("same hosted runner"));
}
