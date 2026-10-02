# Contributing to Semauri

Thank you for helping build Semauri. The project is intentionally designed so contributors can add capabilities without turning the compiler into a collection of special cases.

## Before opening a PR

For bug fixes, include a regression test.

For a language change, please include:

1. the motivating program;
2. the intended AST/HIR/semantic meaning;
3. ambiguity and error cases;
4. an update to `docs/01_LANGUAGE.md` when language behavior changes;
5. compiler tests;
6. backend tests when target generation changes.

Large syntax or semantic changes should start as a GitHub issue so design trade-offs can be discussed before implementation.

## Architectural boundaries

Keep these boundaries intact:

- **Semantic Domains** own application-specific vocabulary, property contracts and domain IR construction.
- **Vocabulary** maps core surface words plus registered domain terms to lexical categories.
- **Lexer** produces tokens and source locations.
- **Parser** produces syntax AST; domain nouns use generic artifact/element/property categories.
- **Typed HIR** owns resolved symbol identity and type-checked structure.
- **HIR optimization** rewrites semantic structure without rendering targets.
- **HIR lowering** evaluates compile-time-known constructs and dispatches domain operations.
- **Domain IR** represents resolved artifact meaning.
- **Backends** render domain IR into target formats.

A new semantic domain should not require changes to the lexer or parser. A new backend should not require grammar changes.

## Extension patterns

### Add a semantic domain

Read [`docs/03_DOMAINS.md`](docs/03_DOMAINS.md). New domains should extend `Semauri::Domains::Definition`, register their vocabulary through `Domains::Registry`, define property types and produce their own semantic IR where appropriate.

A domain PR must include negative tests for invalid values and ambiguity, not only a happy-path example.

### Add a backend

Implement `Semauri::Backends::Base#render`, then register it in a backend registry. Keep backend-specific logic out of AST, HIR and semantic domains unless it is genuinely semantic rather than representational.

### Add core syntax

Core syntax is intentionally smaller than domain vocabulary. Add grammar only for concepts that should exist across domains. Introduce a dedicated AST/HIR representation where appropriate and document deterministic error behavior.

### Add an optimization pass

Implement a pass that accepts and returns Typed HIR, register it with `HIR::Optimization::PassManager`, and add equivalence tests showing optimized and unoptimized compilation preserve observable semantics. Passes must not hide potentially observable compiler/runtime errors.

## Testing

```bash
ruby -Ilib -e 'Dir["test/test_*.rb"].sort.each { |file| require_relative file }'
```

All behavior changes require tests. CI currently validates Ruby 3.2, 3.3 and 3.4. The core intentionally depends only on the Ruby standard library.

## Style

- Prefer small objects with one responsibility.
- Prefer dependency injection at extension boundaries.
- Avoid global mutable state.
- Prefer immutable compiler/domain data structures where practical.
- Do not introduce a dependency for something the standard library handles clearly.
- Keep public APIs documented.
- Use descriptive diagnostic codes for user-facing compiler errors.
- Reject ambiguity instead of adding heuristics that silently change program meaning.

## License

Unless explicitly stated otherwise, contributions submitted to Semauri are accepted under the Apache License 2.0, consistent with Section 5 of that license.
