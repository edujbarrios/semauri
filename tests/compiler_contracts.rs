use semauri::{Artifact, CapabilityPolicy, Compiler};

#[test]
fn hir_symbols_shadowing_and_unselected_branches_are_preserved() {
    let compiler = Compiler::new();
    let hir = compiler
        .hir(
            "Let value be 1. Create a web. If true: Let value be 2. Add a button called Yes. Otherwise: Let hidden be 3. Add a button called No. End.",
        )
        .unwrap();
    assert!(hir.symbols.len() >= 3);
    assert_ne!(hir.symbols[0].id, hir.symbols[1].id);

    let json = hir.to_json().to_string();
    assert!(json.contains("\"kind\":\"if\""));
    assert!(json.contains("hidden"));
}

#[test]
fn nominal_promotion_and_effect_metadata_survive_hir() {
    let hir = Compiler::new()
        .hir("Let path be \"notes.txt\". Write \"hello\" to path.")
        .unwrap()
        .to_json()
        .to_string();

    assert!(hir.contains("\"kind\":\"promote\""));
    assert!(hir.contains("filesystem.path"));
    assert!(hir.contains("filesystem_write"));
}

#[test]
fn effects_are_conservative_before_optimization_and_policy_can_enforce_them() {
    let compiler = Compiler::new();
    let source = "Create a web. If false: Write \"never\" to \"x.txt\". End.";
    let effects = compiler.effect_analysis(source).unwrap();
    assert_eq!(effects.effects(), vec!["filesystem_write".to_string()]);

    let error = compiler
        .validate_capabilities(source, &CapabilityPolicy::allow_none())
        .unwrap_err();
    assert_eq!(error.code, "S334");
}

#[test]
fn optimizer_does_not_remove_a_trapping_initializer() {
    let error = Compiler::new()
        .compile("Let unused be 1 divided by 0. Create a web.", None)
        .unwrap_err();
    assert_eq!(error.code, "S317");
}

#[test]
fn optimizer_keeps_full_symbol_table_but_removes_dead_static_structure() {
    let optimized = Compiler::new()
        .optimized_hir(
            "Let a be 1. Let b be a plus 1. Create a web. If true: Add a button called Yes. Otherwise: Add a button called No. End.",
        )
        .unwrap();
    assert!(optimized.result.symbols.len() >= 2);
    let json = optimized.to_json().to_string();
    assert!(!json.contains("\"kind\":\"if\""));
}

#[test]
fn runtime_values_form_references_and_cannot_drive_compile_time_control_flow() {
    let compiler = Compiler::new();
    let plan = compiler
        .runtime_plan(
            "Within ml: Let dataset be Open dataset \"./data\". Let transformed be Transform dataset using \"normalize\". End.",
        )
        .unwrap()
        .to_json()
        .to_string();
    assert!(plan.contains("runtime_value_ref"));
    assert!(plan.contains("transform_dataset"));

    let error = compiler
        .analyze(
            "Within ml: Let dataset be Open dataset \"./data\". If dataset is equal to dataset: Let x be 1. End. End.",
        )
        .unwrap_err();
    assert_eq!(error.code, "S335");
}

#[test]
fn multi_domain_program_ir_preserves_declarative_focus() {
    let result = Compiler::new()
        .compile(
            "Create a web called Shop. Add a button called Buy. Write \"hello\" to \"notes.txt\". Set the color of the button called Buy to red.",
            None,
        )
        .unwrap();

    assert_eq!(result.outputs.len(), 2);
    let Artifact::Web(web) = &result.semantic.program_ir.units[0].artifact else {
        panic!("expected web artifact first");
    };
    assert_eq!(web.elements[0].properties.get("color").unwrap().to_json(), serde_json::json!("red"));
}

#[test]
fn schema_contract_and_backend_mismatch_errors_are_preserved() {
    let compiler = Compiler::new();

    let type_error = compiler
        .hir(
            "Create a schema called Pet. Add a field called Name. Set the required of the field called Name to \"yes\".",
        )
        .unwrap_err();
    assert_eq!(type_error.code, "S313");

    let datatype_error = compiler
        .analyze(
            "Create a schema called Pet. Add a field called Name. Set the datatype of the field called Name to \"uuid\".",
        )
        .unwrap_err();
    assert_eq!(datatype_error.code, "S328");

    let backend_error = compiler
        .compile("Create a web called Shop.", Some("json-schema"))
        .unwrap_err();
    assert_eq!(backend_error.code, "S401");
}

#[test]
fn source_span_and_property_provenance_remain_available() {
    let result = Compiler::new()
        .compile(
            "Create a web called Shop.\nAdd a button called Buy.\nSet the color of the button called Buy to red.",
            None,
        )
        .unwrap();
    let Artifact::Web(web) = &result.semantic.program_ir.units[0].artifact else {
        panic!("expected web artifact");
    };
    let span = web.elements[0].property_provenance["color"];
    assert_eq!(span.start.line, 3);
    assert!(span.end.column > span.start.column);
}
