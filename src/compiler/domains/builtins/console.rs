fn console_domain() -> DomainSpec {
    DomainSpec {
        name: "console".to_string(),
        default_backend: None,
        artifacts: BTreeMap::new(),
        elements: BTreeMap::new(),
        properties: BTreeMap::new(),
        types: Vec::new(),
        operations: vec![
            OperationSpec {
                name: "print".to_string(),
                verbs: vec!["print".to_string()],
                pattern: vec![PatternSegment::untyped_slot("value")],
                return_type: Type::Unit,
                effects: vec!["console_write".to_string()],
            },
            // Assertions are pure runtime checks: no console or filesystem
            // capability is needed to evaluate an already-computed condition.
            op(
                "assert",
                &["assert"],
                vec![slot("condition", Type::Boolean)],
                Type::Unit,
                &[],
            ),
        ],
    }
}
