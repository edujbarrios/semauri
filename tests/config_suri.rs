// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, ProgramConfig};
use serde_json::json;
use std::fs;

const CONFIG: &str = r#"
# Checked, declarative configuration.
parameter model = "vision-model".
parameter temperature = 0.2.
parameter tags = ["agent", "image"].
environment api_key = "SEMAURI_TEST_API_KEY".
"#;

#[test]
fn parses_typed_parameters_and_resolves_environment_lazily() {
    let config = ProgramConfig::parse(CONFIG).unwrap();
    let resolved = config
        .resolve_with(|name| (name == "SEMAURI_TEST_API_KEY").then(|| "secret".to_string()))
        .unwrap();
    assert_eq!(resolved.get("model"), Some(&json!("vision-model")));
    assert_eq!(resolved.get("temperature"), Some(&json!(0.2)));
    assert_eq!(resolved.get("api_key"), Some(&json!("secret")));
    assert!(resolved.is_environment_binding("api_key"));
}

#[test]
fn summaries_never_expose_environment_values() {
    let config = ProgramConfig::parse(CONFIG).unwrap();
    let summary = config.summary_json(|name| name == "SEMAURI_TEST_API_KEY");
    assert_eq!(summary["environment"][0]["set"], true);
    assert_eq!(
        summary["environment"][0]["variable"],
        "SEMAURI_TEST_API_KEY"
    );
    assert!(!summary.to_string().contains("secret"));
}

#[test]
fn validates_names_duplicates_syntax_and_overrides() {
    for source in [
        "parameter bad-name = 1.",
        "parameter x = 1.\nparameter x = 2.",
        "environment token = OPENAI_API_KEY.",
        "parameter x = 1",
    ] {
        assert!(ProgramConfig::parse(source).is_err(), "{source}");
    }

    let mut config = ProgramConfig::parse("parameter turns = 3.").unwrap();
    config.set_parameter("turns", json!(5)).unwrap();
    assert_eq!(config.resolve().unwrap().get("turns"), Some(&json!(5)));
    assert!(config.set_parameter("missing", json!(1)).is_err());
}

#[test]
fn cli_validates_files_and_applies_declared_overrides() {
    let path = std::env::temp_dir().join(format!("semauri-config-{}.suri", std::process::id()));
    fs::write(
        &path,
        "parameter turns = 3.\nenvironment token = \"MISSING_SEMAURI_TEST_TOKEN\".",
    )
    .unwrap();
    let result = run_cli(&[
        "config".into(),
        path.to_string_lossy().into_owned(),
        "--set".into(),
        "turns=5".into(),
    ]);
    assert_eq!(result.status, 0, "{}", result.stderr);
    let output: serde_json::Value = serde_json::from_str(&result.stdout).unwrap();
    assert_eq!(output["parameters"]["turns"], 5);
    assert_eq!(output["environment"][0]["set"], false);
    assert!(!result.stdout.contains("environment value"));

    let checked = run_cli(&[
        "config".into(),
        path.to_string_lossy().into_owned(),
        "--check-env".into(),
    ]);
    assert_eq!(checked.status, 64);
    assert!(checked.stderr.contains("MISSING_SEMAURI_TEST_TOKEN"));
    fs::remove_file(path).unwrap();
}
