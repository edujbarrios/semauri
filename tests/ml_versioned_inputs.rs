// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{Compiler, RuntimeArgument, Type};

const REPRODUCIBLE_ML_SOURCE: &str = r#"Within ml:
  Let dataset be Source dataset "./images" revision "data-v3".
  Let model be Resolve model "resnet18" revision "weights-2026-10".
  Let device be Select device "cpu".
  Let config be Plan training for 3 epochs using optimizer "adamw" learning rate 0.0003 batch size 16 seed 42 precision "fp32" accumulate 2 steps checkpoint every 100 steps.
  Let run be Fit model using dataset on device with config.
End."#;

#[test]
fn revision_pinned_inputs_flow_through_typed_training_plan() {
    let compiler = Compiler::new();
    let plan = compiler.runtime_plan(REPRODUCIBLE_ML_SOURCE).unwrap();
    assert_eq!(plan.operations.len(), 5);
    assert_eq!(plan.operations[0].name, "source_dataset_revision");
    assert_eq!(plan.operations[1].name, "resolve_model_revision");

    let json = plan.to_json();
    assert_eq!(json["operations"][0]["arguments"]["source"], "./images");
    assert_eq!(json["operations"][0]["arguments"]["revision"], "data-v3");
    assert_eq!(json["operations"][1]["arguments"]["identifier"], "resnet18");
    assert_eq!(json["operations"][1]["arguments"]["revision"], "weights-2026-10");
    assert_eq!(json["operations"][4]["name"], "fit");
    let RuntimeArgument::Ref(dataset) = &plan.operations[4].arguments["dataset"] else {
        panic!("training dataset is not an SSA reference");
    };
    assert_eq!(dataset.producer_id, 1);
    assert_eq!(dataset.ty.to_string(), "ml.dataset");
    let RuntimeArgument::Ref(model) = &plan.operations[4].arguments["model"] else {
        panic!("training model is not an SSA reference");
    };
    assert_eq!(model.producer_id, 2);
    assert_eq!(model.ty.to_string(), "ml.model");
    assert_eq!(plan.operations[0].result.as_ref().unwrap().ty.to_string(), "ml.dataset");
}

#[test]
fn pinned_sources_keep_conservative_effect_checks() {
    let compiler = Compiler::new();
    let effects = compiler.effect_analysis(REPRODUCIBLE_ML_SOURCE)
        .unwrap().effects();
    assert!(effects.contains(&"filesystem_read".to_string()));
    assert!(effects.contains(&"model_load".to_string()));
    assert!(effects.contains(&"model_training".to_string()));
    assert!(effects.contains(&"compute".to_string()));
    assert!(compiler.validate_capabilities(
        REPRODUCIBLE_ML_SOURCE,
        &semauri::CapabilityPolicy::allow_none()
    ).is_err());
}

#[test]
fn blank_revision_and_source_values_are_rejected_during_type_checking() {
    let compiler = Compiler::new();
    for source in [
        r#"Within ml: Let data be Source dataset "./data" revision "". End."#,
        r#"Within ml: Let data be Source dataset "" revision "v3". End."#,
        r#"Within ml: Let model be Resolve model "resnet18" revision "  ". End."#,
        r#"Within ml: Let model be Resolve model " " revision "v1". End."#,
    ] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S348", "{source}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn revision_arguments_remain_strictly_strings() {
    let compiler = Compiler::new();
    let error = compiler.hir(
        "Within ml: Let data be Source dataset \"./data\" revision 12. End."
    ).unwrap_err();
    assert_eq!(error.code, "S329");
}

#[test]
fn old_unversioned_inputs_still_compile_for_backward_compatibility() {
    let plan = Compiler::new().runtime_plan(
        "Within ml: Let d be Open dataset \"./data\". Let m be Load model \"resnet18\". End."
    ).unwrap();
    assert_eq!(plan.operations[0].name, "open_dataset");
    assert_eq!(plan.operations[1].name, "load_model");
}
