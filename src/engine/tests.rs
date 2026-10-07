#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexer_preserves_spans_and_domain_metadata() {
        let compiler = Compiler::new();
        let tokens = compiler
            .tokenize("Create a web for a pet store.")
            .unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Create);
        assert_eq!(tokens[2].kind, TokenKind::DomainArtifact);
        assert_eq!(tokens[0].span, SourceSpan::new(1, 1, 1, 7));
    }

    #[test]
    fn readme_program_compiles_to_red_button() {
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
        assert!(output.contains("<title>Pet Shop</title>"));
        assert!(output.contains("<button style=\"color: red\">Buy</button>"));
        assert!(!output.contains("color: green"));
    }

    #[test]
    fn filesystem_plan_and_effects_are_explicit() {
        let source = r#"
Write "hello" to "notes.txt".
Copy "notes.txt" to "backup.txt".
"#;
        let compiler = Compiler::new();
        let result = compiler.compile(source, None).unwrap();
        assert_eq!(result.backend.as_deref(), Some("posix-sh"));
        assert!(result.output.as_deref().unwrap().contains("cp notes.txt backup.txt"));
        assert_eq!(
            result.effects.effects(),
            vec!["filesystem_read".to_string(), "filesystem_write".to_string()]
        );
    }

    #[test]
    fn ml_operations_form_runtime_ssa_plan() {
        let source = r#"
Within ml:
  Let dataset be Open dataset "./images".
  Let model be Load model "resnet18".
  Let device be Select device "cuda".
  Let run be Train model using dataset on device for 10 epochs.
End.
"#;
        let plan = Compiler::new().runtime_plan(source).unwrap();
        assert_eq!(plan.operations.len(), 4);
        assert_eq!(plan.operations[3].name, "train");
        assert_eq!(
            plan.operations[3].result.as_ref().unwrap().ty.to_string(),
            "ml.training_run"
        );
    }
}
