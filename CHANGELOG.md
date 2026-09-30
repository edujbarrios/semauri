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

### Changed
- `make` is now parsed contextually: it can create an artifact (`Make a web...`) or mutate an existing entity (`Make it blue.`).
- AST JSON output now includes source spans for tooling consumers.
