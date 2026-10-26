use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!("semauri-json-field-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(),
        "console_write".into(), path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn extracts_all_six_types_with_precise_immutability() {
    let result = run(include_str!("../examples/json_field_extraction.suri"));
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Ada\n");
}

#[test]
fn composes_with_existing_json_serialization_and_expression_forms() {
    let doc = "{\"score\":17,\"enabled\":false}";
    let program = format!("let document be {}.\nlet key be \"score\".\nassert (json number at key in document) is equal to 17.\nassert not (json boolean at \"enabled\" in document).", JSON.stringify(doc));
    let result = run(&program);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn missing_null_and_mismatched_fields_are_located_errors() {
    for (document,key,kind) in [
        (r#"{"name":"Ada"}"#,"missing","string"),
        (r#"{"nil":null}"#,"nil","string"),
        (r#"{"count":"3"}"#,"count","number"),
        (r#"{"count":3}"#,"count","boolean"),
        (r#"{"list":[1,"wrong"]}"#,"list","numbers"),
        (r#"{"count":9223372036854775808}"#,"count","number"),
        (r#"{"count":-9223372036854775809}"#,"count","number"),
        ("[]","name","string"),
    ] {
        let source = format!("print json {kind} at {} in {}.",
            serde_json::to_string(key).unwrap(),
            serde_json::to_string(document).unwrap());
        let result = run(&source);
        assert_eq!(result.status, 65, "{source}: {}", result.stderr);
        assert!(result.stderr.contains("S401"), "{source}: {}", result.stderr);
    }
}

#[test]
fn wrong_input_types_fail_during_semantic_check() {
    let compiler = Compiler::new();
    for source in [
        "print json number at 2 in \"{}\".",
        "print json string at \"x\" in 12.",
        "print json booleans at true in false.",
    ] {
        let err = compiler.hir(source).unwrap_err();
        assert_eq!(err.code, "S400", "{source}: {err}");
        assert!(err.span.is_some());
    }
}

#[test]
fn expressions_are_contextual_and_folding_is_safe() {
    let compiler = Compiler::new();
    let formatted = compiler.format_source("PRINT JSON NUMBER AT \"x\" IN \"{}\".").unwrap();
    assert_eq!(formatted, "print json number at \"x\" in \"{}\".");
    let original = "print json number at \"x\" in \"{\\\"x\\\":42}\".";
    let optimized = compiler.optimized_hir(original).unwrap().to_json().to_string();
    assert!(optimized.contains("42"), "{optimized}");
    assert!(!optimized.contains("json_field_number"), "{optimized}");
    let effects = compiler.effect_analysis("print json string at \"x\" in \"{}\".").unwrap();
    assert_eq!(effects.effects(), vec!["console_write".to_string()]);
    let result = run("let json be \"ok\".\nassert json is equal to \"ok\".");
    assert_eq!(result.status, 0, "{}", result.stderr);
}
