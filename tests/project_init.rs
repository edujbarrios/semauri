use semauri::run_cli;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn temp_parent() -> PathBuf {
    let parent = std::env::temp_dir().join(format!("semauri-init-{}-{}",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::create_dir_all(&parent).unwrap();
    parent
}

fn cli(command: &str, file: &std::path::Path) -> semauri::CliResult {
    run_cli(&[command.to_string(), file.to_string_lossy().into_owned()])
}

#[test]
fn init_creates_checked_and_runnable_multi_file_project() {
    let parent = temp_parent();
    let root = parent.join("hello");
    let result = cli("init", &root);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert!(result.stdout.contains("Created Semauri project"));
    for file in [".semauri-root", "main.suri", "lib/math.suri", "tests/math.suri", "README.md"] {
        assert!(root.join(file).is_file(), "Missing {file}");
    }

    let check = cli("check", &root.join("main.suri"));
    assert_eq!(check.status, 0, "{}", check.stderr);

    let denied = cli("run", &root.join("main.suri"));
    assert_eq!(denied.status, 65, "{}", denied.stderr);
    assert!(denied.stderr.contains("S334"), "{}", denied.stderr);

    let output = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        root.join("main.suri").to_string_lossy().into_owned()]);
    assert_eq!(output.status, 0, "{}", output.stderr);
    assert_eq!(output.stdout, "49\n");

    let test = run_cli(&["test".into(), root.join("tests").to_string_lossy().into_owned(),
        "--allow-console".into()]);
    assert_eq!(test.status, 0, "{}", test.stdout);
    assert!(test.stdout.contains("1 passed; 0 failed"), "{}", test.stdout);

    let effects = cli("effects", &root.join("main.suri"));
    assert_eq!(effects.status, 0, "{}", effects.stderr);
    assert!(effects.stdout.contains("console_write"));

    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn init_refuses_to_overwrite_existing_directories_or_files() {
    let parent = temp_parent();
    let root = parent.join("existing");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("keep.txt"), "untouched").unwrap();
    let denied = cli("init", &root);
    assert_eq!(denied.status, 64, "{}", denied.stderr);
    assert_eq!(fs::read_to_string(root.join("keep.txt")).unwrap(), "untouched");
    let empty = cli("init", &parent);
    assert_eq!(empty.status, 64, "{}", empty.stderr);
    assert!(!parent.join("main.suri").exists());
    let bad = run_cli(&["init".into()]);
    assert_eq!(bad.status, 64, "{}", bad.stderr);
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn marked_root_allows_nested_test_imports_but_not_escape() {
    let parent = temp_parent();
    let root = parent.join("project");
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::create_dir(root.join("lib")).unwrap();
    fs::write(root.join(".semauri-root"), "").unwrap();
    fs::write(root.join("lib").join("safe.suri"),
        "define function identity taking n as number to n.\n").unwrap();
    fs::write(root.join("tests").join("check.suri"),
        "import \"../lib/safe.suri\".\nassert invoke identity to 9 is equal to 9.\n").unwrap();
    let allowed = cli("check", &root.join("tests").join("check.suri"));
    assert_eq!(allowed.status, 0, "{}", allowed.stderr);

    // Remove the marker: the entry's own directory becomes the limit.
    fs::remove_file(root.join(".semauri-root")).unwrap();
    let denied = cli("check", &root.join("tests").join("check.suri"));
    assert_eq!(denied.status, 65, "{}", denied.stderr);
    assert!(denied.stderr.contains("S416"), "{}", denied.stderr);

    // Recreate it and prove even the root cannot import from outside.
    fs::write(root.join(".semauri-root"), "").unwrap();
    let outside = parent.join("outside.suri");
    fs::write(&outside, "print \"outside\".\n").unwrap();
    fs::write(root.join("tests").join("check.suri"),
        "import \"../../outside.suri\".\nassert true.\n").unwrap();
    let outside_error = cli("check", &root.join("tests").join("check.suri"));
    assert_eq!(outside_error.status, 65, "{}", outside_error.stderr);
    assert!(outside_error.stderr.contains("S416"), "{}", outside_error.stderr);
    fs::remove_dir_all(parent).unwrap();
}
