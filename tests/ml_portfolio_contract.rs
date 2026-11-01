// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, CapabilityPolicy, Compiler};
use serde_json::Value;

const SOURCE: &str = include_str!("../examples/ml_versioned_inputs.suri");

#[test]
fn checked_in_ml_example_has_deterministic_and_typed_lineage() {
    let compiler = Compiler::new();
    let graph = compiler.runtime_graph(SOURCE).unwrap();
    assert_eq!(graph["kind"], "runtime_dataflow_graph");
    assert_eq!(graph["schema_version"], 1);
    let operations = graph["operations"].as_array().unwrap();
    assert_eq!(operations.len(), 5);
    assert_eq!(operations[0]["name"], "source_dataset_revision");
    assert_eq!(operations[1]["name"], "resolve_model_revision");
    assert_eq!(operations[4]["name"], "fit");
    assert_eq!(operations[4]["depends_on"], serde_json::json!([1, 2, 3, 4]));
    assert_eq!(graph, compiler.runtime_graph(SOURCE).unwrap());
}

#[test]
fn release_example_declares_pinned_inputs_but_does_not_claim_verification() {
    let report = Compiler::new().ml_reproducibility_audit(SOURCE).unwrap();
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["warning_count"], 0);
    assert_eq!(report["status"], "declarations_complete");
    assert_eq!(report["verification"], "declarations_only");
    assert_eq!(report["declared_inputs"][0]["verified"], false);
    assert_eq!(report["declared_inputs"][1]["verified"], false);
}

#[test]
fn reproducibility_preflight_cannot_execute_training_without_capabilities() {
    let compiler = Compiler::new();
    assert!(compiler.validate_capabilities(SOURCE, &CapabilityPolicy::allow_none()).is_err());
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/ml_versioned_inputs.suri");
    let result = run_cli(&["audit".into(), path.into(), "--strict".into()]);
    assert_eq!(result.status, 0, "{}", result.stderr);
    let report: Value = serde_json::from_str(&result.stdout).unwrap();
    assert_eq!(report["warning_count"], 0);
}
