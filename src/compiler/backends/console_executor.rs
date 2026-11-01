fn execute_console_plan(plan: &RuntimePlan) -> Result<String> {
    let mut output = String::new();
    for operation in &plan.operations {
        if operation.domain != "console" {
            return Err(SemauriError::new(
                ErrorKind::Runtime,
                "S501",
                format!(
                    "Unsupported console runtime operation '{}.{}'",
                    operation.domain, operation.name
                ),
                Some(operation.source_span),
                None,
            ));
        }
        let (argument_name, action) = match operation.name.as_str() {
            "print" => ("value", "print"),
            "assert" => ("condition", "assert"),
            _ => {
                return Err(SemauriError::new(
                    ErrorKind::Runtime,
                    "S501",
                    format!("Unsupported console runtime operation 'console.{}'", operation.name),
                    Some(operation.source_span),
                    None,
                ));
            }
        };
        let value = operation.arguments.get(argument_name).ok_or_else(|| {
            SemauriError::new(
                ErrorKind::Runtime,
                "S501",
                format!("Console {action} operation is missing its {argument_name}"),
                Some(operation.source_span),
                None,
            )
        })?;
        match (action, value) {
            ("print", RuntimeArgument::Value(value)) => {
                output.push_str(&display_console_value(value));
                output.push('\n');
            }
            ("assert", RuntimeArgument::Value(ValueData::Boolean(true))) => {}
            ("assert", RuntimeArgument::Value(ValueData::Boolean(false))) => {
                return Err(SemauriError::new(
                    ErrorKind::Runtime,
                    "S502",
                    "Assertion failed",
                    Some(operation.source_span),
                    Some("Inspect the condition or its input values.".to_string()),
                ));
            }
            ("assert", RuntimeArgument::Value(_)) => {
                return Err(SemauriError::new(
                    ErrorKind::Runtime,
                    "S501",
                    "Assertion condition must be boolean",
                    Some(operation.source_span),
                    None,
                ));
            }
            (_, RuntimeArgument::Ref(_)) => {
                return Err(SemauriError::new(
                    ErrorKind::Runtime,
                    "S501",
                    "Using runtime-produced values in console operations is not supported yet",
                    Some(operation.source_span),
                    None,
                ));
            }
            _ => unreachable!("validated above"),
        }
    }
    Ok(output)
}

fn display_console_value(value: &ValueData) -> String {
    match value {
        ValueData::String(value) | ValueData::Color(value) => value.clone(),
        ValueData::Number(value) => value.to_string(),
        ValueData::Boolean(value) => value.to_string(),
        ValueData::List(values) => format!(
            "[{}]",
            values
                .iter()
                .map(display_console_value)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ValueData::Unit => "unit".to_string(),
    }
}
