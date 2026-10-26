use semauri::run_cli;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn new_project() -> (PathBuf, PathBuf) {
    let parent = std::env::temp_dir().join(format!("semauri-format-{}-{}",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::create_dir_all(&parent).unwrap();
    let root = parent.join("app");
    let init = run_cli(&["init".into(), root.to_string_lossy().into_owned()]);
    assert_eq!(init.status, 0, "{}", init.stderr);
    (parent, root)
}

fn fmt(path: &Path, flag: Option<&str>) -> semauri::CliResult {
    let mut args = vec!["fmt".to_string(), path.to_string_lossy().into_owned()];
    if let Some(flag) = flag { args.push(flag.to_string()); }
    run_cli(&args)
}

#[test]
fn formats_a_whole_project_and_enables_verification() {
    let (parent, root) = new_project();
    for (path, from, to) in [
        ("main.suri", "import", "IMPORT"),
        ("lib/math.suri", "define function", "DEFINE FUNCTION"),
        ("tests/math.suri", "assert", "ASSERT"),
    ] {
        let file = root.join(path);
        let source = fs::read_to_string(&file).unwrap();
        fs::write(&file, source.replace(from, to)).unwrap();
    }
    let check = fmt(&root, Some("--check"));
    assert_eq!(check.status, 1, "{}", check.stderr);
    assert!(check.stdout.contains("3 files checked; 3 need formatting"), "{}", check.stdout);
    assert!(check.stdout.contains("main.suri"));
    assert!(check.stdout.contains("lib/math.suri"));
    assert!(check.stdout.contains("tests/math.suri"));
    let repaired = fmt(&root, Some("--write"));
    assert_eq!(repaired.status, 0, "{}", repaired.stderr);
    assert!(repaired.stdout.contains("Formatted 3 of 3"), "{}", repaired.stdout);
    let verified = fmt(&root, Some("--check"));
    assert_eq!(verified.status, 0, "{}", verified.stdout);
    assert!(verified.stdout.contains("3 files checked; 0 need formatting"));
    let project = run_cli(&["verify".into(), root.to_string_lossy().into_owned()]);
    assert_eq!(project.status, 0, "{}", project.stdout);
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn failed_preflight_does_not_modify_any_source_file() {
    let (parent, root) = new_project();
    let main = root.join("main.suri");
    let modified = fs::read_to_string(&main).unwrap().replace("import", "IMPORT");
    fs::write(&main, &modified).unwrap();
    fs::write(root.join("bad.suri"), "print $.\n").unwrap();
    let failed = fmt(&root, Some("--write"));
    assert_eq!(failed.status, 65, "{}", failed.stderr);
    assert_eq!(fs::read_to_string(&main).unwrap(), modified);
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn directories_need_explicit_modes_single_files_remain_compatible() {
    let (parent, root) = new_project();
    let missing = fmt(&root, None);
    assert_eq!(missing.status, 64, "{}", missing.stderr);
    assert!(missing.stderr.contains("requires --check or --write"));
    let main = root.join("main.suri");
    let single = fmt(&main, None);
    assert_eq!(single.status, 0, "{}", single.stderr);
    assert!(single.stdout.starts_with("# Your first multi-file"));
    let checked = fmt(&main, Some("--check"));
    assert_eq!(checked.status, 0, "{}", checked.stderr);
    fs::remove_dir_all(parent).unwrap();
}

#[cfg(unix)]
#[test]
fn does_not_follow_symlinks_or_modify_excluded_cache_paths() {
    use std::os::unix::fs::symlink;
    let (parent, root) = new_project();
    let outside = parent.join("outside.suri");
    fs::write(&outside, "PRINT \"outside\".\n").unwrap();
    symlink(&outside, root.join("linked.suri")).unwrap();
    fs::create_dir(root.join("target")).unwrap();
    fs::write(root.join("target/generated.suri"), "PRINT \"cache\".\n").unwrap();
    let check = fmt(&root, Some("--check"));
    assert_eq!(check.status, 0, "{}", check.stdout);
    assert!(check.stdout.contains("3 files checked; 0 need formatting"), "{}", check.stdout);
    assert_eq!(fs::read_to_string(&outside).unwrap(), "PRINT \"outside\".\n");
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn reject_empty_source_directories() {
    let (parent, root) = new_project();
    let folder = root.join("empty");
    fs::create_dir(&folder).unwrap();
    let result = fmt(&folder, Some("--check"));
    assert_eq!(result.status, 64, "{}", result.stderr);
    assert!(result.stderr.contains("No .suri"));
    fs::remove_dir_all(parent).unwrap();
}
