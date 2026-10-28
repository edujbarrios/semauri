use semauri::Compiler;

#[test]
fn check_style_analysis_and_compilation_leave_no_explanation_strings() {
    let source = r#"within web:
  create a web called Example.
  add a button called Start.
end."#;
    let compiler = Compiler::new();
    let (_, fast) = compiler.analyze_without_explanations(source).unwrap();
    assert!(fast.explanations.is_empty());
    let complete = compiler.compile(source, None).unwrap();
    assert!(complete.semantic.explanations.is_empty());
    // Preserve the existing public Compiler::analyze contract for explain
    // and API clients that explicitly inspect semantic explanations.
    let (_, explained) = compiler.analyze(source).unwrap();
    assert_eq!(explained.explanations.len(), 4);
    assert_eq!(
        explained.explanations[0],
        "Entered semantic domain scope 'web'."
    );
}

#[test]
fn no_explanation_collection_even_after_repeated_list_scans() {
    let source = r#"let items be range from 1 to 10000.
let mapped be map every n in items to (n times n).
let count be count every n in mapped if (n is greater than 100).
print count."#;
    let compiler = Compiler::new();
    let (_, fast) = compiler.analyze_without_explanations(source).unwrap();
    assert!(fast.explanations.is_empty());
    let compiled = compiler.compile(source, None).unwrap();
    assert!(compiled.semantic.explanations.is_empty());
    assert_eq!(fast.runtime_plan.to_json(), compiled.semantic.runtime_plan.to_json());
}
