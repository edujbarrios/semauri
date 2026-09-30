# Changelog

All notable changes to Semauri will be documented in this file.

## Unreleased

### Added
- Deterministic semantic entities and explicit/pronoun reference resolution.
- Source spans, source-aware diagnostics and mutation provenance.
- Typed literals, immutable `Let` bindings and lexical scopes.
- Arithmetic, comparisons, `If / Otherwise / End` and short-circuit boolean logic.
- Stable semantic symbols with unique IDs, kinds, types and definition spans.
- `semauri symbols FILE` introspection.
- Typed HIR nodes that preserve program structure without executing control flow.
- HIR symbol references carrying stable symbol IDs and types.
- `semauri hir FILE` JSON introspection.
- Shared `Semantics::TypeSystem` rules used by evaluation and HIR construction.

### Changed
- `make` is parsed contextually as creation or mutation.
- Property values are AST expressions before semantic resolution.
- Expression evaluation is isolated in `Semantics::ExpressionEvaluator`.
- Conditional branches in the current executable lowering are selected at semantic-analysis time while values remain compile-time known.
- Scope bindings separate source names, semantic symbols and values.
- HIR construction is a separate structural phase; it preserves both branches and does not perform branch selection/constant folding.
