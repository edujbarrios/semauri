// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use serde_json::Value;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

const PINNED_WORKFLOW: &str = r#"Within ml:
  Let dataset be Source dataset "./data" revision "dataset-v3".
  Let model be Resolve model "resnet18" revision "weights-v2".
  Let device be Select device "cpu".
  Let config be Configure training for 3 epochs using optimizer "adamw" learning rate 0.001 batch size 16 seed 42.
  Let run be Fit model using dataset on device with config.
End."#;

const UNPINNED_WORKFLOW: &str = r#"Within ml:
  Let dataset be Open dataset "./data".
  Let model be Load model "resnet18".
  Let device be Select device "cpu".
  Let run be Train model using dataset on device for 3 epochs.
End."#;

fn audit_cli(source: &str, strict: bool) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-audit-{}-{}.suri",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
    ));
    fs::write(&path, source).unwrap();
    let mut args = vec!["audit".into(), path.to_string_lossy().into_owned()];
    if strict {
        args.push("--strict".into());
    }
    let result = run_cli(&args);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn audit_reports_unversioned_inputs_and_unseeded_train_operation() {
    let compiler = Compiler::new();
    let report = compiler.ml_reproducibility_audit(UNPINNED_WORKFLOW).unwrap();
    assert_eq!(report["kind"], "ml_reproducibility_audit");
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["status"], "review_required");
    assert_eq!(report["warning_count"], 3);
    assert_eq!(report["warnings"][0]["code"], "ML001");
    assert_eq!(report["warnings"][1]["code"], "ML002");
    assert_eq!(report["warnings"][2]["code"], "ML003");
    assert!(report["warnings"][0]["source_span"]["start"]["line"].as_u64().is_some());
}

#[test]
fn pinned_workflow_has_no_advisory_warnings_but_is_not_verified() {
    let compiler = Compiler::new();
    let report = compiler.ml_reproducibility_audit(PINNED_WORKFLOW).unwrap();
    assert_eq!(report["warning_count"], 0);
    assert_eq!(report["status"], "declarations_complete");
    assert_eq!(report["verification"], "declarations_only");
    assert_eq!(report["declared_inputs"][0]["kind"], "dataset");
    assert_eq!(report["declared_inputs"][0]["declared_revision"], "dataset-v3");
    assert_eq!(report["declared_inputs"][0]["verified"], false);
    assert_eq!(report["declared_inputs"][1]["declared_revision"], "weights-v2");
    assert_eq!(report, compiler.ml_reproducibility_audit(PINNED_WORKFLOW).unwrap());
}

#[test]
fn strict_cli_can_fail_ci_without_executing_ml_workload() {
    let unsafe_result = audit_cli(UNPINNED_WORKFLOW, true);
    assert_eq!(unsafe_result.status, 2, "{}", unsafe_result.stderr);
    let unsafe_report: Value = serde_json::from_str(&unsafe_result.stdout).unwrap();
    assert_eq!(unsafe_report["warning_count"], 3);

    let review_result = audit_cli(UNPINNED_WORKFLOW, false);
    assert_eq!(review_result.status, 0, "{}", review_result.stderr);

    let clean_result = audit_cli(PINNED_WORKFLOW, true);
    assert_eq!(clean_result.status, 0, "{}", clean_result.stderr);
    let clean_report: Value = serde_json::from_str(&clean_result.stdout).unwrap();
    assert_eq!(clean_report["warning_count"], 0);
}

#[test]
fn audit_rejects_non_ml_sources_and_preserves_compiler_errors() {
    let error = Compiler::new()
        .ml_reproducibility_audit("Print 42.")
        .unwrap_err();
    assert_eq!(error.code, "S349");

    let invalid = audit_cli(
        "Within ml: Let dataset be Source dataset \"\" revision \"v3\". End.",
        false,
    );
    assert_eq!(invalid.status, 65);
    assert!(invalid.stderr.contains("S348"));
}

#[test]
fn audit_is_a_report_and_does_not_grant_training_capabilities() {
    let compiler = Compiler::new();
    let err = compiler
        .validate_capabilities(PINNED_WORKFLOW, &semauri::CapabilityPolicy::allow_none())
        .unwrap_err();
    assert_eq!(err.code, "S334");
    assert!(compiler.ml_reproducibility_audit(PINNED_WORKFLOW).is_ok());
}
