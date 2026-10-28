use semauri::{run_cli, Compiler};
use std::{fs, sync::atomic::{AtomicU64, Ordering}};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-capitalize-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}
#[test]
fn capitalizes_first_scalar_without_changing_the_tail() {
    let r = run(r#"assert (capitalize of "hello WORLD") is equal to "Hello WORLD".
assert (capitalize of "élève") is equal to "Élève".
assert (capitalize of "ßeta") is equal to "SSeta".
assert (capitalize of "") is equal to "".
assert (capitalize of "123abc") is equal to "123abc".
assert (capitalize of " world") is equal to " world".
print capitalize of "hello"."#);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "Hello\n");
}
#[test]
fn rejects_non_text_with_existing_case_transformation_diagnostic() {
    let compiler = Compiler::new();
    for source in ["print capitalize of 42.", "print capitalize of true.",
                   "print capitalize of a list of \"a\", \"b\"."] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S351", "{source}: {error}");
        assert!(error.span.is_some());
    }
}
#[test]
fn contextual_identifiers_and_inspection_remain_supported() {
    let r = run(r#"define function up taking item as string to (capitalize of item).
let items be a list of "cat", "été".
print map every item in items to (invoke up to item).
let capitalize be "unchanged".
print capitalize."#);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "[\"Cat\", \"Été\"]\nunchanged\n");
    let compiler = Compiler::new();
    assert_eq!(compiler.format_source("PRINT CAPITALIZE OF \"hello\".").unwrap(),
               "print capitalize of \"hello\".");
    assert_eq!(compiler.format_source("LET Capitalize BE \"v\". PRINT Capitalize.").unwrap(),
               "let Capitalize be \"v\". print Capitalize.");
    let src = "print capitalize of \"test\".";
    assert!(compiler.hir(src).unwrap().to_json().to_string().contains("capitalize"));
    assert!(compiler.optimized_hir(src).is_ok());
    assert_eq!(compiler.effect_analysis(src).unwrap().effects(),
               vec!["console_write".to_owned()]);
}
