# Semauri

**Natural to write. Deterministic to run.**

Semauri is an experimental, deterministic programming language that explores how close programming can get to human language without asking a probabilistic model to guess what the program means.

Semauri is open source from its first commit. It was created by **Eduardo J. Barrios** and is licensed under the **Apache License 2.0**.

> Status: **0.2 / experimental**. The language is intentionally small while its semantics and architecture are being established.

## Why Semauri?

Most natural-language programming experiments either become unrestricted prose or delegate meaning to an LLM. Semauri takes a different approach:

- **Controlled natural language** — source code should read naturally, but accepted syntax has a specification.
- **Deterministic semantics** — the same valid program has the same meaning every time.
- **Explainable inference** — implicit decisions are inspectable with `semauri explain`.
- **Explicit ambiguity errors** — the compiler refuses to guess when a reference has multiple meanings.
- **No LLM required** — the reference compiler does not need network access or a model.
- **Extensible by design** — surface vocabularies, semantic analysis and output backends are separate layers.

## Example

Create `shop.sema`:

```text
Create a web for a pet store.
Add a button called Buy.
Make it blue.
```

Build it:

```bash
ruby bin/semauri build shop.sema -o shop.html
```

Semauri creates a web document, adds a semantic button entity and resolves `it` to that button because it is the only addressable element in scope.

You can inspect those decisions:

```bash
ruby bin/semauri explain shop.sema
```

```text
1. 'web' resolved to an HTML web document (default web backend).
2. Subject resolved to 'Pet Store'.
3. No title was provided, so the web title defaults to its subject: 'Pet Store'.
4. Added button 'Buy' as button-1.
5. 'it' resolved to button 'Buy' (button-1).
6. Set button-1.color to 'blue'.
```

If the program contains more than one possible referent, Semauri rejects the program instead of guessing:

```text
Create a web called Shop.
Add a button called Buy.
Add an image called Logo.
Make it blue.
```

produces semantic error `S305` because `it` is ambiguous.

## Current language surface

Semauri 0.2 currently supports:

```text
Create a web for a pet store.
Create a web called Hello World.
Make a web called Hello World.
Add a title called Happy Paws.
Add a button called Buy.
Add an image called Logo.
Make it blue.
```

The supported color vocabulary is intentionally constrained and deterministic.

## CLI

```text
semauri tokens FILE
semauri ast FILE
semauri explain FILE
semauri check FILE
semauri build FILE [-o PATH]
semauri version
```

During development, invoke the CLI with `ruby bin/semauri ...`.

## Architecture

```text
source text
    ↓
vocabulary + lexer
    ↓
tokens
    ↓
parser
    ↓
AST
    ↓
semantic resolver + entity table
    ↓
IR
    ↓
backend registry
    ↓
HTML (today), other targets later
```

The important rule is that **backends do not parse natural language**. References are resolved before code generation, so a backend receives explicit semantic entities rather than prose.

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) and [docs/LANGUAGE.md](docs/LANGUAGE.md).

## Development

Requirements:

- Ruby 3.2+
- no runtime gems for the compiler core

Run the test suite:

```bash
ruby -Ilib -e 'Dir["test/test_*.rb"].sort.each { |file| require_relative file }'
```

## Design principles

1. **Specification before convenience.** New syntax should have documented semantics.
2. **Explicit ambiguity errors.** The compiler must reject ambiguity rather than silently guess.
3. **Small core, extension points at boundaries.** Backends and vocabularies are registered/injected.
4. **No semantic work in code generators.** Inference belongs to semantic analysis.
5. **Tests are executable language documentation.** Every language feature needs positive and negative tests.
6. **Compatibility matters.** Breaking syntax/semantic changes are documented while the language matures.

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request. Language changes should include a specification update and tests.

## Roadmap

Next priorities include source spans, richer diagnostics, explicit named references, variables/scope, conditions, semantic domains and additional backends.

See [docs/ROADMAP.md](docs/ROADMAP.md) for milestones.

## License and attribution

Copyright 2026 Eduardo J. Barrios.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
