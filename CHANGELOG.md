# Changelog

All notable changes to Semauri will be documented in this file.

## Unreleased

### Added
- Semantic entities for generated web elements.
- Deterministic pronoun resolution for `it`.
- Ambiguity diagnostics for references.
- Explicit named references such as `Make the button called Buy blue.`.
- Button and image elements.
- Property mutation with natural syntax such as `Make it blue.`.
- HTML rendering for semantic elements.
- Full source spans on lexer tokens and AST nodes using exclusive end positions.
- Source-aware compiler diagnostics with caret/range excerpts.
- Per-property semantic provenance in the IR.
- Typed literal expressions for colors, strings, numbers and booleans.
- Immutable `Let` variable bindings and variable references.
- Scope-based name resolution with duplicate/unknown-variable diagnostics.
- Canonical property assignment syntax: `Set the color of ... to ...`.
- Semantic type checking for property values.
- First-class lexical `Block`, `BinaryExpression` and `UnaryExpression` AST nodes.
- Arithmetic expressions with precedence and parentheses.
- Strict equality and numeric ordering comparisons.
- Deterministic `If / Otherwise / End` control flow.
- Child lexical scopes for conditional blocks.
- Logical `and`, `or`, and `not` expressions with short-circuit evaluation.
- Stable semantic symbols with unique IDs, kind, type and definition spans.
- Shared symbol tables across lexical child scopes.
- `semauri symbols FILE` compiler introspection command.

### Changed
- `make` is parsed contextually: it can create an artifact or mutate an existing entity.
- AST JSON output includes source spans for tooling consumers.
- Property values are represented as AST expressions before semantic resolution.
- Expression evaluation is isolated in `Semantics::ExpressionEvaluator`.
- Conditional branches are currently selected at semantic-analysis time while all values are compile-time known.
- Scope bindings now separate source names, semantic symbols and runtime/constant values.
- Variable-resolution explanations include the resolved symbol identity.
