// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

// All operations in this domain are backed by the operating system's
// cryptographically secure random source and require explicit permission.
fn random_domain() -> DomainSpec {
    DomainSpec {
        name: "random".to_string(),
        default_backend: None,
        artifacts: BTreeMap::new(),
        elements: BTreeMap::new(),
        properties: BTreeMap::new(),
        types: Vec::new(),
        operations: vec![
            op(
                "password",
                &["password"],
                vec![lit("of"), slot("count", Type::Number), lit("characters")],
                Type::String,
                &["secure_random"],
            ),
            op(
                "secure_string",
                &["secure"],
                vec![
                    lit("random"),
                    lit("string"),
                    lit("of"),
                    slot("count", Type::Number),
                    lit("characters"),
                    lit("from"),
                    slot("alphabet", Type::String),
                ],
                Type::String,
                &["secure_random"],
            ),
            op(
                "integer",
                &["draw"],
                vec![
                    lit("random"), lit("integer"), lit("from"),
                    slot("minimum", Type::Number),
                    lit("to"),
                    slot("maximum", Type::Number),
                ],
                Type::Number,
                &["secure_random"],
            ),
            op(
                "character",
                &["pick"],
                vec![
                    lit("random"), lit("character"), lit("from"),
                    slot("alphabet", Type::String),
                ],
                Type::String,
                &["secure_random"],
            ),
        ],
    }
}
