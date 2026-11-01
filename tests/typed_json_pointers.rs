use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!("semauri-pointer-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(),
        "console_write".into(), path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

fn expression(document: &str, pointer: &str) -> String {
    format!("json at pointer {} in {}",
        serde_json::to_string(pointer).unwrap(),
        serde_json::to_string(document).unwrap())
}

#[test]
fn decodes_nested_json_with_rfc6901_escapes() {
    let result = run(include_str!("../examples/json_pointers.suri"));
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Ada\n");
}

#[test]
fn empty_pointer_reads_the_json_root() {
    let inner = expression("\"hello\"", "");
    let result = run(&format!("assert (decode json as string from ({inner})) is equal to \"hello\"."));
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn missing_invalid_and_ambiguous_pointers_produce_diagnostics() {
    let document = r#"{"users":[{"name":"Ada"}],"a/b":true}"#;
    for pointer in [
        "/users/1/name", "/users/01/name", "/users/-/name",
        "/users/-1/name", "/a~2b", "users/0/name",
        "/users/0/missing", "/users/0/name/child",
    ] {
        let result = run(&format!("print {}.", expression(document, pointer)));
        assert_eq!(result.status, 65, "{pointer}: {}", result.stderr);
        assert!(result.stderr.contains("S407"), "{pointer}: {}", result.stderr);
    }
}

#[test]
fn semantic_errors_and_contextual_identifiers() {
    let compiler = Compiler::new();
    for source in [
        "print json at pointer 7 in \"{}\".",
        "print json at pointer \"/x\" in true.",
    ] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S406", "{source}: {error}");
        assert!(error.span.is_some());
    }
    let result = run(r#"let pointer be "/x".
let document be "{\"x\":true}".
let active be decode json as boolean from (json at pointer pointer in document).
assert active."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn formatter_and_optimizer_use_the_same_semantics() {
    let compiler = Compiler::new();
    assert_eq!(compiler.format_source("PRINT JSON AT POINTER \"/x\" IN \"{}\".").unwrap(),
        "print json at pointer \"/x\" in \"{}\".");
    let code = format!("print {}.", expression(r#"{"x":42}"#, "/x"));
    let optimized = compiler.optimized_hir(&code).unwrap().to_json().to_string();
    assert!(!optimized.contains("json_pointer"), "{optimized}");
    assert!(optimized.contains("42"), "{optimized}");
    assert_eq!(compiler.effect_analysis(&code).unwrap().effects(),
        vec!["console_write".to_string()]);
}
