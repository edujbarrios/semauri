use semauri::{run_cli, Compiler};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn project() -> PathBuf {
    let root = std::env::temp_dir().join(format!("semauri-modules-{}-{}",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::create_dir_all(&root).unwrap();
    root
}

fn put(root: &Path, name: &str, contents: &str) -> PathBuf {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, contents).unwrap();
    path
}

fn cli(command: &str, path: &Path) -> semauri::CliResult {
    run_cli(&[command.to_string(), path.to_string_lossy().into_owned()])
}

#[test]
fn nested_deduplicated_imports_work_in_check_run_and_inspection() {
    let root = project();
    put(&root, "lib/base.suri", "define function double taking n as number to (n times 2).\n");
    put(&root, "lib/maths.suri",
        "import \"base.suri\".\ndefine function square taking n as number to (n times n).\n");
    let main = put(&root, "main.suri",
        "import \"lib/maths.suri\".\nimport \"lib/base.suri\".\n\
         assert invoke double to 3 is equal to 6.\n\
         print invoke square to 7.\n");
    let check = cli("check", &main);
    assert_eq!(check.status, 0, "{}", check.stderr);
    let hir = cli("hir", &main);
    assert_eq!(hir.status, 0, "{}", hir.stderr);
    assert!(hir.stdout.contains("square"), "{}", hir.stdout);
    let effects = cli("effects", &main);
    assert_eq!(effects.status, 0, "{}", effects.stderr);
    assert!(effects.stdout.contains("console_write"), "{}", effects.stdout);
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        main.to_string_lossy().into_owned()]);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "49\n");
    let tests = run_cli(&["test".into(), main.to_string_lossy().into_owned(),
        "--allow-console".into()]);
    assert_eq!(tests.status, 0, "{}", tests.stdout);
    assert!(tests.stdout.contains("1 passed; 0 failed"), "{}", tests.stdout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn circular_and_late_imports_are_rejected() {
    let root = project();
    let a = put(&root, "a.suri", "import \"b.suri\".\nprint 1.\n");
    put(&root, "b.suri", "import \"a.suri\".\nprint 2.\n");
    let bad = cli("check", &a);
    assert_eq!(bad.status, 65, "{}", bad.stderr);
    assert!(bad.stderr.contains("S416"), "{}", bad.stderr);
    assert!(bad.stderr.contains("Circular"), "{}", bad.stderr);
    put(&root, "b.suri", "print 1.\nimport \"a.suri\".\n");
    let late = cli("check", &a);
    assert_eq!(late.status, 65, "{}", late.stderr);
    assert!(late.stderr.contains("S416"), "{}", late.stderr);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn malformed_missing_absolute_and_outside_paths_are_rejected() {
    let root = project();
    for contents in [
        "import \"missing.suri\".\nprint 1.\n",
        "import \"/tmp/not-local.suri\".\nprint 1.\n",
        "import \"outside.txt\".\nprint 1.\n",
        "import nothing.\nprint 1.\n",
        "import \"x.suri\"\nprint 1.\n",
    ] {
        let main = put(&root, "main.suri", contents);
        let result = cli("check", &main);
        assert_eq!(result.status, 65, "{contents}: {}", result.stderr);
        assert!(result.stderr.contains("S416"), "{contents}: {}", result.stderr);
    }
    let parent = root.parent().unwrap();
    let outsider = parent.join(format!("outside-{}.suri", NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&outsider, "print 1.\n").unwrap();
    let main = put(&root, "main.suri",
        &format!("import \"../{}\".\n", outsider.file_name().unwrap().to_string_lossy()));
    let result = cli("check", &main);
    assert_eq!(result.status, 65, "{}", result.stderr);
    assert!(result.stderr.contains("S416"), "{}", result.stderr);
    fs::remove_file(outsider).unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn symlink_escape_is_rejected() {
    use std::os::unix::fs::symlink;
    let root = project();
    let outside = root.parent().unwrap().join(format!("external-{}.suri",
        NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&outside, "print 1.\n").unwrap();
    symlink(&outside, root.join("linked.suri")).unwrap();
    let main = put(&root, "main.suri", "import \"linked.suri\".\n");
    let result = cli("check", &main);
    assert_eq!(result.status, 65, "{}", result.stderr);
    assert!(result.stderr.contains("S416"), "{}", result.stderr);
    fs::remove_file(outside).unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn imported_errors_report_the_original_file_and_line() {
    let root = project();
    put(&root, "lib/broken.suri",
        "# A broken imported function.\ndefine function broken taking n as number to (n plus \"bad\").\n");
    let main = put(&root, "main.suri", "import \"lib/broken.suri\".\nprint 1.\n");
    let checked = cli("check", &main);
    assert_eq!(checked.status, 65, "{}", checked.stderr);
    assert!(checked.stderr.contains("broken.suri:"), "{}", checked.stderr);
    assert!(checked.stderr.contains("at 2:"), "{}", checked.stderr);
    assert!(checked.stderr.contains("n plus \"bad\""), "{}", checked.stderr);
    let json = run_cli(&["check".into(), "--json".into(),
        main.to_string_lossy().into_owned()]);
    assert_eq!(json.status, 65, "{}", json.stdout);
    let data: serde_json::Value = serde_json::from_str(&json.stdout).unwrap();
    assert_eq!(data["diagnostics"][0]["span"]["start"]["line"], 2);
    assert!(data["diagnostics"][0]["file"].as_str().unwrap().ends_with("broken.suri"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn import_code_must_obey_normal_capability_policy() {
    let root = project();
    put(&root, "lib.suri", "print \"This import requires console authorization\".\n");
    let main = put(&root, "main.suri",
        "import \"lib.suri\".\nassert true.\n");
    let denied = cli("run", &main);
    assert_eq!(denied.status, 65, "{}", denied.stderr);
    assert!(denied.stderr.contains("S334"), "{}", denied.stderr);
    let allowed = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        main.to_string_lossy().into_owned()]);
    assert_eq!(allowed.status, 0, "{}", allowed.stderr);
    assert!(allowed.stdout.contains("requires console authorization"), "{}", allowed.stdout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn formatter_and_in_memory_api_behave_explicitly() {
    let compiler = Compiler::new();
    assert_eq!(compiler.format_source("IMPORT \"lib/math.suri\".\n").unwrap(),
        "import \"lib/math.suri\".\n");
    let error = compiler.hir("import \"math.suri\".").unwrap_err();
    assert_eq!(error.code, "S416");
    let root = project();
    put(&root, "lib.suri", "define function identity taking n as number to n.\n");
    let main = put(&root, "main.suri", "IMPORT \"lib.suri\".\nassert invoke identity to 4 is equal to 4.\n");
    let check = cli("check", &main);
    assert_eq!(check.status, 0, "{}", check.stderr);
    let formatted = cli("fmt", &main);
    assert_eq!(formatted.status, 0, "{}", formatted.stderr);
    assert!(formatted.stdout.starts_with("import \"lib.suri\"."), "{}", formatted.stdout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn oversized_source_graph_is_refused() {
    let root = project();
    let large = " ".repeat(4 * 1024 * 1024 + 1);
    let main = put(&root, "main.suri", &large);
    let checked = cli("check", &main);
    assert_eq!(checked.status, 65, "{}", checked.stderr);
    assert!(checked.stderr.contains("S416"), "{}", checked.stderr);
    fs::remove_dir_all(root).unwrap();
}
