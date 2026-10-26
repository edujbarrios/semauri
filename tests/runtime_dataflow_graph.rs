// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler, RuntimeArgument, Type};
use serde_json::Value;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn ml_source() -> &'static str {
    r#"Within ml:
  Let dataset be Open dataset "./data".
  Let normalized be Transform dataset using "normalize".
  Let model be Load model "resnet18".
  Let device be Select device "cpu".
  Let config be Plan training for 3 epochs using optimizer "adamw" learning rate 0.0003 batch size 16 seed 42 precision "bf16" accumulate 4 steps checkpoint every 100 steps.
  Let run be Fit model using normalized on device with config.
End."#
}

#[test]
fn graph_represents_typed_ml_lineage_with_deterministic_dependencies() {
    let compiler = Compiler::new();
    let graph = compiler.runtime_graph(ml_source()).unwrap();
    assert_eq!(graph["kind"], "runtime_dataflow_graph");
    assert_eq!(graph["schema_version"], 1);
    let nodes = graph["operations"].as_array().unwrap();
    assert_eq!(nodes.len(), 6);
    assert_eq!(nodes[0]["name"], "open_dataset");
    assert_eq!(nodes[1]["depends_on"], serde_json::json!([1]));
    assert_eq!(nodes[5]["name"], "fit");
    assert_eq!(nodes[5]["depends_on"], serde_json::json!([2, 3, 4, 5]));
    assert_eq!(nodes[5]["produces"]["type"]["name"], "training_run");
    assert_eq!(nodes[5]["effects"], serde_json::json!(["compute", "model_training"]));
    assert_eq!(graph, compiler.runtime_graph(ml_source()).unwrap());
    assert!(graph["edges"].as_array().unwrap().iter().any(|edge| {
        edge["producer"] == 2 && edge["consumer"] == 6 && edge["argument"] == "dataset"
    }));
}

#[test]
fn graph_rejects_invalid_future_producer_references_and_type_forgery() {
    let compiler = Compiler::new();
    let mut plan = compiler.runtime_plan(ml_source()).unwrap();
    if let RuntimeArgument::Ref(reference) = plan.operations[1].arguments.get_mut("dataset").unwrap() {
        reference.producer_id = 6;
    } else {
        panic!("Expected an SSA runtime reference");
    }
    assert_eq!(plan.dataflow_graph().unwrap_err().code, "S347");

    let mut plan = compiler.runtime_plan(ml_source()).unwrap();
    if let RuntimeArgument::Ref(reference) = plan.operations[1].arguments.get_mut("dataset").unwrap() {
        reference.ty = Type::Number;
    } else {
        panic!("Expected an SSA runtime reference");
    }
    assert_eq!(plan.dataflow_graph().unwrap_err().code, "S347");
}

#[test]
fn graph_rejects_corrupted_operation_ids_and_mismatched_results() {
    let compiler = Compiler::new();
    let mut plan = compiler.runtime_plan(ml_source()).unwrap();
    plan.operations[1].id = 99;
    assert_eq!(plan.dataflow_graph().unwrap_err().code, "S347");

    let mut plan = compiler.runtime_plan(ml_source()).unwrap();
    plan.operations[0].result.as_mut().unwrap().producer_id = 100;
    assert_eq!(plan.dataflow_graph().unwrap_err().code, "S347");
}

#[test]
fn graph_export_does_not_require_permission_or_execute_ml_effects() {
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    let file = std::env::temp_dir().join(format!(
        "semauri-ml-graph-{}-{}.suri", std::process::id(), id
    ));
    fs::write(&file, ml_source()).unwrap();
    let result = run_cli(&["graph".into(), file.to_string_lossy().into_owned()]);
    fs::remove_file(&file).unwrap();
    assert_eq!(result.status, 0, "{}", result.stderr);
    let graph: Value = serde_json::from_str(&result.stdout).unwrap();
    assert_eq!(graph["operations"].as_array().unwrap().len(), 6);
    assert!(result.stderr.is_empty());
}

#[test]
fn pure_console_operations_remain_valid_without_dependencies() {
    let graph = Compiler::new().runtime_graph("Print 42.").unwrap();
    assert_eq!(graph["operations"][0]["name"], "print");
    assert!(graph["operations"][0]["produces"].is_null());
    assert_eq!(graph["edges"].as_array().unwrap().len(), 0);
}
