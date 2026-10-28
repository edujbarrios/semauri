use semauri::{run_cli, Compiler};
use std::{fs, sync::atomic::{AtomicU64, Ordering}};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path=std::env::temp_dir().join(format!("semauri-cbrt-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path,source).unwrap();
    let result=run_cli(&["run".into(),"--allow".into(),"console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}
#[test]
fn exact_integer_cubes_and_signed_boundary() {
    let r=run(r#"assert (cbrt of 0) is equal to 0.
assert (cbrt of 1) is equal to 1.
assert (cbrt of (minus 27)) is equal to (minus 3).
assert (cbrt of 9223358842721533951) is equal to 2097151.
let minimum be (minus 9223372036854775807) minus 1.
assert (cbrt of minimum) is equal to (minus 2097152).
print cbrt of 125."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"5\n");
}
#[test]
fn nonperfect_cubes_yield_finite_fractional_roots() {
    let r=run(r#"let x be cbrt of 2.
assert x is greater than 1.
assert x is less than 2.
let negative be cbrt of (minus 2).
assert negative is greater than (minus 2).
assert negative is less than (minus 1).
assert (cbrt of (minus 8.0)) is equal to (minus 2)."#);
    assert_eq!(r.status,0,"{}",r.stderr);
}
#[test]
fn typed_errors_and_contextual_names() {
    let c=Compiler::new();
    for src in ["print cbrt of true.","print cbrt of \"text\".","print cbrt of a list of 1, 2."] {
        let e=c.hir(src).unwrap_err();
        assert_eq!(e.code,"S437","{src}: {e}");
        assert!(e.span.is_some());
    }
    let r=run("let cbrt be 7. print cbrt. print cbrt of 64.");
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"7\n4\n");
    assert_eq!(c.format_source("PRINT CBRT OF 27.").unwrap(),"print cbrt of 27.");
}
#[test]
fn functions_maps_and_optimizer_support_cbrt() {
    let r=run(r#"define function root taking n as number to (cbrt of n).
let values be a list of minus 8, 0, 27.
print map every n in values to (invoke root to n)."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[-2, 0, 3]\n");
    let c=Compiler::new();
    assert!(c.hir("print cbrt of 8.").unwrap().to_json().to_string().contains("cbrt"));
    assert!(c.optimized_hir("print cbrt of 8.").is_ok());
    assert_eq!(c.effect_analysis("print cbrt of 8.").unwrap().effects(),vec!["console_write".to_owned()]);
}
