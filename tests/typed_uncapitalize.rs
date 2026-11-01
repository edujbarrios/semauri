use semauri::{run_cli, Compiler};
use std::{fs, sync::atomic::{AtomicU64, Ordering}};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!("semauri-uncapitalize-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}
#[test]
fn first_scalar_lowercases_and_tail_is_preserved() {
    let r = run(r#"assert (uncapitalize of "HELLO World") is equal to "hELLO World".
assert (uncapitalize of "ÉTÉ") is equal to "éTÉ".
assert (uncapitalize of "") is equal to "".
assert (uncapitalize of " World") is equal to " World".
assert (uncapitalize of "123ABC") is equal to "123ABC".
print uncapitalize of "HELLO"."#);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "hELLO\n");
}
#[test]
fn typed_errors_and_contextual_name_are_preserved() {
    let compiler = Compiler::new();
    for src in ["print uncapitalize of 42.", "print uncapitalize of true.",
                "print uncapitalize of a list of \"a\", \"b\"."] {
        let error = compiler.hir(src).unwrap_err();
        assert_eq!(error.code, "S351", "{src}: {error}");
        assert!(error.span.is_some());
    }
    let r = run("let uncapitalize be \"name\". print uncapitalize.");
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "name\n");
}
#[test]
fn function_map_formatter_and_optimizer_work() {
    let r = run(r#"define function lowerfirst taking word as string to (uncapitalize of word).
let words_list be a list of "ALPHA", "ÉTÉ".
print map every word in words_list to (invoke lowerfirst to word)."#);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "[aLPHA, éTÉ]\n");
    let c = Compiler::new();
    assert_eq!(c.format_source("PRINT UNCAPITALIZE OF \"HI\".").unwrap(),
               "print uncapitalize of \"HI\".");
    let src = "print uncapitalize of \"HI\".";
    assert!(c.hir(src).unwrap().to_json().to_string().contains("uncapitalize"));
    assert!(c.optimized_hir(src).is_ok());
    assert_eq!(c.effect_analysis(src).unwrap().effects(), vec!["console_write".to_owned()]);
}
