use semauri::{run_cli, Compiler};
use std::{fs, sync::atomic::{AtomicU64, Ordering}};
static NEXT: AtomicU64=AtomicU64::new(1);
fn run(source:&str)->semauri::CliResult {
    let path=std::env::temp_dir().join(format!("semauri-parity-{}-{}.suri",
        std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
    fs::write(&path,source).unwrap();
    let r=run_cli(&["run".into(),"--allow".into(),"console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();r
}
#[test]
fn integer_parity_handles_zero_negative_and_i64_limits() {
    let r=run(r#"assert even of 0.
assert odd of 1.
assert odd of (minus 3).
assert even of (minus 4).
assert even of 9223372036854775806.
assert odd of 9223372036854775807.
let min be (minus 9223372036854775807) minus 1.
assert even of min.
print odd of 9."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"true\n");
}
#[test]
fn integral_decimals_are_supported_and_nonintegral_rejected() {
    let r=run("assert even of 2.0. assert odd of (minus 3.0).");
    assert_eq!(r.status,0,"{}",r.stderr);
    for src in ["print even of 2.5.","print odd of 9223372036854775808.0."] {
        let r=run(src);
        assert_eq!(r.status,65,"{src}: {}",r.stderr);
        assert!(r.stderr.contains("S426"),"{}",r.stderr);
    }
}
#[test]
fn type_errors_are_located() {
    let c=Compiler::new();
    for src in ["print even of true.","print odd of \"text\".","print even of a list of 2, 4."] {
        let e=c.hir(src).unwrap_err();
        assert_eq!(e.code,"S425","{src}: {e}");
        assert!(e.span.is_some());
    }
}
#[test]
fn parity_is_pure_and_composes_with_lists_and_functions() {
    let r=run(r#"define function is_even taking n as number to (even of n).
let values be a list of minus 2, minus 1, 0, 1, 2.
print map every n in values to (invoke is_even to n).
assert count every n in values if (odd of n) is equal to 2."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[true, false, true, false, true]\n");
}
#[test]
fn parity_words_remain_identifiers_and_format() {
    let c=Compiler::new();
    let r=run("let even be 7. let odd be 4. print even. print odd.");
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"7\n4\n");
    assert_eq!(c.format_source("PRINT EVEN OF 4.").unwrap(),"print even of 4.");
    assert_eq!(c.format_source("PRINT ODD OF 3.").unwrap(),"print odd of 3.");
    assert_eq!(c.format_source("LET Even BE 7. PRINT Even.").unwrap(),"let Even be 7. print Even.");
    let src="print even of 2.";
    assert!(c.hir(src).unwrap().to_json().to_string().contains("even"));
    assert!(c.optimized_hir(src).is_ok());
    assert_eq!(c.effect_analysis(src).unwrap().effects(),vec!["console_write".to_owned()]);
}
