use semauri::{run_cli, Compiler};
use std::{fs, sync::atomic::{AtomicU64, Ordering}};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path=std::env::temp_dir().join(format!("semauri-prime-{}-{}.suri",
        std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
    fs::write(&path,source).unwrap();
    let r=run_cli(&["run".into(),"--allow".into(),"console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();r
}
#[test]
fn typed_primality_handles_boundaries_and_large_pseudoprimes() {
    let r=run(r#"assert not (prime of (minus 7)).
assert not (prime of 0).
assert not (prime of 1).
assert prime of 2.
assert prime of 3.
assert prime of 97.
assert not (prime of 561).
assert not (prime of 341550071728321).
assert not (prime of 9223372036854775807).
assert prime of 2.0.
print prime of 17."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"true\n");
}
#[test]
fn invalid_decimal_domain_is_rejected_without_truncation() {
    for src in ["print prime of 2.5.", "print prime of 9223372036854775808.0."] {
        let r=run(src);
        assert_eq!(r.status,65,"{src}: {}",r.stderr);
        assert!(r.stderr.contains("S436"),"{}",r.stderr);
    }
}
#[test]
fn rejects_non_numbers_statically_and_keeps_contextual_name() {
    let c=Compiler::new();
    for src in ["print prime of true.","print prime of \"x\"."] {
        let e=c.hir(src).unwrap_err();
        assert_eq!(e.code,"S435","{e}");
        assert!(e.span.is_some());
    }
    let r=run("let prime be 4. print prime. print prime of 7.");
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"4\ntrue\n");
    assert_eq!(c.format_source("PRINT PRIME OF 7.").unwrap(),"print prime of 7.");
}
#[test]
fn composes_with_functions_lists_and_inspection() {
    let r=run(r#"define function is_prime taking n as number to (prime of n).
let numbers be a list of 2, 4, 17, 18, 19.
print map every n in numbers to (invoke is_prime to n)."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[true, false, true, false, true]\n");
    let c=Compiler::new();
    assert!(c.hir("print prime of 7.").unwrap().to_json().to_string().contains("prime"));
    assert!(c.optimized_hir("print prime of 7.").is_ok());
    assert_eq!(c.effect_analysis("print prime of 7.").unwrap().effects(),vec!["console_write".to_owned()]);
}
