use semauri::{run_cli, Compiler};
use std::{fs, sync::atomic::{AtomicU64, Ordering}};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-swapcase-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}
#[test]
fn unicode_swapcase_changes_letter_case_without_mangling_other_text() {
    let r = run(r#"assert (swapcase of "HeLLo 123!?") is equal to "hEllO 123!?".
assert (swapcase of "ßéta") is equal to "SSÉTA".
assert (swapcase of "Été") is equal to "éTÉ".
assert (swapcase of "") is equal to "".
assert (swapcase of "123!?") is equal to "123!?".
print swapcase of "Hi THERE"."#);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "hI there\n");
}
#[test]
fn type_errors_and_contextual_name_are_preserved() {
    let compiler = Compiler::new();
    for src in ["print swapcase of 12.", "print swapcase of false.",
                "print swapcase of a list of \"X\", \"y\"."] {
        let error = compiler.hir(src).unwrap_err();
        assert_eq!(error.code, "S351", "{src}: {error}");
        assert!(error.span.is_some());
    }
    let r = run("let swapcase be \"name\". print swapcase.");
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "name\n");
}
#[test]
fn maps_functions_formatter_and_optimizer_use_identical_semantics() {
    let r = run(r#"define function invert taking value as string to (swapcase of value).
let words_list be a list of "Hi", "éTa".
print map every word in words_list to (invoke invert to word)."#);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "[hI, ÉtA]\n");
    let compiler = Compiler::new();
    assert_eq!(compiler.format_source("PRINT SWAPCASE OF \"A\".").unwrap(),
               "print swapcase of \"A\".");
    assert_eq!(compiler.format_source("LET Swapcase BE \"x\". PRINT Swapcase.").unwrap(),
               "let Swapcase be \"x\". print Swapcase.");
    let src = "print swapcase of \"A\".";
    assert!(compiler.hir(src).unwrap().to_json().to_string().contains("swapcase"));
    assert!(compiler.optimized_hir(src).is_ok());
    assert_eq!(compiler.effect_analysis(src).unwrap().effects(),
               vec!["console_write".to_owned()]);
}
