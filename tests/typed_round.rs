use semauri::{run_cli, Compiler};
use std::{fs, sync::atomic::{AtomicU64, Ordering}};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-nearest-{}-{}.suri",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
    fs::write(&path,source).unwrap();
    let r=run_cli(&["run".into(),"--allow".into(),"console_write".into(),path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();r
}
#[test]
fn rounds_half_values_away_from_zero_and_other_decimals() {
    let r=run(r#"print round of 2.5.
print round of (minus 2.5).
print round of 2.49.
print round of (minus 2.49).
print round of 0.0.
print round of 2.0."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"3\n-3\n2\n-2\n0\n2\n");
}
#[test]
fn retains_exact_i64_integers_and_avoids_float_saturation() {
    let r=run(r#"let large be 9007199254740993.
let smallest be (minus 9223372036854775807) minus 1.
assert (round of large) is equal to large.
assert (round of smallest) is equal to smallest.
let huge be parse number from "9223372036854775808.0".
assert (round of huge) is equal to huge."#);
    assert_eq!(r.status,0,"{}",r.stderr);
}
#[test]
fn type_errors_and_contextual_identifier() {
    let c=Compiler::new();
    for src in ["print round of true.","print round of \"x\".","print round of a list of 1, 2."] {
        let e=c.hir(src).unwrap_err();
        assert_eq!(e.code,"S427","{src}: {e}");
        assert!(e.span.is_some());
    }
    let r=run("let round be 8. print round. print round of 1.6.");
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"8\n2\n");
    assert_eq!(c.format_source("PRINT ROUND OF 2.5.").unwrap(),"print round of 2.5.");
    assert_eq!(c.format_source("LET Round BE 4. PRINT Round.").unwrap(),"let Round be 4. print Round.");
}
#[test]
fn works_in_functions_lists_optimizer_and_effect_analysis() {
    let r=run(r#"define function nearest taking n as number to (round of n).
let input be a list of minus 1.5, 0.5, 3.6.
print map every n in input to (invoke nearest to n)."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[-2, 1, 4]\n");
    let c=Compiler::new();
    let source="print round of 1.5.";
    assert!(c.hir(source).unwrap().to_json().to_string().contains("round"));
    assert!(c.optimized_hir(source).is_ok());
    assert_eq!(c.effect_analysis(source).unwrap().effects(),vec!["console_write".to_owned()]);
}
