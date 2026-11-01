use semauri::{run_cli, Compiler};
use std::{fs, sync::atomic::{AtomicU64, Ordering}};
static NEXT: AtomicU64=AtomicU64::new(1);
fn run(source:&str)->semauri::CliResult {
    let path=std::env::temp_dir().join(format!("semauri-truncate-{}-{}.suri",
        std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
    fs::write(&path,source).unwrap();
    let r=run_cli(&["run".into(),"--allow".into(),"console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();r
}
#[test]
fn truncates_both_signs_toward_zero_including_subunit_values() {
    let r=run(r#"print truncate of 2.9.
print truncate of (minus 2.9).
print truncate of 0.5.
print truncate of (minus 0.5).
print truncate of 0.
print truncate of 3.0."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"2\n-2\n0\n0\n0\n3\n");
}
#[test]
fn maintains_precise_i64_and_avoids_saturation() {
    let r=run(r#"let big be 9007199254740993.
let smallest be (minus 9223372036854775807) minus 1.
assert (truncate of big) is equal to big.
assert (truncate of smallest) is equal to smallest.
let huge be parse number from "9223372036854775808.0".
assert (truncate of huge) is equal to huge."#);
    assert_eq!(r.status,0,"{}",r.stderr);
}
#[test]
fn preserves_identifiers_and_reports_type_errors() {
    let c=Compiler::new();
    for src in ["print truncate of true.","print truncate of \"x\".","print truncate of a list of 1, 2."] {
        let e=c.hir(src).unwrap_err();
        assert_eq!(e.code,"S429","{src}: {e}");
        assert!(e.span.is_some());
    }
    let r=run("let truncate be 7. print truncate. print truncate of (minus 2.8).");
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"7\n-2\n");
    assert_eq!(c.format_source("PRINT TRUNCATE OF 2.8.").unwrap(),"print truncate of 2.8.");
    assert_eq!(c.format_source("LET Truncate BE 5. PRINT Truncate.").unwrap(),"let Truncate be 5. print Truncate.");
}
#[test]
fn combines_with_functions_transforms_optimization_and_effects() {
    let r=run(r#"define function whole taking n as number to (truncate of n).
let samples be a list of minus 2.5, 0.4, 5.9.
print map every n in samples to (invoke whole to n)."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[-2, 0, 5]\n");
    let c=Compiler::new();
    let src="print truncate of 2.5.";
    assert!(c.hir(src).unwrap().to_json().to_string().contains("truncate"));
    assert!(c.optimized_hir(src).is_ok());
    assert_eq!(c.effect_analysis(src).unwrap().effects(),vec!["console_write".to_owned()]);
}
