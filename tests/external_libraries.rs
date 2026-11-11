// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
//! Offline, explicitly declared third-party Semauri source libraries.
use semauri::run_cli;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn fixture() -> (PathBuf, PathBuf, PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "semauri-external-libs-{}-{}", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
    ));
    let project = root.join("project");
    let external = root.join("third-party").join("math-kit");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&external).unwrap();
    fs::write(project.join(".semauri-root"), "").unwrap();
    write(&external, ".semauri-library.json", r#"{
  "schema": 1, "name": "math_kit", "version": "1.2.3", "entry": "src/main.suri"
}"#);
    write(&external, "src/main.suri", "import \"internal.suri\".\n\
        define function triple taking n as number to (n times 3).\n");
    write(&external, "src/internal.suri",
        "define function square taking n as number to (n times n).\n");
    manifest(&project, json!({
        "schema": 1,
        "libraries": [
            {"name":"math_kit","version":"1.2.3","path":"../third-party/math-kit"}
        ]
    }));
    (root, project, external)
}

fn write(root: &Path, relative: &str, contents: &str) -> PathBuf {
    let target = root.join(relative);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, contents).unwrap();
    target
}

fn manifest(project: &Path, document: Value) {
    fs::write(project.join(".semauri-libraries.json"),
        serde_json::to_string_pretty(&document).unwrap()).unwrap();
}

fn call(command: &str, file: &Path) -> semauri::CliResult {
    run_cli(&[command.to_string(), file.to_string_lossy().into_owned()])
}

