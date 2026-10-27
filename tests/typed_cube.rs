use semauri::{run_cli, Compiler};
use std::{fs, sync::atomic::{AtomicU64, Ordering}};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path=std::env::temp_dir().join(format!("semauri-cube-{}-{}.suri",
        std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
    fs::write(&path,source).unwrap();
    let result=run_cli(&["run".into(),"--allow".into(),"console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();result
}
#[test]
fn exact_cube_and_fractional_values() {
    let result=run(r#"print cube of 0.
print cube of (minus 5).
print cube of 2.5.
print cube of 2097151."#);
    assert_eq!(result.status,0,"{}",result.stderr);
    assert_eq!(result.stdout,"0\n-125\n15.625\n9223358842721533951\n");
}
#[test]
fn checked_integer_overflow_is_diagnostic() {
    for code in ["print cube of 2097152.","print cube of (minus 2097152)."] {
        let result=run(code);
        assert_eq!(result.status,65,"{code}: {}",result.stderr);
        assert!(result.stderr.contains("S434"),"{}",result.stderr);
    }
}
#[test]
fn static_type_and_contextual_identifier() {
    let compiler=Compiler::new();
    for code in ["print cube of true.","print cube of \"x\"."] {
        let err=compiler.hir(code).unwrap_err();
        assert_eq!(err.code,"S433","{err}");
        assert!(err.span.is_some());
    }
    let result=run("let cube be 7. print cube. print cube of 3.");
    assert_eq!(result.status,0,"{}",result.stderr);
    assert_eq!(result.stdout,"7\n27\n");
    assert_eq!(compiler.format_source("PRINT CUBE OF 2.").unwrap(),"print cube of 2.");
}
#[test]
fn composes_with_functions_maps_and_static_inspection() {
    let result=run(r#"define function cubed taking n as number to (cube of n).
let values be a list of minus 2, 0, 3.
print map every n in values to (invoke cubed to n)."#);
    assert_eq!(result.status,0,"{}",result.stderr);
    assert_eq!(result.stdout,"[-8, 0, 27]\n");
    let compiler=Compiler::new();
    assert!(compiler.optimized_hir("print cube of 3.").is_ok());
    assert!(compiler.hir("print cube of 3.").unwrap().to_json().to_string().contains("cube"));
    assert_eq!(compiler.effect_analysis("print cube of 3.").unwrap().effects(),vec!["console_write".to_owned()]);
}
