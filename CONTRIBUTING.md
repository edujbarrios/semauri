# Contributing to Semauri

Thank you for helping build Semauri. The project is intentionally designed so that contributors can add capabilities without turning the compiler into a collection of special cases.

## Before opening a PR

For bug fixes, include a regression test.

For a language change, please include all of the following:

1. the motivating example program;
2. the intended AST/semantic meaning;
3. ambiguity and error cases;
4. an update to `docs/LANGUAGE.md`;
5. parser/semantic tests;
6. backend tests only when code generation changes.

Large syntax or semantic changes should start as a GitHub issue so design trade-offs can be discussed before implementation.

## Architectural boundaries

Please keep these boundaries intact:

- **Vocabulary** maps surface words to lexical token categories.
- **Lexer** produces tokens and source locations.
- **Parser** produces AST nodes; it does not choose output formats.
- **Semantic resolver** handles contextual defaults and meaning.
- **IR** represents resolved program meaning independent of syntax.
- **Backends** render IR into target formats.

A new backend should generally not require changes to the lexer or parser. A new synonym should generally not require changes to a backend.

## Extension patterns

### Add a surface synonym

Edit or introduce a vocabulary implementation under `lib/semauri/vocabulary/` and add lexer tests.

### Add a backend

Implement `Semauri::Backends::Base#render`, then register it in a registry. Avoid putting backend-specific logic into the parser.

### Add syntax

Add the smallest required token/grammar changes, a dedicated AST node where appropriate, semantic handling via the visitor interface, and tests.

## Testing

```bash
ruby -Ilib -e 'Dir["test/test_*.rb"].sort.each { |file| require_relative file }'
```

All behavior changes should be covered by tests. The core currently depends only on the Ruby standard library so that contributors can run it with a stock Ruby installation.

## Style

- Prefer small objects with one responsibility.
- Prefer dependency injection at extension boundaries.
- Avoid global mutable state.
- Do not introduce a dependency for something the standard library handles clearly.
- Keep public APIs documented.
- Use descriptive diagnostic codes for user-facing compiler errors.

## License

Unless explicitly stated otherwise, contributions submitted to Semauri are accepted under the Apache License 2.0, consistent with Section 5 of that license.
