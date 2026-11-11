// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
//! Aliased imports keep function definitions and calls isolated.
use semauri::run_cli;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn fixture() -> (PathBuf, PathBuf) {
    let root = std::env::temp_dir().join(format!("semauri-ns-{}-{}",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    let project = root.join("app");
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join(".semauri-root"), "").unwrap();
    library(&root, "math_kit", "define function shared taking n as number to (n times 2).\n");
    library(&root, "other_kit", "define function shared taking n as number to (n plus 10).\n");
    fs::write(project.join(".semauri-libraries.json"), json!({
        "schema":1,"libraries":[
            {"name":"math_kit","version":"1.0.0","path":"../math_kit"},
            {"name":"other_kit","version":"1.0.0","path":"../other_kit"}
        ]
    }).to_string()).unwrap();
    (root, project)
}

fn library(root: &Path, name: &str, source: &str) {
    let folder = root.join(name);
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join(".semauri-library.json"),json!({
        "schema":1,"name":name,"version":"1.0.0","entry":"lib.suri"
    }).to_string()).unwrap();
    fs::write(folder.join("lib.suri"),source).unwrap();
}
fn put(root: &Path, name: &str, text: &str) -> PathBuf {
    let file = root.join(name);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file,text).unwrap();
    file
}
fn call(command: &str, file: &Path) -> semauri::CliResult {
    run_cli(&[command.into(),file.to_string_lossy().into_owned()])
}
fn run(file: &Path) -> semauri::CliResult {
    run_cli(&["run".into(),"--allow".into(),"console_write".into(),
        file.to_string_lossy().into_owned()])
}

