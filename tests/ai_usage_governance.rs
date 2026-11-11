// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use serde_json::Value;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-ai-governance-{}-{}.suri",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&[
        "run".into(), path.to_string_lossy().into_owned(),
        "--allow".into(), "model_load".into(),
        "--allow".into(), "network".into(),
        "--allow".into(), "model_inference".into(),
        "--allow".into(), "console_write".into(),
    ]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn ai_usage_reports_cache_hits_and_limits_without_prompt_contents() {
    let source = include_str!("../examples/ai_governance.suri");
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    let usage: Value = serde_json::from_str(result.stdout.trim()).unwrap();
    assert_eq!(usage["requests"], 1);
    assert_eq!(usage["request_limit"], 1);
    assert_eq!(usage["prompt_character_limit"], 4);
    assert_eq!(usage["cache_hits"], 1);
    assert_eq!(usage["cache_entries"], 1);
    assert!(!result.stdout.contains("hola"));
}

#[test]
fn prompt_bounds_count_unicode_scalars_and_reject_oversize_inputs() {
    let fine = run("bound ai prompts to 2 characters. let model be connect language model \"local/test\". let reply be prompt model with \"á🙂\". print reply.");
    assert_eq!(fine.status, 0, "{}", fine.stderr);

    let too_long = run("bound ai prompts to 1 characters. let model be connect language model \"local/test\". let reply be prompt model with \"á🙂\". print reply.");
    assert_eq!(too_long.status, 70);
    assert!(too_long.stderr.contains("AI prompt exceeds character limit"), "{}", too_long.stderr);
}

#[test]
fn prompt_bound_also_limits_embedded_text_and_embeddings_spend_budget() {
    let too_long = run("bound ai prompts to 1 characters. let model be connect language model \"local/test\". let vector be embed \"two\" using model.");
    assert_eq!(too_long.status, 70);
    assert!(too_long.stderr.contains("AI prompt exceeds character limit"));

    let budget = run("cap ai requests to 1. let model be connect language model \"local/test\". let first be embed \"a\" using model. let second be embed \"b\" using model.");
    assert_eq!(budget.status, 70);
    assert!(budget.stderr.contains("AI request budget exhausted"));
}

#[test]
fn invalid_prompt_bound_and_static_type_errors_are_rejected() {
    for count in ["minus 1", "1.5"] {
        let source = format!("bound ai prompts to {count} characters.");
        let result = run(&source);
        assert_eq!(result.status, 70, "{count}: {}", result.stderr);
        assert!(result.stderr.contains("non-negative integer"));
    }
    let bad = Compiler::new().hir("bound ai prompts to \"many\" characters.");
    assert!(bad.is_err());
}

#[test]
fn usage_report_is_pure_and_available_to_static_analysis() {
    let source = "let usage be report ai usage.";
    let compiler = Compiler::new();
    assert!(compiler.hir(source).is_ok());
    assert!(compiler.effect_analysis(source).unwrap().effects().is_empty());
}
