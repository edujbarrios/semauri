// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::Compiler;

const AGENT: &str = include_str!("../examples/ai_agent.suri");
const VISION: &str = include_str!("../examples/vision.suri");

#[test]
fn language_model_and_bounded_agent_form_a_typed_plan() {
    let compiler = Compiler::new();
    let plan = compiler.runtime_plan(AGENT).unwrap().to_json();
    let operations = plan["operations"].as_array().unwrap();
    assert_eq!(operations.len(), 6);
    assert_eq!(operations[0]["name"], "read_string");
    assert_eq!(operations[1]["name"], "read_number");
    assert_eq!(operations[2]["name"], "language_model");
    assert_eq!(operations[3]["name"], "bounded_agent");
    assert_eq!(operations[4]["name"], "run_agent");
    assert_eq!(operations[4]["result"]["type"]["domain"], "ai");
    assert!(compiler
        .effect_analysis(AGENT)
        .unwrap()
        .effects()
        .contains(&"tool_execution".to_string()));
}

#[test]
fn vision_input_keeps_file_network_and_inference_effects_explicit() {
    let compiler = Compiler::new();
    compiler.hir(VISION).unwrap();
    let effects = compiler.effect_analysis(VISION).unwrap().effects();
    for expected in [
        "config_read",
        "filesystem_read",
        "model_inference",
        "model_load",
        "network",
    ] {
        assert!(
            effects.contains(&expected.to_string()),
            "{expected}: {effects:?}"
        );
    }
}

#[test]
fn model_kinds_are_not_interchangeable() {
    let source = r#"within ai:
  let model be activate vision model "vision".
  let answer be prompt model with "hello".
end."#;
    let error = Compiler::new().hir(source).unwrap_err();
    assert_eq!(error.code, "S329");
}

#[test]
fn generic_embedding_and_tool_primitives_are_registered() {
    let compiler = Compiler::new();
    let metadata = compiler.domains().to_json().to_string();
    for operation in ["embed_text", "typed_tool", "inspect_image", "generate_text"] {
        assert!(metadata.contains(operation), "{operation}");
    }
}
