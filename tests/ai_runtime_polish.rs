// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use serde_json::Value;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-ai-polish-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&[
        "run".into(), path.to_string_lossy().into_owned(),
        "--allow".into(), "model_load".into(),
        "--allow".into(), "network".into(),
        "--allow".into(), "model_inference".into(),
        "--allow".into(), "tool_execution".into(),
        "--allow".into(), "console_write".into(),
    ]);
    fs::remove_file(path).unwrap();
    result
}

fn report(source: &str) -> Value {
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    serde_json::from_str(result.stdout.trim()).unwrap()
}

#[test]
fn bounded_fifo_cache_evicts_oldest_and_reports_counts() {
    let usage = report(r#"
cap ai requests to 3.
memoize ai response cache.
limit ai cache to 1 entries.
let model be connect language model "local/test".
let a be prompt model with "a".
let b be prompt model with "b".
let again be prompt model with "a".
print report ai usage.
"#);
    assert_eq!(usage["requests"], 3);
    assert_eq!(usage["cache_hits"], 0);
    assert_eq!(usage["cache_entries"], 1);
    assert_eq!(usage["cache_capacity"], 1);
    assert_eq!(usage["cache_evictions"], 2);
}

#[test]
fn cache_hit_does_not_refresh_fifo_order() {
    let usage = report(r#"
memoize ai response cache.
limit ai cache to 2 entries.
let model be connect language model "local/test".
let a be prompt model with "a".
let b be prompt model with "b".
let a_again be prompt model with "a".
let c be prompt model with "c".
let a_recomputed be prompt model with "a".
print report ai usage.
"#);
    assert_eq!(usage["requests"], 4);
    assert_eq!(usage["cache_hits"], 1);
    assert_eq!(usage["cache_entries"], 2);
    assert_eq!(usage["cache_evictions"], 2);
}

#[test]
fn reducing_capacity_and_reset_keep_cumulative_metrics() {
    let usage = report(r#"
memoize ai response cache.
let model be connect language model "local/test".
let a be prompt model with "a".
let b be prompt model with "b".
limit ai cache to 1 entries.
reset ai response cache.
print report ai usage.
"#);
    assert_eq!(usage["requests"], 2);
    assert_eq!(usage["cache_entries"], 0);
    assert_eq!(usage["cache_evictions"], 1);
    assert_eq!(usage["cache_capacity"], 1);
}

#[test]
fn zero_capacity_prevents_caching_but_not_inference() {
    let usage = report(r#"
memoize ai response cache.
limit ai cache to 0 entries.
let model be connect language model "local/test".
let a be prompt model with "a".
let again be prompt model with "a".
print report ai usage.
"#);
    assert_eq!(usage["requests"], 2);
    assert_eq!(usage["cache_entries"], 0);
    assert_eq!(usage["cache_hits"], 0);
}

#[test]
fn output_limit_rejects_new_and_previously_cached_responses() {
    let new_result = run(r#"
restrict ai outputs to 5 characters.
let model be connect language model "local/test".
let reply be prompt model with "hello".
print reply.
"#);
    assert_eq!(new_result.status, 70);
    assert!(new_result.stderr.contains("AI output exceeds character limit"), "{}", new_result.stderr);

    let cached_result = run(r#"
memoize ai response cache.
restrict ai outputs to 500 characters.
let model be connect language model "local/test".
let first be prompt model with "hello".
restrict ai outputs to 2 characters.
let again be prompt model with "hello".
print again.
"#);
    assert_eq!(cached_result.status, 70);
    assert!(cached_result.stderr.contains("AI output exceeds character limit"), "{}", cached_result.stderr);
}

#[test]
fn agent_enforces_max_turns_instead_of_ignoring_configuration() {
    let result = run(r#"
let model be connect language model "local/test".
let agent be compose agent using model system "Be short." max 1 turns.
let first be ask agent with "a".
let second be ask agent with "b".
print second.
"#);
    assert_eq!(result.status, 70);
    assert!(result.stderr.contains("AI agent turn limit exhausted"), "{}", result.stderr);

    let usage = report(r#"
memoize ai response cache.
let model be connect language model "local/test".
let first_agent be compose agent using model system "A" max 2 turns.
let second_agent be compose agent using model system "B" max 1 turns.
let a be ask first_agent with "hi".
let again be ask first_agent with "hi".
let b be ask second_agent with "hi".
print report ai usage.
"#);
    assert_eq!(usage["requests"], 2);
    assert_eq!(usage["cache_hits"], 1);
    assert_eq!(usage["agent_turns"], 3);
    assert_eq!(usage["agent_count"], 2);
}

#[test]
fn agent_prompt_limit_accounts_for_system_context() {
    let result = run(r#"
bound ai prompts to 3 characters.
let model be connect language model "local/test".
let agent be compose agent using model system "long" max 1 turns.
let answer be ask agent with "a".
print answer.
"#);
    assert_eq!(result.status, 70);
    assert!(result.stderr.contains("AI prompt exceeds character limit"), "{}", result.stderr);
}

#[test]
fn counts_are_validated_and_pure_controls_are_typed() {
    for expression in [
        "limit ai cache to minus 1 entries.",
        "limit ai cache to 1.5 entries.",
        "restrict ai outputs to minus 1 characters.",
        "restrict ai outputs to 1.5 characters.",
        "let model be connect language model \"local/test\". let agent be compose agent using model system \"x\" max 1.5 turns.",
    ] {
        let result = run(expression);
        assert_eq!(result.status, 70, "{expression}: {}", result.stderr);
        assert!(result.stderr.contains("non-negative integer"), "{expression}: {}", result.stderr);
    }

    let compiler = Compiler::new();
    let source = "limit ai cache to 2 entries. restrict ai outputs to 16 characters. reset ai response cache. let metrics be report ai usage.";
    compiler.hir(source).unwrap();
    assert!(compiler.effect_analysis(source).unwrap().effects().is_empty());

    let formatted = compiler.format_source("LIMIT AI CACHE TO 2 ENTRIES. RESTRICT AI OUTPUTS TO 16 CHARACTERS. RESET AI RESPONSE CACHE.").unwrap();
    assert!(formatted.contains("limit ai cache to 2 entries."));
    assert!(formatted.contains("restrict ai outputs to 16 characters."));
    assert!(formatted.contains("reset ai response cache."));
}
