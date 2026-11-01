use semauri::{run_cli, Compiler};
use std::{fs, sync::atomic::{AtomicU64, Ordering}};
static NEXT: AtomicU64=AtomicU64::new(1);
fn run(source:&str)->semauri::CliResult {
    let path=std::env::temp_dir().join(format!("semauri-sqrt-{}-{}.suri",
        std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
    fs::write(&path,source).unwrap();
    let r=run_cli(&["run".into(),"--allow".into(),"console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();r
}
#[test]
fn exact_squares_and_non_perfect_roots() {
    let r=run(r#"assert (sqrt of 0) is equal to 0.
assert (sqrt of 1) is equal to 1.
assert (sqrt of 81) is equal to 9.
assert (sqrt of 81.0) is equal to 9.
assert (sqrt of 9223372030926249001) is equal to 3037000499.
let result be sqrt of 2.
assert result is greater than 1.
assert result is less than 2.
print sqrt of 9."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"3\n");
}
#[test]
fn negative_roots_return_located_error() {
    for input in ["print sqrt of (minus 1).","print sqrt of (minus 0.5)."] {
        let r=run(input);
        assert_eq!(r.status,65,"{input}: {}",r.stderr);
        assert!(r.stderr.contains("S424"),"{}",r.stderr);
    }
}
#[test]
fn wrong_type_is_rejected_before_evaluation() {
    let c=Compiler::new();
    for src in ["print sqrt of true.","print sqrt of \"x\".","print sqrt of a list of 4, 9."] {
        let e=c.hir(src).unwrap_err();
        assert_eq!(e.code,"S423","{src}: {e}");
        assert!(e.span.is_some());
    }
}
#[test]
fn sqrt_composes_with_lists_functions_and_square() {
    let r=run(r#"define function magnitude taking n as number to (sqrt of (square of n)).
let values be a list of minus 4, 0, 3.
print map every n in values to (invoke magnitude to n)."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[4, 0, 3]\n");
}
#[test]
fn identifiers_formatter_and_compiler_inspection_stay_compatible() {
    let c=Compiler::new();
    let r=run("let sqrt be 7. print sqrt. print sqrt of 64.");
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"7\n8\n");
    assert_eq!(c.format_source("PRINT SQRT OF 4.").unwrap(),"print sqrt of 4.");
    assert_eq!(c.format_source("LET Sqrt BE 4. PRINT Sqrt.").unwrap(),"let Sqrt be 4. print Sqrt.");
    let src="print sqrt of 64.";
    assert!(c.hir(src).unwrap().to_json().to_string().contains("sqrt"));
    assert!(c.optimized_hir(src).is_ok());
    assert_eq!(c.effect_analysis(src).unwrap().effects(),vec!["console_write".to_owned()]);
}
