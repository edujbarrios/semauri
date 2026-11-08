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
    #[test]
    fn hashed_distinct_preserves_nested_and_storage_sensitive_equality() {
        let input = vec![
            ValueData::Number(NumberValue::Int(1)),
            ValueData::Number(NumberValue::Float(1.0)),
            ValueData::Number(NumberValue::Int(1)),
            ValueData::String("é".into()),
            ValueData::String("é".into()),
            ValueData::Color("red".into()),
            ValueData::Color("red".into()),
            ValueData::List(vec![ValueData::Boolean(true)]),
            ValueData::List(vec![ValueData::Boolean(true)]),
            ValueData::List(vec![ValueData::Boolean(false)]),
        ];
        let expected = vec![
            input[0].clone(), input[1].clone(), input[3].clone(),
            input[5].clone(), input[7].clone(), input[9].clone(),
        ];
        assert_eq!(distinct_values(input), expected);
        let negative_zero = ValueData::Number(NumberValue::Float(-0.0));
        let positive_zero = ValueData::Number(NumberValue::Float(0.0));
        assert_eq!(distinct_fingerprint(&negative_zero), distinct_fingerprint(&positive_zero));
        assert_eq!(distinct_values(vec![negative_zero, positive_zero]).len(), 1);
    }

    #[test]
    fn hashed_distinct_matches_previous_equality_for_mixed_values() {
        let input = vec![
            ValueData::Unit,
            ValueData::Boolean(false),
            ValueData::Boolean(true),
            ValueData::Boolean(false),
            ValueData::String("hello".into()),
            ValueData::String("HELLO".into()),
            ValueData::Number(NumberValue::Float(0.5)),
            ValueData::Number(NumberValue::Float(0.5)),
            ValueData::Number(NumberValue::Int(0)),
            ValueData::List(vec![ValueData::Number(NumberValue::Float(0.0))]),
            ValueData::List(vec![ValueData::Number(NumberValue::Float(-0.0))]),
        ];
        let mut previous = Vec::new();
        for value in &input {
            if !previous.contains(value) {
                previous.push(value.clone());
            }
        }
        assert_eq!(distinct_values(input), previous);
    }


}
