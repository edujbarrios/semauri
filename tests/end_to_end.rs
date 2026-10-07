use semauri::{run_cli, Artifact, CapabilityPolicy, Compiler, TokenKind};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

fn temp_file(contents: &str) -> PathBuf {
    let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("semauri-rust-parity-{}-{id}.sema", std::process::id()));
    fs::write(&path, contents).unwrap();
    path
}

#[test]
fn lexer_matches_core_surface_contract() {
    let tokens = Compiler::new()
        .tokenize("Create a web for a pet store.")
        .unwrap();
    let kinds = tokens.iter().map(|token| token.kind).collect::<Vec<_>>();

    assert_eq!(
        kinds,
        vec![
            TokenKind::Create,
            TokenKind::Article,
            TokenKind::DomainArtifact,
            TokenKind::For,
            TokenKind::Article,
            TokenKind::Word,
            TokenKind::Word,
            TokenKind::Dot,
            TokenKind::Eof
        ]
    );
    assert_eq!(tokens[0].span.start.line, 1);
    assert_eq!(tokens[0].span.start.column, 1);
    assert_eq!(tokens[0].span.end.column, 7);
}

#[test]
fn readme_program_compiles_deterministically() {
    let source = r#"
Let price be 18.
Let tax be 4.
Let total be price plus tax.
Create a web called Pet Shop.
Add a button called Buy.
If total is greater than 20:
  Set the color of the button called Buy to red.
Otherwise:
  Set the color of the button called Buy to green.
End.
"#;

    let result = Compiler::new().compile(source, None).unwrap();
    let output = result.output.unwrap();

    assert_eq!(result.backend.as_deref(), Some("html"));
    assert!(output.contains("<title>Pet Shop</title>"));
    assert!(output.contains("<button style=\"color: red\">Buy</button>"));
    assert!(!output.contains("color: green"));
}