#[test]
fn imports_external_and_nested_libraries_from_outside_project() {
    let (root, project, _) = fixture();
    let source = write(&project, "main.suri",
        "IMPORT \"library:math_kit\".\n\
         assert invoke square to 7 is equal to 49.\n\
         print invoke triple to 5.\n");
    let check = call("check", &source);
    assert_eq!(check.status, 0, "{}", check.stderr);
    let result = run_cli(&[
        "run".into(), "--allow".into(), "console_write".into(),
        source.to_string_lossy().into_owned()
    ]);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "15\n");
    let symbols = call("hir", &source);
    assert_eq!(symbols.status, 0, "{}", symbols.stderr);
    assert!(symbols.stdout.contains("triple"), "{}", symbols.stdout);
    assert!(symbols.stdout.contains("square"), "{}", symbols.stdout);
    let formatted = call("fmt", &source);
    assert_eq!(formatted.status, 0, "{}", formatted.stderr);
    assert!(formatted.stdout.starts_with("import \"library:math_kit\"."));
    let inspected = call("libraries", &source);
    assert_eq!(inspected.status, 0, "{}", inspected.stderr);
    let libraries: Value = serde_json::from_str(&inspected.stdout).unwrap();
    assert_eq!(libraries["libraries"][0]["name"], "math_kit");
    assert_eq!(libraries["libraries"][0]["version"], "1.2.3");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn external_libraries_work_with_project_tests_and_verification() {
    let (root, project, _) = fixture();
    write(&project, "main.suri",
        "import \"library:math_kit\".\nprint invoke square to 4.\n");
    let suite = write(&project, "tests/library.suri",
        "import \"library:math_kit\".\nassert invoke triple to 3 is equal to 9.\n");
    let tests = run_cli(&[
        "test".into(), suite.to_string_lossy().into_owned()
    ]);
    assert_eq!(tests.status, 0, "{}", tests.stdout);
    let verify = run_cli(&[
        "verify".into(), project.to_string_lossy().into_owned()
    ]);
    assert_eq!(verify.status, 0, "{}", verify.stdout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn imported_effects_always_require_explicit_permissions() {
    let (root, project, external) = fixture();
    write(&external, "src/internal.suri",
        "print \"third-party effect\".\n\
         define function square taking n as number to (n times n).\n");
    let source = write(&project, "main.suri",
        "import \"library:math_kit\".\nassert true.\n");
    let effects = call("effects", &source);
    assert_eq!(effects.status, 0, "{}", effects.stderr);
    assert!(effects.stdout.contains("console_write"), "{}", effects.stdout);
    let denied = call("run", &source);
    assert_eq!(denied.status, 65, "{}", denied.stderr);
    assert!(denied.stderr.contains("S334"), "{}", denied.stderr);
    let allowed = run_cli(&[
        "run".into(), "--allow".into(), "console_write".into(),
        source.to_string_lossy().into_owned()
    ]);
    assert_eq!(allowed.status, 0, "{}", allowed.stderr);
    assert_eq!(allowed.stdout, "third-party effect\n");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn undeclared_library_and_version_mismatch_are_rejected() {
    let (root, project, external) = fixture();
    let source = write(&project, "main.suri", "import \"library:missing\".\nprint 1.\n");
    let undeclared = call("check", &source);
    assert_eq!(undeclared.status, 65, "{}", undeclared.stderr);
    assert!(undeclared.stderr.contains("S417"), "{}", undeclared.stderr);
    write(&project, "main.suri", "import \"library:math_kit\".\nprint 1.\n");
    write(&external, ".semauri-library.json", r#"{
      "schema": 1, "name": "math_kit", "version": "9.9.9", "entry": "src/main.suri"
    }"#);
    let mismatch = call("check", &source);
    assert_eq!(mismatch.status, 65, "{}", mismatch.stderr);
    assert!(mismatch.stderr.contains("S417"), "{}", mismatch.stderr);
    assert!(mismatch.stderr.contains("version"), "{}", mismatch.stderr);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn source_imports_are_confined_to_their_own_library_roots() {
    let (root, project, external) = fixture();
    let source = write(&project, "main.suri",
        "import \"library:math_kit\".\nassert true.\n");
    let outside = write(&root, "third-party/outside.suri", "print 1.\n");
    write(&external, "src/main.suri",
        "import \"../../outside.suri\".\n\
         define function triple taking n as number to (n times 3).\n");
    let escape = call("check", &source);
    assert_eq!(escape.status, 65, "{}", escape.stderr);
    assert!(escape.stderr.contains("S416"), "{}", escape.stderr);
    assert!(escape.stderr.contains("escapes"), "{}", escape.stderr);

    // The old project boundary is still enforced; a library declaration
    // does not implicitly authorize relative imports outside the project.
    write(&project, "main.suri",
        "import \"../third-party/outside.suri\".\nassert true.\n");
    let project_escape = call("check", &source);
    assert_eq!(project_escape.status, 65, "{}", project_escape.stderr);
    assert!(project_escape.stderr.contains("S416"), "{}", project_escape.stderr);
    assert!(outside.exists());
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn library_symlink_escape_is_rejected() {
    use std::os::unix::fs::symlink;
    let (root, project, external) = fixture();
    write(&root, "third-party/outside.suri", "print 1.\n");
    symlink(root.join("third-party/outside.suri"),
        external.join("src").join("escape.suri")).unwrap();
    write(&external, "src/main.suri",
        "import \"escape.suri\".\n\
         define function triple taking n as number to (n times 3).\n");
    let source = write(&project, "main.suri",
        "import \"library:math_kit\".\nassert true.\n");
    let result = call("check", &source);
    assert_eq!(result.status, 65, "{}", result.stderr);
    assert!(result.stderr.contains("S416"), "{}", result.stderr);
    assert!(result.stderr.contains("escapes"), "{}", result.stderr);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn catalog_rejects_malformed_metadata_duplicates_and_absolute_paths() {
    let (root, project, _) = fixture();
    let source = write(&project, "main.suri", "print 1.\n");
    for document in [
        json!({"schema":2,"libraries":[]}),
        json!({"schema":1,"libraries":[
            {"name":"math_kit","version":"1.2.3","path":"../third-party/math-kit"},
            {"name":"math_kit","version":"1.2.3","path":"../third-party/math-kit"}
        ]}),
        json!({"schema":1,"libraries":[
            {"name":"bad/name","version":"1.2.3","path":"../third-party/math-kit"}
        ]}),
        json!({"schema":1,"libraries":[
            {"name":"math_kit","version":"v1","path":"../third-party/math-kit"}
        ]}),
        json!({"schema":1,"libraries":[
            {"name":"math_kit","version":"1.2.3","path":root.to_string_lossy()}
        ]}),
    ] {
        manifest(&project, document);
        let invalid = call("check", &source);
        assert_eq!(invalid.status, 65, "{}", invalid.stderr);
        assert!(invalid.stderr.contains("S417"), "{}", invalid.stderr);
    }
    fs::write(project.join(".semauri-libraries.json"), "{ invalid").unwrap();
    let malformed = call("libraries", &project);
    assert_eq!(malformed.status, 65, "{}", malformed.stderr);
    assert!(malformed.stderr.contains("S417"), "{}", malformed.stderr);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn supports_transitive_declared_dependencies_and_rejects_cycles() {
    let (root, project, external) = fixture();
    let helper = root.join("third-party/helper");
    write(&helper, ".semauri-library.json", r#"{
        "schema":1,"name":"helper","version":"2.0.0","entry":"lib.suri"
    }"#);
    write(&helper, "lib.suri",
        "define function shift taking n as number to (n plus 5).\n");
    manifest(&project, json!({"schema":1,"libraries":[
        {"name":"math_kit","version":"1.2.3","path":"../third-party/math-kit"},
        {"name":"helper","version":"2.0.0","path":"../third-party/helper"}
    ]}));
    write(&external, "src/main.suri",
        "import \"library:helper\".\n\
         define function triple taking n as number to (n times 3).\n");
    let source = write(&project, "main.suri",
        "import \"library:math_kit\".\n\
         import \"library:helper\".\n\
         print invoke shift to 7.\n");
    let ok = run_cli(&[
        "run".into(), "--allow".into(), "console_write".into(),
        source.to_string_lossy().into_owned()
    ]);
    assert_eq!(ok.status, 0, "{}", ok.stderr);
    assert_eq!(ok.stdout, "12\n");
    write(&helper, "lib.suri",
        "import \"library:math_kit\".\n\
         define function shift taking n as number to (n plus 5).\n");
    let cycle = call("check", &source);
    assert_eq!(cycle.status, 65, "{}", cycle.stderr);
    assert!(cycle.stderr.contains("Circular"), "{}", cycle.stderr);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn semantic_errors_in_external_files_keep_the_original_file_location() {
    let (root, project, external) = fixture();
    write(&external, "src/internal.suri",
        "# External source.\n\
         define function broken taking n as number to (n plus \"not a number\").\n");
    let source = write(&project, "main.suri",
        "import \"library:math_kit\".\nassert true.\n");
    let result = call("check", &source);
    assert_eq!(result.status, 65, "{}", result.stderr);
    assert!(result.stderr.contains("internal.suri:"), "{}", result.stderr);
    assert!(result.stderr.contains("at 2:"), "{}", result.stderr);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn init_creates_a_valid_empty_library_catalog() {
    let root = std::env::temp_dir().join(format!("semauri-lib-init-{}-{}",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    let initialized = call("init", &root);
    assert_eq!(initialized.status, 0, "{}", initialized.stderr);
    let catalog: Value = serde_json::from_str(
        &fs::read_to_string(root.join(".semauri-libraries.json")).unwrap()
    ).unwrap();
    assert_eq!(catalog["schema"], 1);
    assert_eq!(catalog["libraries"].as_array().unwrap().len(), 0);
    let inspected = call("libraries", &root);
    assert_eq!(inspected.status, 0, "{}", inspected.stderr);
    fs::remove_dir_all(root).unwrap();
}
