fn nominal(domain: &str, name: &str, base: Type, promote_from_base: bool) -> Type {
    Type::Nominal(NominalType::new(domain, name, base, promote_from_base))
}

fn filesystem_path_type() -> Type {
    nominal("filesystem", "path", Type::String, true)
}

fn ml_type(name: &str, base: Type) -> Type {
    nominal("ml", name, base, false)
}

fn slot(name: &str, ty: Type) -> PatternSegment {
    PatternSegment::Slot {
        name: name.to_string(),
        ty: Some(ty),
    }
}

fn lit(word: &str) -> PatternSegment {
    PatternSegment::Literal(word.to_string())
}

fn op(
    name: &str,
    verbs: &[&str],
    pattern: Vec<PatternSegment>,
    return_type: Type,
    effects: &[&str],
) -> OperationSpec {
    OperationSpec {
        name: name.to_string(),
        verbs: verbs.iter().map(|value| value.to_string()).collect(),
        pattern,
        return_type,
        effects: effects.iter().map(|value| value.to_string()).collect(),
    }
}

