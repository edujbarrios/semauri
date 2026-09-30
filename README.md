# Semauri

**Natural to write. Deterministic to run.**

Semauri is an experimental deterministic programming language that explores how close programming can get to human language without asking a probabilistic model to guess what a program means.

Created by **Eduardo J. Barrios**. Open source under the **Apache License 2.0**.

> Status: **0.4.x / experimental**. Semauri already has a lexer, handwritten parser, typed expressions, lexical scopes, stable semantic symbols, deterministic references and conditional control flow. The language remains intentionally small while the compiler architecture matures.

## Philosophy

Semauri uses **controlled natural language** rather than unrestricted prose.

- **Deterministic semantics** — ambiguity is a compiler error, not a request to guess.
- **No LLM required** — the reference compiler is local and deterministic.
- **Explainable resolution** — `semauri explain` exposes semantic decisions.
- **Compiler tooling first** — tokens, AST, symbols and diagnostics are inspectable.
- **Extensible boundaries** — vocabularies, semantics, IR and backends are separate layers.

## Example

```text
Let basePrice be 18.
Let tax be 4.
Let total be basePrice plus tax.

Create a web called Pet Shop.
Add a button called Buy.

If total is greater than 20 and not false:
  Let accent be red.
  Set the color of the button called Buy to accent.
Otherwise:
  Set the color of the button called Buy to green.
End.
```

Build it:

```bash
ruby bin/semauri build shop.sema -o shop.html
```

Inspect compiler stages:

```bash
ruby bin/semauri tokens shop.sema
ruby bin/semauri ast shop.sema
ruby bin/semauri symbols shop.sema
ruby bin/semauri explain shop.sema
ruby bin/semauri check shop.sema
```

## Current language surface

Semauri currently supports:

- web documents, buttons and images
- explicit and constrained-pronoun references
- colors, strings, numbers and booleans
- immutable `Let` bindings
- lexical shadowing
- arithmetic with precedence and parentheses
- typed comparisons
- `and`, `or`, `not` with short-circuit semantics
- `If / Otherwise / End`
- canonical property assignment such as `Set the color of ... to ...`

Current `If` branches are selected during semantic analysis because all current values are compile-time-known. Runtime control flow will require a dedicated control-flow IR rather than silently changing this model.

## Compiler architecture

```text
source text
    ↓
vocabulary + lexer
    ↓
tokens
    ↓
recursive-descent parser
    ↓
syntax AST
    ↓
semantic scopes + symbol table + expression typing
    ↓
semantic/domain IR
    ↓
backend registry
    ↓
HTML (today), other targets later
```

A variable name is not its identity. Bindings receive stable semantic symbol IDs, so shadowed names remain distinct and tooling can inspect definitions independently of source spelling.

The next compiler architecture milestone is a **typed HIR** between syntax AST and target/domain IR. See [docs/ROADMAP.md](docs/ROADMAP.md).

## CLI

```text
semauri tokens FILE
semauri ast FILE
semauri symbols FILE
semauri explain FILE
semauri check FILE
semauri build FILE [-o PATH]
semauri version
```

During development, invoke the CLI with `ruby bin/semauri ...`.

## Development

Requirements:

- Ruby 3.2+
- no runtime gems for the compiler core

Run tests:

```bash
ruby -Ilib -e 'Dir["test/test_*.rb"].sort.each { |file| require_relative file }'
```

CI runs syntax checks and the complete suite on Ruby 3.2, 3.3 and 3.4.

## Design principles

1. **Specification before convenience.** New syntax needs explicit semantics.
2. **Ambiguity is an error.** The compiler never probabilistically chooses a meaning.
3. **Names and symbols are different concepts.** Semantic identity survives shadowing and future tooling passes.
4. **Small core, explicit extension boundaries.** Backends and domains should not monkey-patch the parser.
5. **No semantic work in code generators.** Resolution and typing happen before rendering.
6. **Tests are executable language documentation.** Positive and negative behavior belongs in the suite.
7. **Compatibility matters.** Existing surface syntax should lower through common semantic machinery.

## Contributing

Contributions are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request. Language changes should include tests and a specification/roadmap update.

## License and attribution

Copyright 2026 Eduardo J. Barrios.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
