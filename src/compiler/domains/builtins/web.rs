fn web_domain() -> DomainSpec {
    DomainSpec {
        name: "web".to_string(),
        default_backend: Some("html".to_string()),
        artifacts: [
            ("web", "web"),
            ("website", "web"),
            ("webpage", "web"),
            ("page", "web"),
        ]
        .into_iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect(),
        elements: [("button", "button"), ("image", "image"), ("picture", "image")]
            .into_iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect(),
        properties: [("color", ("color", Type::Color))]
            .into_iter()
            .map(|(surface, (kind, ty))| {
                (surface.to_string(), (kind.to_string(), ty))
            })
            .collect(),
        types: vec![],
        operations: vec![],
    }
}

