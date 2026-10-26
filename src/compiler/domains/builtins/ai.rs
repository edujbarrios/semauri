fn ai_domain() -> DomainSpec {
    let language_model = ai_type("language_model", Type::String);
    let vision_model = ai_type("vision_model", Type::String);
    let response = ai_type("response", Type::String);
    let embedding = ai_type("embedding", Type::Opaque);
    let tool = ai_type("tool", Type::Opaque);
    let agent = ai_type("agent", Type::Opaque);

    let types = [
        &language_model,
        &vision_model,
        &response,
        &embedding,
        &tool,
        &agent,
    ]
    .iter()
    .filter_map(|ty| match ty {
        Type::Nominal(value) => Some(value.clone()),
        _ => None,
    })
    .collect();

    DomainSpec {
        name: "ai".to_string(),
        default_backend: None,
        artifacts: BTreeMap::new(),
        elements: BTreeMap::new(),
        properties: BTreeMap::new(),
        types,
        operations: vec![
            op(
                "language_model",
                &["connect"],
                vec![
                    lit("language"),
                    lit("model"),
                    slot("identifier", Type::String),
                ],
                language_model.clone(),
                &["model_load"],
            ),
            op(
                "vision_model",
                &["activate"],
                vec![
                    lit("vision"),
                    lit("model"),
                    slot("identifier", Type::String),
                ],
                vision_model.clone(),
                &["model_load"],
            ),
            op(
                "generate_text",
                &["prompt"],
                vec![
                    slot("model", language_model.clone()),
                    lit("with"),
                    slot("prompt", Type::String),
                ],
                response.clone(),
                &["network", "model_inference"],
            ),
            op(
                "inspect_image",
                &["inspect"],
                vec![
                    lit("image"),
                    slot("image", Type::String),
                    lit("using"),
                    slot("model", vision_model),
                    lit("with"),
                    slot("prompt", Type::String),
                ],
                response.clone(),
                &["filesystem_read", "network", "model_inference"],
            ),
            op(
                "embed_text",
                &["embed"],
                vec![
                    slot("text", Type::String),
                    lit("using"),
                    slot("model", language_model.clone()),
                ],
                embedding,
                &["network", "model_inference"],
            ),
            op(
                "typed_tool",
                &["declare"],
                vec![
                    lit("tool"),
                    slot("name", Type::String),
                    lit("schema"),
                    slot("schema", Type::String),
                ],
                tool,
                &[],
            ),
            op(
                "bounded_agent",
                &["compose"],
                vec![
                    lit("agent"),
                    lit("using"),
                    slot("model", language_model),
                    lit("system"),
                    slot("system", Type::String),
                    lit("max"),
                    slot("max_turns", Type::Number),
                    lit("turns"),
                ],
                agent.clone(),
                &[],
            ),
            op(
                "run_agent",
                &["ask"],
                vec![
                    slot("agent", agent),
                    lit("with"),
                    slot("prompt", Type::String),
                ],
                response,
                &["network", "model_inference", "tool_execution"],
            ),
        ],
    }
}
