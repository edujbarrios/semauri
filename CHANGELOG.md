# Changelog

All notable changes to Semauri will be documented in this file.

## Unreleased

### Added
- Deterministic semantic entities and explicit/pronoun reference resolution.
- Source spans, source-aware diagnostics and mutation provenance.
- Typed literals, immutable `Let` bindings and lexical scopes.
- Arithmetic, comparisons, conditional control flow and short-circuit boolean logic.
- Stable semantic symbols and compiler symbol introspection.
- Typed HIR with stable symbol references and `semauri hir` introspection.
- Shared type rules for evaluation and HIR construction.
- Homogeneous natural-language list literals such as `a list of 10, 20, 30`.
- Parametric `list<T>` semantic types.
- Static `For every ... in ...` iteration with lexical iterator scope.
- HIR-preserved `for_each` nodes and stable iterator symbol IDs.
- Canonical README example stored in `examples/shop.sema` and covered by tests.

### Changed
- Scope bindings now support binding an existing semantic symbol to different compile-time values without recreating symbol identity.
- Static loop execution reuses one iterator symbol across all iterations.
- HIR construction remains structural and does not unroll loops or select conditional branches.
