## 0.21.0 — 2026-10-23

### Language: typed Boolean reductions
- Add `any of` and `all of` on `list<boolean>` with static type checking (`S356`), no new effects, integration tests, and a runnable example. `any` and `all` are newly reserved keywords. [PR #160](https://github.com/edujbarrios/semauri/pull/160).

### Language: typed numeric median
- Add `median of` for finite `list<number>` with checked type constraints (`S353`) and guarded empty/non-finite evaluation (`S357`). Exact integral medians and even integer midpoints preserve i64 precision using i128 intermediate sums; decimals follow f64 rounding. `median` is newly reserved. [PR #161](https://github.com/edujbarrios/semauri/pull/161).
- Fix existing unary `minus` integer precision loss in both execution and constant folding; preserve exact i64 values and diagnose signed negation overflow (`S358`).
- Include runnable examples, integration/regression tests, updated language reference, and release packaging documentation.

### Boundaries
- These operations are pure and work on concrete core-language lists, not tensors or runtime ML dataset handles. This release does not introduce model-training execution.
- Existing mixed/uppercase language syntax remains compatible, apart from the newly reserved words.

See [v0.21.0 release notes](docs/releases/0.21.0.md).

## 0.20.0 — 2026-10-23

### Typed numeric extrema
- Add `minimum of` and `maximum of` for `list<number>`, with static `S353` type errors, empty/non-finite `S355` evaluation diagnostics and no new effects. Exact integer-only comparisons preserve `i64` extrema beyond 2^53; mixed decimal/integer comparisons follow current `f64` semantics. [PR #157](https://github.com/edujbarrios/semauri/pull/157).
- Add integration tests for type checking, fractional and negative numbers, precision, formatting, composition, and effect analysis; include a runnable example and language specification.

### AI engineering reproducibility and portfolio evidence
- Add a documented ML compiler case study and a CI contract against the checked-in versioned ML example, covering deterministic typed dependency graphs, declaration audits and capability enforcement. [PR #158](https://github.com/edujbarrios/semauri/pull/158).
- Clarify that ML workflow planning, revision declarations, effect analysis and reproducibility preflight do not implement model training or prove external artifact integrity.

### Compatibility
- The new reserved words `minimum` and `maximum` cannot be used as bare variable identifiers. Other existing language constructs and case-insensitive command parsing remain supported.
- Native builds continue to target six Linux/macOS/Windows platform architectures and include SHA256 checksums on release.

See [v0.20.0 release notes](docs/releases/0.20.0.md).

## 0.19.0 — 2026-10-22

### Typed sequence and data processing

- Add `reverse of` for concrete strings and immutable homogeneous lists, maintaining list element types and leaving original inputs unchanged. Literal strings can be reversed during constant folding; string reversal operates on Unicode scalar values.
- Add `sum of` and `average of` for `list<number>`, with static type checking (`S353`), exact checked integer sums, decimal arithmetic for averages, and located numeric overflow/non-finite errors (`S354`).
- Introduce explicit reverse type diagnostic `S352`, integrate new keywords into the canonical lowercase formatter, and add integration tests and runnable examples for both features.
- Refresh the README, language grammar and documentation navigation to include new collection operations and their precision limits.

### Compatibility and limitations

- Existing uppercase/mixed-case Semauri programs remain accepted. `reverse`, `sum` and `average` are newly reserved language keywords; use different names for user identifiers that previously used those words.
- Numeric aggregation operates on concrete typed lists; the reference compiler does not compute over runtime-produced ML tensors or datasets.
- No new effects, external dependencies or framework execution capabilities are introduced.

See [v0.19.0 release notes](docs/releases/0.19.0.md).

## 0.18.0 — 2026-10-22

### Language

- Add typed `uppercase of` and `lowercase of` string expressions, with deterministic Unicode case mapping, compile-time type checking (`S351`), constant folding, runtime lowering and no additional effect capabilities.
- Support new string operations in the canonical lowercase source formatter, examples and regression tests.

### Getting started and documentation

- Present Semauri prominently as a statically typed programming language with an independent Rust compiler, native CLI and domain-specific build/execution backends.
- Refresh the primary project messages: **“A programming language for readable, typed and predictable software”** and **“Write clearly. Compile confidently. Build with intent.”**
- Add OS-specific installation and first-program instructions, including how to open PowerShell/Terminal, create and edit a `.sema` file, check and run it, resolve `PATH` issues and understand explicit capabilities.
- Expand the language reference and installation troubleshooting without changing the English-language Semauri syntax or the existing Spanish course.

### Compatibility

- Existing programs remain valid, including uppercase and mixed-case source commands. No new runtime permissions or dependencies are required.
- This release does not change the distinction between executable supported core operations and declarative ML workflow plans.

See [v0.18.0 release notes](docs/releases/0.18.0.md).

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
