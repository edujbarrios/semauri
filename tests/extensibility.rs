use semauri::{
    Artifact, BackendRegistry, Compiler, DomainRegistry, DomainSpec, OperationSpec, PatternSegment,
    TokenKind, Type, ValueData,
};

fn canvas_domain() -> DomainSpec {
    DomainSpec::new("canvas")
        .with_artifact("canvas", "canvas")
        .with_element("badge", "badge")
        .with_property("tone", "tone", Type::Color)
}

#[test]
fn external_declarative_domain_and_backend_compile() {
    let mut domains = DomainRegistry::new();
    domains.register(canvas_domain()).unwrap();

    let mut backends = BackendRegistry::new();
    backends.register("canvas", |artifact: &Artifact| match artifact {
        Artifact::Generic(canvas) => {
            let badge = &canvas.elements[0];
            let tone = match badge.properties.get("tone") {
                Some(ValueData::Color(value)) => value.clone(),
                _ => String::new(),
            };
            Ok(format!(
                "{}|{}|{}|{}",
                canvas.title, badge.kind, badge.label, tone
            ))
        }
        _ => Ok("wrong artifact".to_string()),
    });

    let compiler = Compiler::with_registries(domains, backends);
    let result = compiler.compile(
        "Create a canvas called Status Board. Add a badge called Health. Set the tone of the badge called Health to green.",
        Some("canvas"),
    ).unwrap();

    assert_eq!(
        result.output.as_deref(),
        Some("Status Board|badge|Health|green")
    );
}

#[test]
fn external_domain_terms_are_generic_tokens_and_metadata() {
    let mut domains = DomainRegistry::new();
    domains.register(canvas_domain()).unwrap();
    let compiler = Compiler::with_domains(domains.clone());

    let tokens = compiler
        .tokenize("Create a canvas. Add a badge. Set the tone of it to red.")
        .unwrap();
    assert!(tokens
        .iter()
        .any(|token| token.kind == TokenKind::DomainArtifact));
    assert!(tokens
        .iter()
        .any(|token| token.kind == TokenKind::DomainElement));
    assert!(tokens
        .iter()
        .any(|token| token.kind == TokenKind::DomainProperty));

    let metadata = domains.to_json();
    assert_eq!(metadata["domains"][0]["name"], "canvas");
    assert_eq!(metadata["domains"][0]["properties"]["tone"], "tone");
}

#[test]
fn registry_collision_checks_are_transactional_and_reserved_words_fail() {
    let mut domains = DomainRegistry::new();
    domains
        .register(DomainSpec::new("one").with_artifact("thing", "thing"))
        .unwrap();

    let collision = domains.register(
        DomainSpec::new("two")
            .with_artifact("other", "other")
            .with_element("thing", "thing"),
    );
    assert!(collision.unwrap_err().contains("conflicts"));
    assert_eq!(domains.names(), vec!["one".to_string()]);
    assert!(!domains.words().contains(&"other".to_string()));

    let reserved = domains.register(DomainSpec::new("invalid").with_artifact("if", "artifact"));
    assert!(reserved
        .unwrap_err()
        .contains("reserved English vocabulary"));
}

#[test]
fn external_operation_and_action_collision_match_legacy_contract() {
    let archive = DomainSpec::new("archive").with_operation(OperationSpec::new(
        "delete",
        ["delete"],
        vec![PatternSegment::slot("entry", Type::String)],
        Type::Unit,
        ["archive_write"],
    ));
    let mut domains = DomainRegistry::builtins();
    domains.register(archive).unwrap();
    let compiler = Compiler::with_domains(domains);

    let error = compiler.parse("Delete \"tmp.log\".").unwrap_err();
    assert_eq!(error.code, "S240");

    let hir = compiler
        .hir("Within archive: Delete \"old-entry\". End.")
        .unwrap()
        .to_json();
    assert_eq!(
        hir["program"]["fields"]["statements"][0]["fields"]["domain"],
        "archive"
    );
}

#[test]
fn custom_backend_can_render_builtin_ir() {
    let mut backends = BackendRegistry::builtins();
    backends.register("dummy", |_artifact: &Artifact| Ok("dummy".to_string()));
    let compiler = Compiler::with_registries(DomainRegistry::builtins(), backends);

    let result = compiler
        .compile("Create a web called Hello.", Some("dummy"))
        .unwrap();
    assert_eq!(result.output.as_deref(), Some("dummy"));
}

#[test]
fn domain_without_default_backend_requires_explicit_backend() {
    let mut domains = DomainRegistry::new();
    domains.register(canvas_domain()).unwrap();
    let compiler = Compiler::with_domains(domains);

    let error = compiler
        .compile("Create a canvas called Status Board.", None)
        .unwrap_err();
    assert_eq!(error.code, "S404");
}

#[test]
fn external_color_property_makes_implicit_make_ambiguous() {
    let mut domains = DomainRegistry::builtins();
    domains.register(canvas_domain()).unwrap();
    let compiler = Compiler::with_domains(domains);

    let error = compiler
        .parse("Create a web called Shop. Add a button called Buy. Make it blue.")
        .unwrap_err();
    assert_eq!(error.code, "S236");
}