#[test]
fn colliding_external_functions_are_isolated_by_alias() {
    let (root, project) = fixture();
    let app = put(&project,"main.suri",
        "import \"library:math_kit\" as math.\n\
         import \"library:other_kit\" as other.\n\
         assert (invoke math.shared to 5) is equal to 10.\n\
         print invoke other.shared to 5.\n");
    for command in ["check","hir","effects"] {
        let result = call(command,&app);
        assert_eq!(result.status,0,"{command}: {}",result.stderr);
    }
    let result = run(&app);
    assert_eq!(result.status,0,"{}",result.stderr);
    assert_eq!(result.stdout,"15\n");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn internal_unqualified_function_calls_resolve_inside_each_namespace() {
    let (root,project) = fixture();
    library(&root,"math_kit",
        "define function shared taking n as number to (n times 2).\n\
         define function combined taking n as number to ((invoke shared to n) plus 1).\n");
    library(&root,"other_kit",
        "define function shared taking n as number to (n plus 10).\n\
         define function combined taking n as number to ((invoke shared to n) plus 2).\n");
    let app = put(&project,"main.suri",
        "import \"library:math_kit\" as alpha.\n\
         import \"library:other_kit\" as beta.\n\
         assert (invoke alpha.combined to 4) is equal to 9.\n\
         print invoke beta.combined to 4.\n");
    let result = run(&app);
    assert_eq!(result.status,0,"{}",result.stderr);
    assert_eq!(result.stdout,"16\n");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn nested_and_local_aliases_work_together() {
    let (root,project) = fixture();
    put(&project,"local.suri","define function shared taking n as number to (n times 7).\n");
    put(&root.join("math_kit"),"helper.suri",
        "define function square taking n as number to (n times n).\n");
    library(&root,"math_kit",
        "import \"helper.suri\" as helper.\n\
         define function fourth taking n as number to (invoke helper.square to (invoke helper.square to n)).\n");
    let app = put(&project,"main.suri",
        "import \"local.suri\" as local.\n\
         import \"library:math_kit\" as math.\n\
         assert (invoke local.shared to 2) is equal to 14.\n\
         print invoke math.fourth to 2.\n");
    let result = run(&app);
    assert_eq!(result.status,0,"{}",result.stderr);
    assert_eq!(result.stdout,"16\n");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn same_package_can_be_imported_twice_with_distinct_aliases() {
    let (root,project) = fixture();
    let app = put(&project,"main.suri",
        "import \"library:math_kit\" as primary.\n\
         import \"library:math_kit\" as second.\n\
         assert (invoke primary.shared to 4) is equal to 8.\n\
         print invoke second.shared to 8.\n");
    let result = run(&app);
    assert_eq!(result.status,0,"{}",result.stderr);
    assert_eq!(result.stdout,"16\n");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn namespaced_imports_deny_unqualified_lookup_and_duplicate_aliases() {
    let (root,project) = fixture();
    for (body,code) in [
        ("import \"library:math_kit\" as math.\nprint invoke shared to 4.\n","S369"),
        ("import \"library:math_kit\" as math.\nimport \"library:other_kit\" as math.\nprint 1.\n","S416"),
        ("import \"library:math_kit\" as math.\nprint invoke other.shared to 1.\n","S416"),
        ("import \"library:math_kit\" as for.\nprint 1.\n","S416"),
        ("import \"library:math_kit\" as prompt.\nprint 1.\n","S416"),
        ("import \"library:math_kit\" as invalid/path.\nprint 1.\n","S416"),
    ] {
        let app = put(&project,"main.suri",body);
        let result = call("check",&app);
        assert_eq!(result.status,65,"{body}: {}",result.stderr);
        assert!(result.stderr.contains(code),"{body}: {}",result.stderr);
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn scoped_rewriting_does_not_change_strings_or_comments() {
    let (root,project) = fixture();
    library(&root,"math_kit",
        "# invoke fake to 1.\n\
         define function shared taking n as string to n.\n");
    let app = put(&project,"main.suri",
        "import \"library:math_kit\" as math.\n\
         print invoke math.shared to \"invoke fake to 3\".\n");
    let result = run(&app);
    assert_eq!(result.status,0,"{}",result.stderr);
    assert_eq!(result.stdout,"invoke fake to 3\n");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn imported_top_level_effects_remain_permission_checked() {
    let (root,project) = fixture();
    library(&root,"math_kit",
        "print \"external effect\".\n\
         define function shared taking n as number to (n plus 1).\n");
    let app = put(&project,"main.suri",
        "import \"library:math_kit\" as math.\nassert true.\n");
    let denied = call("run",&app);
    assert_eq!(denied.status,65,"{}",denied.stderr);
    assert!(denied.stderr.contains("S334"),"{}",denied.stderr);
    let result = run(&app);
    assert_eq!(result.status,0,"{}",result.stderr);
    assert_eq!(result.stdout,"external effect\n");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn unaliased_imports_keep_their_original_meaning() {
    let (root,project) = fixture();
    let app = put(&project,"main.suri",
        "import \"library:math_kit\".\nprint invoke shared to 5.\n");
    let result = run(&app);
    assert_eq!(result.status,0,"{}",result.stderr);
    assert_eq!(result.stdout,"10\n");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn formatter_understands_import_aliases_and_qualified_calls() {
    let (root,project) = fixture();
    let app = put(&project,"main.suri",
        "IMPORT \"library:math_kit\" AS math.\nPRINT INVOKE math.shared TO 5.\n");
    let formatted = call("fmt",&app);
    assert_eq!(formatted.status,0,"{}",formatted.stderr);
    assert!(formatted.stdout.contains("import \"library:math_kit\" as math."),
        "{}",formatted.stdout);
    assert!(formatted.stdout.contains("print invoke math.shared to 5."),
        "{}",formatted.stdout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn semantic_errors_in_aliased_files_keep_original_locations() {
    let (root,project) = fixture();
    let invalid = "define function shared taking n as number to (n plus \"bad\").\n";
    library(&root,"math_kit",invalid);
    let app = put(&project,"main.suri",
        "import \"library:math_kit\" as math.\nassert true.\n");
    let result = run_cli(&["check".into(),"--json".into(),
        app.to_string_lossy().into_owned()]);
    assert_eq!(result.status,65,"{}",result.stdout);
    let value: serde_json::Value = serde_json::from_str(&result.stdout).unwrap();
    let diagnostic = &value["diagnostics"][0];
    assert!(diagnostic["file"].as_str().unwrap().ends_with("lib.suri"),
        "{value}");
    assert_eq!(diagnostic["span"]["start"]["line"],1,"{value}");
    let column = diagnostic["span"]["start"]["column"].as_u64().unwrap();
    assert!(column > 0 && column <= invalid.trim_end().chars().count() as u64,
        "Column must refer to the original source, not the generated identifier: {value}");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn generated_namespace_prefix_is_reserved_in_source_files() {
    let (root,project) = fixture();
    for name in ["semauri_ns_1__shared", "SEMAURI_NS_1__SHARED"] {
        let app = put(&project,"main.suri",
            &format!("define function {name} taking n as number to n.\n\
                print invoke {name} to 2.\n"));
        let result = call("check",&app);
        assert_eq!(result.status,65,"{}",result.stderr);
        assert!(result.stderr.contains("S416"),"{}",result.stderr);
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn aliased_libraries_can_return_typed_records_with_colliding_function_names() {
    let (root, project) = fixture();
    library(&root, "math_kit",
        "define function profile taking n as number to record of (\"score\": n, \"label\": \"math\").\n");
    library(&root, "other_kit",
        "define function profile taking n as number to record of (\"score\": (n plus 10), \"label\": \"other\").\n");
    let app = put(&project, "main.suri",
        "import \"library:math_kit\" as math.\n\
         import \"library:other_kit\" as other.\n\
         let math_result be invoke math.profile to 5.\n\
         let other_result be invoke other.profile to 5.\n\
         assert (field \"score\" of math_result) is equal to 5.\n\
         print field \"label\" of other_result.\n");
    let result = run(&app);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "other\n");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn aliased_libraries_can_return_typed_dictionaries() {
    let (root, project) = fixture();
    library(&root, "math_kit",
        "define function settings taking n as number to dictionary of (\"primary\": n, \"double\": (n times 2)).\n");
    library(&root, "other_kit",
        "define function settings taking n as number to dictionary of (\"primary\": (n plus 3), \"double\": (n plus 4)).\n");
    let app = put(&project, "main.suri",
        "import \"library:math_kit\" as math.\n\
         import \"library:other_kit\" as other.\n\
         let left be invoke math.settings to 5.\n\
         let right be invoke other.settings to 5.\n\
         assert (entry \"primary\" of left) is equal to 5.\n\
         assert (entry \"primary\" of right) is equal to 8.\n\
         print json of left.\n");
    let result = run(&app);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "{\"double\":10,\"primary\":5}\n");
    fs::remove_dir_all(root).unwrap();
}
