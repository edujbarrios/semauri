use semauri::{run_cli, Compiler};
use std::{fs, sync::atomic::{AtomicU64, Ordering}};
static NEXT: AtomicU64=AtomicU64::new(1);
fn run(source:&str)->semauri::CliResult {
    let path=std::env::temp_dir().join(format!("semauri-factorial-{}-{}.suri",
        std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
    fs::write(&path,source).unwrap();
    let r=run_cli(&["run".into(),"--allow".into(),"console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();r
}
#[test]
fn factorial_exact_endpoints_and_integral_decimals() {
    let r=run(r#"print factorial of 0.
print factorial of 1.
print factorial of 5.
print factorial of 20.
print factorial of 5.0."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"1\n1\n120\n2432902008176640000\n120\n");
}
#[test]
fn rejects_negative_fractional_and_overflow_cases() {
    for src in [
        "print factorial of (minus 1).",
        "print factorial of 2.5.",
        "print factorial of 21.",
        "print factorial of 21.0.",
    ] {
        let r=run(src);
        assert_eq!(r.status,65,"{src}: {}",r.stderr);
        assert!(r.stderr.contains("S432"),"{src}: {}",r.stderr);
    }
}
#[test]
fn static_types_and_contextual_names() {
    let c=Compiler::new();
    for src in ["print factorial of true.","print factorial of \"hello\".","print factorial of a list of 1, 2."] {
        let e=c.hir(src).unwrap_err();
        assert_eq!(e.code,"S431","{src}: {e}");
        assert!(e.span.is_some());
    }
    let r=run("let factorial be 4. print factorial. print factorial of 3.");
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"4\n6\n");
    assert_eq!(c.format_source("PRINT FACTORIAL OF 5.").unwrap(),"print factorial of 5.");
    assert_eq!(c.format_source("LET Factorial BE 5. PRINT Factorial.").unwrap(),"let Factorial be 5. print Factorial.");
}
#[test]
fn runs_in_functions_lists_and_optimizer() {
    let r=run(r#"define function fact taking n as number to (factorial of n).
let values be a list of 0, 1, 3, 5.
print map every n in values to (invoke fact to n)."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[1, 1, 6, 120]\n");
    let c=Compiler::new();
    let src="print factorial of 5.";
    assert!(c.hir(src).unwrap().to_json().to_string().contains("factorial"));
    assert!(c.optimized_hir(src).is_ok());
    assert_eq!(c.effect_analysis(src).unwrap().effects(),vec!["console_write".to_owned()]);
}
