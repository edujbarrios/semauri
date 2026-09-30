# Semauri

**Natural to write. Deterministic to run.**

## About

Semauri is an experimental programming language that lets people write deterministic programs using controlled natural language.

It is not an LLM wrapper and it does not ask AI to guess what the user meant. Semauri parses a defined language, builds compiler structures, resolves meaning, checks types and then lowers the program to a target backend.

Created by **Eduardo J. Barrios** and open sourced from the beginning under the **Apache License 2.0**.

> Status: **0.4.x / experimental**

## Example

```text
Let price be 18.
Let tax be 4.
Let total be price plus tax.

Create a web called Pet Shop.
Add a button called Buy.

If total is greater than 20:
  Set the color of the button called Buy to red.
Otherwise:
  Set the color of the button called Buy to green.
End.
```

Compile it:

```bash
ruby bin/semauri build shop.sema -o shop.html
```

## How Semauri was built

The reference compiler is written in **Ruby** and intentionally avoids runtime dependencies in its core.

It started from a very small handwritten compiler and has been grown layer by layer:

```text
source
  ↓
lexer
  ↓
tokens
  ↓
recursive-descent parser
  ↓
AST
  ↓
scopes + symbols + type checking
  ↓
semantic IR
  ↓
backend
```

Important implementation choices:

- handwritten recursive-descent parser
- immutable AST/semantic objects where practical
- lexical scopes and stable semantic symbol IDs
- typed expressions and strict comparisons
- deterministic reference resolution
- explicit ambiguity errors
- source spans and compiler diagnostics
- visitor, strategy, registry and dependency-injection patterns
- tests used as executable language specification

The compiler is being developed incrementally so each stage remains understandable and replaceable. The next architectural step is a typed HIR between the syntax AST and lower-level/domain IR.

## Current features

- numbers, strings, booleans and colors
- immutable `Let` bindings
- arithmetic and parentheses
- typed comparisons
- `and`, `or`, `not` with short-circuit evaluation
- lexical scopes and shadowing
- stable semantic symbols
- `If / Otherwise / End`
- web documents, buttons and images
- explicit references and constrained `it` resolution
- HTML backend
- source-aware diagnostics

## Inspect the compiler

Semauri exposes its internal stages instead of hiding them:

```bash
ruby bin/semauri tokens shop.sema
ruby bin/semauri ast shop.sema
ruby bin/semauri symbols shop.sema
ruby bin/semauri explain shop.sema
ruby bin/semauri check shop.sema
```

## Development

Requires Ruby 3.2+.

```bash
git clone https://github.com/edujbarrios/semauri.git
cd semauri
ruby -Ilib -e 'Dir["test/test_*.rb"].sort.each { |file| require_relative file }'
```

CI tests Ruby 3.2, 3.3 and 3.4.

## Project principles

- natural syntax does not mean ambiguous semantics
- the compiler must fail instead of guessing
- no LLM is required by the deterministic compiler core
- semantic work happens before code generation
- new language features require tests and documented semantics
- extension points should be explicit and maintainable

See [docs/LANGUAGE.md](docs/LANGUAGE.md), [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) and [docs/ROADMAP.md](docs/ROADMAP.md).

## Contributing

Pull requests are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Copyright 2026 Eduardo J. Barrios.

Licensed under the **Apache License 2.0**. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
