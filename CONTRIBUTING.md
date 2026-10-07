# Contributing to Semauri

Thank you for helping build Semauri. The project is intentionally designed so contributors can add capabilities without turning the compiler into a collection of special cases.

## Before opening a PR

For bug fixes, include a regression test. For language changes, include the motivating program, intended AST/HIR/semantic meaning, ambiguity/error cases, documentation updates, compiler tests, and backend tests when target generation changes.

## Architectural boundaries

Keep these boundaries intact:

- **Semantic Domains** own application-specific vocabulary, property contracts and domain IR construction.
- **Vocabulary/Lexer** produce deterministic tokens and source spans.
- **Parser** produces syntax AST.
- **Typed HIR** owns resolved symbol identity and type-checked structure.
- **HIR optimization** rewrites semantic structure without rendering targets.
- **HIR lowering** evaluates compile-time-known constructs and creates domain/runtime plans.
- **Domain IR** represents resolved artifact meaning.
- **Backends** render domain IR into target formats.
- **Effects/capabilities** remain explicit and are validated before execution.

A new semantic domain should avoid special-casing the lexer/parser where possible. A new backend should not require grammar changes.

## Rust development

The reference compiler is a Rust crate. Rust 1.83+ is required for development:

```bash
cargo test --all-targets
cargo run -- help
cargo run -- build examples/shop.sema
```

Behavior changes require tests. Keep source diagnostics and CLI exit-code contracts stable unless the change explicitly intends to revise them.

## Extension patterns

New domains belong in the domain registry/specification layer and must include negative tests for invalid values and ambiguity. New backends should render semantic IR without embedding grammar logic. New optimization passes must preserve observable behavior, including compiler/runtime errors that can trap.

## Style

Prefer small, explicit data structures; avoid global mutable state; keep public APIs documented; use descriptive diagnostic codes; and reject ambiguity instead of silently guessing.

## License

Unless explicitly stated otherwise, contributions submitted to Semauri are accepted under the Apache License 2.0, consistent with Section 5 of that license.
