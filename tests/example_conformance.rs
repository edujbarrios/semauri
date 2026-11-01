// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

//! The public examples are executable documentation, not untested pseudocode.
//! These tests deliberately use the checked-in files rather than duplicating
//! their sources in test strings.

use semauri::{run_cli, Compiler};
use std::fs;
use std::path::{Path, PathBuf};

fn example(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

const WEB_EXAMPLES: &[&str] = &[
    "hello.suri",
    "control_flow.suri",
    "context.suri",
    "shop.suri",
    "variables.suri",
    "otherwise_if.suri",
    "unless_unicode.suri",
];

#[test]
fn all_checked_in_examples_pass_cli_check_and_compile() {
    let compiler = Compiler::new();
    for name in WEB_EXAMPLES.iter().copied().chain([
        "hello_console.suri",
        "filesystem.suri",
        "pet_schema.suri",
    ]) {
        let path = example(name);
        let checked = run_cli(&["check".into(), path.to_string_lossy().to_string()]);
        assert_eq!(checked.status, 0, "{name}: {}", checked.stderr);
        assert_eq!(checked.stdout, "OK\n", "{name}");

        let source = fs::read_to_string(&path).unwrap();
        let compiled = compiler.compile(&source, None)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        assert!(compiled.ast.to_json().is_object(), "{name}: missing AST");
    }
}

#[test]
fn web_examples_build_real_html_and_preserve_visible_behavior() {
    let compiler = Compiler::new();
    for name in WEB_EXAMPLES {
        let source = fs::read_to_string(example(name)).unwrap();
        let result = compiler.compile(&source, None).unwrap();
        assert_eq!(result.backend.as_deref(), Some("html"), "{name}");
        let html = result.output.unwrap();
        assert!(html.starts_with("<!doctype html>"), "{name}: {html}");
        assert!(html.contains("</html>"), "{name}: {html}");
    }
    let expected = [
        ("control_flow.suri", ">Buy</button>"),
        ("context.suri", "color: blue"),
        ("shop.suri", "color: red"),
        ("variables.suri", "color: blue"),
        ("otherwise_if.suri", ">Passed</button>"),
        ("unless_unicode.suri", ">Passed</button>"),
    ];
    for (name, evidence) in expected {
        let source = fs::read_to_string(example(name)).unwrap();
        let html = compiler.compile(&source, None).unwrap().output.unwrap();
        assert!(html.contains(evidence), "{name}: missing {evidence}");
    }
}

#[test]
fn schema_example_produces_parseable_json_schema() {
    let source = fs::read_to_string(example("pet_schema.suri")).unwrap();
    let result = Compiler::new().compile(&source, None).unwrap();
    assert_eq!(result.backend.as_deref(), Some("json-schema"));
    let document: serde_json::Value = serde_json::from_str(&result.output.unwrap()).unwrap();
    assert!(document["properties"]["Name"].is_object());
    assert!(document["properties"]["Age"].is_object());
}

#[test]
fn console_example_runs_only_with_explicit_approval() {
    let path = example("hello_console.suri");
    let source = path.to_string_lossy().to_string();
    let denied = run_cli(&["run".into(), source.clone()]);
    assert_eq!(denied.status, 65);
    assert!(denied.stderr.contains("S334"));

    let allowed = run_cli(&[
        "run".into(), "--allow".into(), "console_write".into(), source,
    ]);
    assert_eq!(allowed.status, 0, "{}", allowed.stderr);
    assert_eq!(allowed.stdout, "Hello from Semauri\nSemauri\n42\n");
}

#[test]
fn filesystem_example_compiles_but_dry_run_has_no_effects() {
    let path = example("filesystem.suri");
    let source = fs::read_to_string(&path).unwrap();
    let compiler = Compiler::new();
    let result = compiler.compile(&source, None).unwrap();
    assert_eq!(result.backend.as_deref(), Some("posix-sh"));
    let script = result.output.unwrap();
    assert!(script.contains("mkdir -p build"));
    assert!(script.contains("rm -f build/old.tmp"));

    let temp = std::env::temp_dir().join(format!(
        "semauri-example-dryrun-{}", std::process::id()
    ));
    if temp.exists() {
        fs::remove_dir_all(&temp).unwrap();
    }
    fs::create_dir(&temp).unwrap();
    let args = [
        "run".into(),
        "--cwd".into(),
        temp.to_string_lossy().to_string(),
        "--dry-run".into(),
        path.to_string_lossy().to_string(),
    ];
    let dry_run = run_cli(&args);
    assert_eq!(dry_run.status, 0, "{}", dry_run.stderr);
    assert_eq!(dry_run.stdout, script);
    assert!(!Path::new(&temp.join("build")).exists());
    fs::remove_dir(&temp).unwrap();
}
