fn render_json_schema(artifact: &Artifact) -> Result<String> {
    let Artifact::Schema(program) = artifact else {
        return Err(SemauriError::backend(
            "S401",
            "JSON Schema backend cannot render this semantic IR",
            None,
        ));
    };
    let mut properties = JsonMap::new();
    let mut required = Vec::new();
    for field in &program.fields {
        let datatype = match field.properties.get("datatype") {
            Some(ValueData::String(value)) => value.clone(),
            _ => "string".to_string(),
        };
        properties.insert(field.label.clone(), json!({"type": datatype}));
        if matches!(field.properties.get("required"), Some(ValueData::Boolean(true))) {
            required.push(field.label.clone());
        }
    }
    let mut document = JsonMap::new();
    document.insert(
        "$schema".to_string(),
        json!("https://json-schema.org/draft/2020-12/schema"),
    );
    document.insert("title".to_string(), json!(program.title));
    document.insert("type".to_string(), json!("object"));
    document.insert("properties".to_string(), JsonValue::Object(properties));
    if !required.is_empty() {
        document.insert("required".to_string(), json!(required));
    }
    Ok(format!(
        "{}\n",
        serde_json::to_string_pretty(&JsonValue::Object(document)).unwrap()
    ))
}

