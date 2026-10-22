## 0.17.0 — 2026-10-22

### Added: lowercase language workflow

- The canonical spelling of all Semauri source commands and registered domain actions is lowercase; parsing remains case-insensitive for backward compatibility.
- `semauri fmt FILE` prints canonical lowercase source, `--write` rewrites it, and `--check` returns a nonzero exit status for formatting drift.
- Converted all 19 official examples and 22 beginner-course programs to lowercase language syntax, with automated conformance checks; documented code samples in both course languages now follow the style.

### Added: typed language features

- Pure Boolean `assert` statements, source-aware runtime assertion failures (`S502`), and the `semauri test PATH` command with strict effect controls.
- `length of` expressions for Unicode strings and typed lists, typed `contains` substring/list-membership comparisons, zero-based `list at index` access, and immutable same-type list concatenation with `plus`.
- Machine-readable `check --json` diagnostics while retaining human-readable defaults and existing inspector JSON outputs.

### Added: ML compiler and analysis infrastructure

- Validated typed SSA dataflow graphs via `semauri graph FILE` and `Compiler::runtime_graph`; graphs describe ML dependencies and reject invalid producer references.
- Version-qualified dataset/model input declarations: `source dataset ... revision ...` and `resolve model ... revision ...`.
- `semauri audit FILE [--strict]` for advisory ML provenance/seed checks; strict mode can reject incomplete declarations in CI.

### Fixed and improved

- File-only deletion semantics in native filesystem execution; recursive deletion of directories is refused.
- Preflight validation of filesystem operation arguments and names.
- Regression tests and CI gates for shipped examples, learning programs, language operations, ML graph integrity and reproducibility declarations.

### Compatibility and limitations

- Previously valid capitalized `.sema` programs still compile. The canonical lowercase convention is a source-style change, not a breaking grammar migration.
- Quoted strings, user-defined names and labels, comments, Rust APIs, generated output case and JSON keys are not blanket-lowercased.
- ML revisions are *declared*, not artifact-verified. Semauri still produces ML workflow plans rather than executing model training.
- User-defined functions and general dynamic runtime control flow are not part of this release.

See [v0.17.0 release notes](docs/releases/0.17.0.md).

# Changelog

All notable changes to Semauri are documented here.

## 0.16.1 — 2026-10-12

### Changed

- **Learn to Code is now bilingual:** opening `learn_to_code/` displays a language selector linking to complete `es/` and `en/` courses.
- Moved all 11 original Spanish lessons to `learn_to_code/es/` while keeping their programs and exercises.
- Added 11 equivalent English lessons under `learn_to_code/en/`, each with a translated guide, challenge and working Semauri program.
- Updated documentation, installation instructions, packaging smoke tests and learning-course regression tests for both languages.
- Release packages now include both language trees on Linux, macOS and Windows.

## 0.16.0 — 2026-10-12

### Added

- An 11-lesson beginner course in Spanish, with runnable `.sema` programs and challenges.
- `Unless`, `Otherwise if` / `Else if`, and Unicode scalar string escapes.
- Beginner course packaged with native compiler distributions.

## 0.15.0 — 2026-10-08

### Added

- `//` single-line comments and nestable `/* ... */` multi-line comments.
- Readable numeric literals with `_` digit separators and scientific `e` / `E` notation.

### Fixed

- Unterminated block comments produce a lexical diagnostic (`S105`).
- Invalid numeric separators, malformed exponents, integers outside the signed 64-bit range, and non-finite numeric literals produce `S106` rather than crashing the compiler.
- Comment delimiters inside strings remain ordinary text.

### Validation

- Language contract regressions added for both features.
- GitHub Actions checks Rust tests and builds/smoke-tests six native distribution packages.

## 0.14.0 — 2026-10-07

### Changed

- The reference compiler and CLI are implemented entirely in Rust.
- Semantic-domain registration and backend registration are public Rust extension APIs.
- External declarative domains can define artifacts, elements, typed properties, nominal types and deterministic operations without parser-specific cases.
- Runtime plans, effects/capabilities, optimization, filesystem execution and ML planning remain available through the native compiler.
- Official packaging builds one native executable for Linux, macOS and Windows on x86_64 and arm64.
- The source tree, tests, documentation and distribution pipeline are Rust-only.

### Validation

- Language, HIR, optimizer, runtime, schema, filesystem, ML and extension contracts are covered by Rust tests.
- CI checks the Rust-only repository invariant.
- Release artifacts are built and smoke-tested natively on all six supported target/architecture combinations.

## Earlier releases

Detailed release notes for earlier language milestones remain under `docs/`:

- [0.13](docs/releases/0.13.md)
- [0.12](docs/releases/0.12.md)
- [0.11](docs/releases/0.11.md)
- [0.9](docs/releases/0.9.md)
