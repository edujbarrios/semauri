fn execute_console_plan(plan: &RuntimePlan) -> Result<String> {
    let mut output = String::new();
    for operation in &plan.operations {
        if operation.domain != "console" || operation.name != "print" {
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
        let value = operation.arguments.get("value").ok_or_else(|| {
            SemauriError::new(
                ErrorKind::Runtime,
                "S501",
                "Console print operation is missing its value",
                Some(operation.source_span),
                None,
            )
        })?;
        match value {
            RuntimeArgument::Value(value) => {
                output.push_str(&display_console_value(value));
                output.push('\n');
            }
            RuntimeArgument::Ref(_) => {
                return Err(SemauriError::new(
                    ErrorKind::Runtime,
                    "S501",
                    "Printing runtime-produced values is not supported yet",
                    Some(operation.source_span),
                    Some("Print compile-time values such as literals, arithmetic results, or immutable bindings.".to_string()),
                ))
            }
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
