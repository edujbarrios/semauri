# Changelog

All notable changes to Semauri will be documented in this file.

## Unreleased

### Added
- Semantic entities for generated web elements.
- Deterministic pronoun resolution for `it`.
- Ambiguity diagnostics for references.
- Button and image elements.
- Property mutation with natural syntax such as `Make it blue.`.
- HTML rendering for semantic elements.

### Changed
- `make` is now parsed contextually: it can create an artifact (`Make a web...`) or mutate an existing entity (`Make it blue.`).
