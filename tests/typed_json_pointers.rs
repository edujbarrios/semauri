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

fn source(document: &str, pointer: &str, ty: &str) -> String {
    format!("print json {ty} at pointer {} in {}.",
        serde_json::to_string(pointer).unwrap(),
        serde_json::to_string(document).unwrap())
}

#[test]
fn nested_objects_and_arrays_are_typed() {
    let result = run(include_str!("../examples/json_pointers.suri"));
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Ada\n");
}

#[test]
fn supports_pointer_escaping_and_array_indices() {
    let doc = r#"{"a/b":{"~value":true},"values":["zero","one"]}"#;
    let code = format!("assert (json boolean at pointer \"/a~1b/~0value\" in {}) is equal to true.\nassert (json string at pointer \"/values/1\" in {}) is equal to \"one\".",
        serde_json::to_string(doc).unwrap(), serde_json::to_string(doc).unwrap());
    let result = run(&code);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn empty_pointer_reads_root_value() {
    let result = run(&source("[1,2,3]", "", "numbers"));
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "[1, 2, 3]\n");
}

#[test]
fn malformed_paths_wrong_types_and_missing_values_are_diagnostic() {
    let doc = r#"{"users":[{"name":"Ada"}],"a/b":true}"#;
    for (path, ty) in [
        ("/users/1/name", "string"), ("/users/01/name", "string"),
        ("/users/-/name", "string"), ("/a~2b", "boolean"),
        ("users/0/name", "string"), ("/users/0/name", "number"),
        ("/users/0/missing", "string"), ("/users/0", "strings"),
    ] {
        let result = run(&source(doc, path, ty));
        assert_eq!(result.status, 65, "{path}: {}", result.stderr);
        assert!(result.stderr.contains("S405"), "{path}: {}", result.stderr);
    }
    let huge = "{\"count\":9223372036854775808}";
    let result = run(&source(huge, "/count", "number"));
    assert!(result.stderr.contains("S405"), "{}", result.stderr);
}

#[test]
fn semantic_checks_and_constant_folding_work() {
    let compiler = Compiler::new();
    for input in [
        "print json number at pointer 2 in \"{}\".",
        "print json number at pointer \"/x\" in true.",
    ] {
        let err = compiler.hir(input).unwrap_err();
        assert_eq!(err.code, "S404", "{input}: {err}");
    }
    let formatted = compiler.format_source("PRINT JSON NUMBER AT POINTER \"/x\" IN \"{}\".").unwrap();
    assert_eq!(formatted, "print json number at pointer \"/x\" in \"{}\".");
    let text = source(r#"{"x":42}"#, "/x", "number");
    let folded = compiler.optimized_hir(&text).unwrap().to_json().to_string();
    assert!(folded.contains("42"), "{folded}");
    assert!(!folded.contains("json_pointer_number"), "{folded}");
    assert_eq!(compiler.effect_analysis(&text).unwrap().effects(), vec!["console_write".to_string()]);
}
