#[derive(Clone, Debug)]
pub struct DomainRegistry {
    domains: Vec<DomainSpec>,
}

impl Default for DomainRegistry {
    fn default() -> Self {
        Self::builtins()
    }
}

impl DomainRegistry {
    pub fn new() -> Self {
        Self { domains: Vec::new() }
    }

    pub fn builtins() -> Self {
        let mut registry = Self::new();
        for domain in [
            web_domain(),
            structured_data_domain(),
            filesystem_domain(),
            console_domain(),
            ml_domain(),
        ] {
            registry
                .register(domain)
                .expect("built-in semantic domains must be valid");
        }
        registry
    }

    pub fn register(
        &mut self,
        domain: DomainSpec,
    ) -> std::result::Result<&mut Self, String> {
        if self.domains.iter().any(|existing| existing.name == domain.name) {
            return Err(format!("Domain '{}' is already registered", domain.name));
        }

        let pending = domain_surface_entries(&domain)?;
        for (word, is_action) in &pending {
            if keyword_kind(word).is_some() || is_color(word) {
                return Err(format!(
                    "Semantic domain term '{}' conflicts with reserved English vocabulary",
                    word
                ));
            }

            for existing in &self.domains {
                for (existing_word, existing_is_action) in domain_surface_entries(existing)? {
                    if existing_word != *word {
                        continue;
                    }
                    if *is_action && existing_is_action {
                        continue;
                    }
                    return Err(format!(
                        "Domain term '{}' conflicts between '{}' and '{}'",
                        word, existing.name, domain.name
                    ));
                }
            }
        }

        self.domains.push(domain);
        Ok(self)
    }

    pub fn with_domain(
        mut self,
        domain: DomainSpec,
    ) -> std::result::Result<Self, String> {
        self.register(domain)?;
        Ok(self)
    }

    pub fn words(&self) -> Vec<String> {
        let mut words = self
            .domains
            .iter()
            .flat_map(|domain| domain_surface_entries(domain).unwrap_or_default())
            .map(|(word, _)| word)
            .collect::<Vec<_>>();
        words.sort();
        words.dedup();
        words
    }

    pub fn names(&self) -> Vec<String> {
        let mut names = self
            .domains
            .iter()
            .map(|domain| domain.name.clone())
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    pub fn fetch(&self, name: &str) -> Result<&DomainSpec> {
        self.domains
            .iter()
            .find(|domain| domain.name == name)
            .ok_or_else(|| {
                SemauriError::semantic(
                    "S326",
                    format!("Unknown semantic domain '{name}'"),
                    None,
                    None,
                )
            })
    }

    pub fn classify(&self, word: &str) -> Option<TokenLiteral> {
        let key = word.to_lowercase();
        let mut action_candidates = Vec::new();

        for domain in &self.domains {
            if let Some(kind) = domain.artifacts.get(&key) {
                return Some(TokenLiteral::DomainTerm(DomainTerm {
                    domain: domain.name.clone(),
                    category: DomainCategory::Artifact,
                    kind: kind.clone(),
                }));
            }
            if let Some(kind) = domain.elements.get(&key) {
                return Some(TokenLiteral::DomainTerm(DomainTerm {
                    domain: domain.name.clone(),
                    category: DomainCategory::Element,
                    kind: kind.clone(),
                }));
            }
            if let Some((kind, _)) = domain.properties.get(&key) {
                return Some(TokenLiteral::DomainTerm(DomainTerm {
                    domain: domain.name.clone(),
                    category: DomainCategory::Property,
                    kind: kind.clone(),
                }));
            }
            for operation in &domain.operations {
                if operation.verbs.iter().any(|verb| verb == &key) {
                    action_candidates.push(DomainTerm {
                        domain: domain.name.clone(),
                        category: DomainCategory::Action,
                        kind: operation.name.clone(),
                    });
                }
            }
        }

        if action_candidates.is_empty() {
            None
        } else {
            Some(TokenLiteral::ActionCandidates(action_candidates))
        }
    }

    fn infer_property_for_type(&self, value_type: &Type) -> Option<(String, String)> {
        let mut candidates = Vec::new();
        for domain in &self.domains {
            for (_surface, (kind, ty)) in &domain.properties {
                if ty == value_type {
                    let candidate = (domain.name.clone(), kind.clone());
                    if !candidates.contains(&candidate) {
                        candidates.push(candidate);
                    }
                }
            }
        }
        if candidates.len() == 1 {
            candidates.into_iter().next()
        } else {
            None
        }
    }

    fn property_type(&self, domain_name: &str, property: &str) -> Option<Type> {
        self.domains
            .iter()
            .find(|domain| domain.name == domain_name)
            .and_then(|domain| {
                domain
                    .properties
                    .values()
                    .find(|(kind, _)| kind == property)
                    .map(|(_, ty)| ty.clone())
            })
    }

    pub fn to_json(&self) -> JsonValue {
        let mut domains = self.domains.iter().collect::<Vec<_>>();
        domains.sort_by(|a, b| a.name.cmp(&b.name));
        json!({
            "domains": domains.into_iter().map(DomainSpec::to_json).collect::<Vec<_>>()
        })
    }
}

fn domain_surface_entries(
    domain: &DomainSpec,
) -> std::result::Result<Vec<(String, bool)>, String> {
    let mut entries = Vec::new();
    let mut seen = BTreeSet::new();

    for surface in domain
        .artifacts
        .keys()
        .chain(domain.elements.keys())
        .chain(domain.properties.keys())
    {
        let word = surface.to_lowercase();
        if !seen.insert(word.clone()) {
            return Err(format!(
                "Duplicate term '{}' in domain '{}'",
                word, domain.name
            ));
        }
        entries.push((word, false));
    }

    for operation in &domain.operations {
        if operation.verbs.is_empty() {
            return Err(format!(
                "Operation '{}' requires at least one verb",
                operation.name
            ));
        }
        let mut slot_names = BTreeSet::new();
        for segment in &operation.pattern {
            if let PatternSegment::Slot { name, .. } = segment {
                if !slot_names.insert(name.clone()) {
                    return Err(format!(
                        "Operation '{}' contains duplicate argument slots",
                        operation.name
                    ));
                }
            }
        }
        for verb in &operation.verbs {
            let word = verb.to_lowercase();
            if word.is_empty() {
                return Err(format!(
                    "Operation '{}' contains an empty verb",
                    operation.name
                ));
            }
            if !seen.insert(word.clone()) {
                return Err(format!(
                    "Duplicate term '{}' in domain '{}'",
                    word, domain.name
                ));
            }
            entries.push((word, true));
        }
    }

    Ok(entries)
}

