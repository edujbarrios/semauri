use semauri::{run_cli, Compiler};
use std::{fs, sync::atomic::{AtomicU64, Ordering}};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-square-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(),"--allow".into(),"console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn square_is_precise_for_integers_decimals_and_composition() {
    let r=run(r#"print square of (minus 5).
print square of 0.
print square of 2.5.
print square of 3037000499.
let items be a list of minus 3, 0, 4.
print map every x in items to (square of x).
define function squared taking x as number to (square of x).
print invoke squared to 6."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"25\n0\n6.25\n9223372030926249001\n[9, 0, 16]\n36\n");
}
#[test]
fn checked_overflow_reports_source_position() {
    let r=run("print square of 3037000500.");
    assert_eq!(r.status,65,"{}",r.stderr);
    assert!(r.stderr.contains("S422"),"{}",r.stderr);
}
#[test]
fn invalid_types_are_rejected_statically() {
    for src in ["print square of true.","print square of \"x\".","print square of a list of 1, 2."] {
        let e=Compiler::new().hir(src).unwrap_err();
        assert_eq!(e.code,"S421","{src}: {e}");
        assert!(e.span.is_some());
    }
}
#[test]
fn remains_a_contextual_identifier_and_formats() {
    let c=Compiler::new();
    let r=run("let square be 7. print square. print square of 3.");
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"7\n9\n");
    assert_eq!(c.format_source("PRINT SQUARE OF 2.").unwrap(),"print square of 2.");
    assert_eq!(c.format_source("LET Square BE 5. PRINT Square.").unwrap(),"let Square be 5. print Square.");
}
#[test]
fn optimized_hir_and_effects_preserve_behavior() {
    let c=Compiler::new();
    let source="print square of 9.";
    assert!(c.hir(source).unwrap().to_json().to_string().contains("square"));
    assert!(c.optimized_hir(source).is_ok());
    assert_eq!(c.effect_analysis(source).unwrap().effects(),vec!["console_write".to_owned()]);
}