#[test]
fn html_backend_escapes_title() {
    let output = Compiler::new()
        .compile(r#"Create a web called "Cats & Dogs"."#, None)
        .unwrap()
        .output
        .unwrap();

    assert!(output.contains("Cats &amp; Dogs"));
}

#[test]
fn pronouns_and_named_references_keep_semantics() {
    let source = r#"
Create a web called Shop.
Add a button called Buy.
Make it blue.
"#;
    let (_, semantic) = Compiler::new().analyze(source).unwrap();
    let Artifact::Web(web) = &semantic.program_ir.units[0].artifact else {
        panic!("expected web IR");
    };
    assert!(matches!(web.elements[0].properties.get("color"), Some(semauri::ValueData::Color(value)) if value == "blue"));

    let ambiguous = r#"
Create a web called Shop.
Add a button called Buy.
Add an image called Logo.
Make it blue.
"#;
    let error = Compiler::new().analyze(ambiguous).unwrap_err();
    assert_eq!(error.code, "S305");
}

#[test]
fn collections_scopes_and_short_circuit_match_reference() {
    let source = r#"
Let colors be a list of red, green.
Create a web called Shop.
Add a button called Buy.
For every accent in colors:
  Set the color of the button called Buy to accent.
End.
If true or 1 divided by 0 is equal to 1:
  Add a button called Safe.
End.
"#;

    let result = Compiler::new().compile(source, None).unwrap();
    let output = result.output.unwrap();
    assert!(output.contains("color: green"));
    assert!(output.contains(">Safe</button>"));

    let error = Compiler::new()
        .analyze("Create a web. If true or missing is equal to true: Add a button called Invalid. End.")
        .unwrap_err();
    assert_eq!(error.code, "S312");
}

#[test]
fn filesystem_workflow_preserves_plan_backend_and_effects() {
    let source = r#"
Within filesystem:
  Mkdir "build".
  Write "hello" to "build/notes.txt".
  Append " world" to "build/notes.txt".
  Copy "build/notes.txt" to "build/backup.txt".
  Move "build/backup.txt" to "build/archive.txt".
  Touch "build/complete.marker".
  Delete "build/old.tmp".
End.
"#;

    let result = Compiler::new().compile(source, None).unwrap();
    assert_eq!(result.backend.as_deref(), Some("posix-sh"));
    let output = result.output.unwrap();
    assert!(output.contains("mkdir -p build"));
    assert!(output.contains("printf '%s' hello > build/notes.txt"));
    assert!(output.contains("cp build/notes.txt build/backup.txt"));
    assert!(output.contains("mv build/backup.txt build/archive.txt"));
    assert!(output.contains("touch build/complete.marker"));
    assert!(output.contains("rm -f build/old.tmp"));
    assert_eq!(
        result.effects.effects(),
        vec!["filesystem_read".to_string(), "filesystem_write".to_string()]
    );
}

#[test]
fn capability_policy_remains_explicit() {
    let compiler = Compiler::new();
    let source = r#"Write "hello" to "notes.txt"."#;

    let error = compiler
        .validate_capabilities(source, &CapabilityPolicy::allow_none())
        .unwrap_err();

    assert_eq!(error.code, "S334");
    assert!(error.message.contains("filesystem_write"));
}

#[test]
fn structured_data_compiles_to_json_schema() {
    let source = r#"
Create a schema called Pet.
Add a field called Name.
Set the datatype of the field called Name to "string".
Set the required of the field called Name to true.
Add a field called Age.
Set the datatype of the field called Age to "integer".
"#;

    let result = Compiler::new().compile(source, None).unwrap();
    assert_eq!(result.backend.as_deref(), Some("json-schema"));
    let json: Value = serde_json::from_str(result.output.as_deref().unwrap()).unwrap();
    assert_eq!(json["title"], "Pet");
    assert_eq!(json["properties"]["Name"]["type"], "string");
    assert_eq!(json["properties"]["Age"]["type"], "integer");
    assert_eq!(json["required"], serde_json::json!(["Name"]));
}

#[test]
fn multidomain_compilation_returns_independent_outputs() {
    let source = r#"
Create a web called Pet Shop.
Add a button called Buy.
Write "hello" to "notes.txt".
Set the color of the button called Buy to red.
"#;

    let result = Compiler::new().compile(source, None).unwrap();
    assert!(result.output.is_none());
    assert!(result.backend.is_none());
    assert_eq!(result.outputs.len(), 2);
    assert_eq!(
        result.outputs.iter().map(|output| output.backend.as_str()).collect::<Vec<_>>(),
        vec!["html", "posix-sh"]
    );

    let error = Compiler::new().compile(source, Some("html")).unwrap_err();
    assert_eq!(error.code, "S405");
}

#[test]
fn ml_runtime_plan_preserves_typed_lineage() {
    let source = r#"
Within ml:
  Let dataset be Open dataset "./images".
  Let model be Load model "resnet18".
  Let device be Select device "cuda".
  Let config be Configure training for 12 epochs using optimizer "adamw" learning rate 0.0003 batch size 32 seed 42.
  Let run be Fit model using dataset on device with config.
End.
"#;

    let plan = Compiler::new().runtime_plan(source).unwrap();
    assert_eq!(
        plan.operations.iter().map(|operation| operation.name.as_str()).collect::<Vec<_>>(),
        vec!["open_dataset", "load_model", "select_device", "configure_training", "fit"]
    );
    assert_eq!(
        plan.operations.last().unwrap().result.as_ref().unwrap().ty.to_string(),
        "ml.training_run"
    );
}

#[test]
fn ml_static_validation_codes_are_preserved() {
    let invalid = r#"
Within ml:
  Let config be Configure training for 0 epochs using optimizer "adam" learning rate 0.001 batch size 8 seed 1.
End.
"#;
    let error = Compiler::new().hir(invalid).unwrap_err();
    assert_eq!(error.code, "S336");

    let invalid_metric = r#"
Within ml:
  Let dataset be Open dataset "./validation".
  Let model be Load model "resnet18".
  Let device be Select device "cpu".
  Let evaluation be Evaluate model on dataset using device metric "bleu".
End.
"#;
    let error = Compiler::new().hir(invalid_metric).unwrap_err();
    assert_eq!(error.code, "S341");
}

#[test]
fn cli_run_requires_authorization_and_dry_run_is_safe() {
    let path = temp_file(r#"Write "hello" to "notes.txt"."#);

    let denied = run_cli(&["run".into(), path.display().to_string()]);
    assert_eq!(denied.status, 65);
    assert!(denied.stderr.contains("S334"));

    let dry = run_cli(&[
        "run".into(),
        "--dry-run".into(),
        path.display().to_string(),
    ]);
    assert_eq!(dry.status, 0);
    assert!(dry.stdout.contains("printf"));

    let _ = fs::remove_file(path);
}

#[test]
fn optimizer_preserves_output_and_removes_static_control_flow() {
    let source = r#"
Let price be 18.
Let tax be 4.
Let total be price plus tax.
Create a web called Pet Shop.
If total is greater than 20:
  Add a button called Buy.
Otherwise:
  Add a button called Wait.
End.
"#;

    let optimized = Compiler::new().optimized_hir(source).unwrap();
    let json = optimized.to_json().to_string();
    assert!(optimized.changes > 0);
    assert!(!json.contains("\"kind\":\"if\""));
    assert!(!json.contains("\"kind\":\"let\""));

    let output = Compiler::new().compile(source, None).unwrap().output.unwrap();
    assert!(output.contains(">Buy</button>"));
}

#[test]
fn source_spans_use_exclusive_end_columns() {
    let tokens = Compiler::new().tokenize("Make it blue.").unwrap();
    assert_eq!(tokens[0].span.start.column, 1);
    assert_eq!(tokens[0].span.end.column, 5);
    assert_eq!(tokens[1].span.start.column, 6);
    assert_eq!(tokens[1].span.end.column, 8);
    assert_eq!(tokens[2].span.start.column, 9);
    assert_eq!(tokens[2].span.end.column, 13);
}
