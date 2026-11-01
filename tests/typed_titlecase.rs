use semauri::{run_cli, Compiler};
use std::{fs, sync::atomic::{AtomicU64, Ordering}};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!("semauri-titlecase-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}
#[test]
fn titlecases_words_and_preserves_unicode_whitespace() {
    let r = run(r#"assert (titlecase of "hELLO wORLD") is equal to "Hello World".
assert (titlecase of "ßETA ÉTÉ") is equal to "SSeta Été".
assert (titlecase of "  TWO  WORDS ") is equal to "  Two  Words ".
assert (titlecase of "a-b") is equal to "A-b".
assert (titlecase of "") is equal to "".
print titlecase of "hELLO wORLD"."#);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "Hello World\n");
}
#[test]
fn rejects_non_strings_and_does_not_reserve_titlecase_name() {
    let c = Compiler::new();
    for src in ["print titlecase of 42.", "print titlecase of false.",
                "print titlecase of a list of \"a\", \"b\"."] {
        let error = c.hir(src).unwrap_err();
        assert_eq!(error.code, "S351", "{src}: {error}");
        assert!(error.span.is_some());
    }
    let r = run("let titlecase be \"name\". print titlecase.");
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "name\n");
}
#[test]
fn functions_map_formatter_optimizer_and_effects_work() {
    let r = run(r#"define function heading taking word as string to (titlecase of word).
let words_list be a list of "hELLO", "éTÉ".
print map every word in words_list to (invoke heading to word)."#);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "[Hello, Été]\n");
    let c = Compiler::new();
    assert_eq!(c.format_source("PRINT TITLECASE OF \"hELLO\".").unwrap(),
               "print titlecase of \"hELLO\".");
    let src = "print titlecase of \"hELLO\".";
    assert!(c.hir(src).unwrap().to_json().to_string().contains("titlecase"));
    assert!(c.optimized_hir(src).is_ok());
    assert_eq!(c.effect_analysis(src).unwrap().effects(), vec!["console_write".to_owned()]);
}
