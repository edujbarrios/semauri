use semauri::{run_cli, Compiler};
use std::{fs, sync::atomic::{AtomicU64, Ordering}};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!("semauri-digits-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}
#[test]
fn counts_digits_of_zero_signed_integers_and_minimum() {
    let r = run(r#"assert (digits of 0) is equal to 1.
assert (digits of 8) is equal to 1.
assert (digits of 10) is equal to 2.
assert (digits of (minus 123)) is equal to 3.
assert (digits of 9007199254740993) is equal to 16.
assert (digits of 9223372036854775807) is equal to 19.
let boundary_value be (minus 9223372036854775807) minus 1.
assert (digits of boundary_value) is equal to 19.
assert (digits of 100.0) is equal to 3.
print digits of 456."#);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "3\n");
}
#[test]
fn rejects_fractional_or_out_of_range_numeric_inputs() {
    for src in [
        "print digits of 1.25.",
        "print digits of (minus 3.5).",
        "let too_large be parse number from \"9223372036854775808.0\". print digits of too_large.",
    ] {
        let r = run(src);
        assert_eq!(r.status, 65, "{src}: {}", r.stderr);
        assert!(r.stderr.contains("S442"), "{src}: {}", r.stderr);
    }
}
#[test]
fn rejects_non_numeric_operands_statically() {
    let compiler = Compiler::new();
    for source in ["print digits of true.", "print digits of \"hello\".",
                   "print digits of a list of 10, 20."] {
        let err = compiler.hir(source).unwrap_err();
        assert_eq!(err.code, "S441", "{source}: {err}");
        assert!(err.span.is_some());
    }
}
#[test]
fn works_with_lists_functions_and_contextual_identifiers() {
    let r = run(r#"define function width taking n as number to (digits of n).
let inputs be a list of 0, 10, 333, 10000.
let widths be map every n in inputs to (invoke width to n).
assert (widths at 0) is equal to 1.
assert (widths at 1) is equal to 2.
assert (widths at 2) is equal to 3.
assert (widths at 3) is equal to 5.
let digits be 7.
print digits."#);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "7\n");
    let compiler = Compiler::new();
    assert_eq!(compiler.format_source("PRINT DIGITS OF 100.").unwrap(), "print digits of 100.");
    assert!(compiler.hir("print digits of 20.").unwrap().to_json().to_string().contains("digits"));
    assert!(compiler.optimized_hir("print digits of 20.").is_ok());
    assert_eq!(compiler.effect_analysis("print digits of 20.").unwrap().effects(),
               vec!["console_write".to_owned()]);
}
