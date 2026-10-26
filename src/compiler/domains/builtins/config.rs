fn config_domain() -> DomainSpec {
    DomainSpec {
        name: "config".to_string(),
        default_backend: None,
        artifacts: BTreeMap::new(),
        elements: BTreeMap::new(),
        properties: BTreeMap::new(),
        types: Vec::new(),
        operations: vec![
            op(
                "read_string",
                &["string"],
                vec![lit("config"), slot("name", Type::String)],
                Type::String,
                &["config_read"],
            ),
            op(
                "read_number",
                &["number"],
                vec![lit("config"), slot("name", Type::String)],
                Type::Number,
                &["config_read"],
            ),
            op(
                "read_boolean",
                &["boolean"],
                vec![lit("config"), slot("name", Type::String)],
                Type::Boolean,
                &["config_read"],
            ),
        ],
    }
}
