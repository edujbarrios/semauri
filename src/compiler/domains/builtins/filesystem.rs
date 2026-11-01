fn filesystem_domain() -> DomainSpec {
    let path = filesystem_path_type();
    DomainSpec {
        name: "filesystem".to_string(),
        default_backend: Some("posix-sh".to_string()),
        artifacts: BTreeMap::new(),
        elements: BTreeMap::new(),
        properties: BTreeMap::new(),
        types: match &path {
            Type::Nominal(value) => vec![value.clone()],
            _ => vec![],
        },
        operations: vec![
            op(
                "write",
                &["write"],
                vec![slot("content", Type::String), lit("to"), slot("path", path.clone())],
                Type::Unit,
                &["filesystem_write"],
            ),
            op(
                "append",
                &["append"],
                vec![slot("content", Type::String), lit("to"), slot("path", path.clone())],
                Type::Unit,
                &["filesystem_write"],
            ),
            op(
                "copy",
                &["copy"],
                vec![slot("source", path.clone()), lit("to"), slot("destination", path.clone())],
                Type::Unit,
                &["filesystem_read", "filesystem_write"],
            ),
            op(
                "move",
                &["move"],
                vec![slot("source", path.clone()), lit("to"), slot("destination", path.clone())],
                Type::Unit,
                &["filesystem_read", "filesystem_write"],
            ),
            op(
                "make_directory",
                &["mkdir"],
                vec![slot("path", path.clone())],
                Type::Unit,
                &["filesystem_write"],
            ),
            op(
                "touch",
                &["touch"],
                vec![slot("path", path.clone())],
                Type::Unit,
                &["filesystem_write"],
            ),
            op(
                "delete",
                &["delete", "remove"],
                vec![slot("path", path)],
                Type::Unit,
                &["filesystem_write"],
            ),
        ],
    }
}

