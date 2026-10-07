fn structured_data_domain() -> DomainSpec {
    DomainSpec {
        name: "structured_data".to_string(),
        default_backend: Some("json-schema".to_string()),
        artifacts: [("schema", "schema")]
            .into_iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect(),
        elements: [("field", "field")]
            .into_iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect(),
        properties: [
            ("datatype", ("datatype", Type::String)),
            ("required", ("required", Type::Boolean)),
        ]
        .into_iter()
        .map(|(surface, (kind, ty))| {
            (surface.to_string(), (kind.to_string(), ty))
        })
        .collect(),
        types: vec![],
        operations: vec![],
    }
}

