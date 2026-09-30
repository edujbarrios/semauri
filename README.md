# Semauri

**Natural to write. Deterministic to run.**

Semauri is an experimental, deterministic programming language that explores how close programming can get to human language without asking a probabilistic model to guess what the program means.

Semauri is open source from its first commit. It was created by **Eduardo J. Barrios** and is licensed under the **Apache License 2.0**.

> Status: **0.1 / experimental**. The language is intentionally tiny while its semantics and architecture are being established.

## Why Semauri?

Most natural-language programming experiments either become unrestricted prose or delegate meaning to an LLM. Semauri takes a different approach:

- **Controlled natural language** — source code should read naturally, but accepted syntax has a specification.
- **Deterministic semantics** — the same valid program has the same meaning every time.
- **Explainable inference** — implicit decisions are inspectable with `semauri explain`.
- **No LLM required** — the reference compiler does not need network access or a model.
- **Extensible by design** — surface vocabularies, semantic analysis and output backends are separate layers.

## First program

Create `shop.sema`:

```text
Create a web for a pet store.
```

Build it:

```bash
ruby bin/semauri build shop.sema -o shop.html
```

Semauri resolves `web` to the default web target (HTML), interprets `pet store` as the subject, and because no title was explicitly given, derives the title from that subject.

```html
<!doctype html>
<html lang="en">
<head>
  ...
  <title>Pet Store</title>
</head>
<body>
  <h1>Pet Store</h1>
</body>
</html>
```

You can inspect the decision instead of trusting hidden magic:

```bash
ruby bin/semauri explain shop.sema
```

```text
1. 'web' resolved to an HTML web document (default web backend).
2. Subject resolved to 'Pet Store'.
3. No title was provided, so the web title defaults to its subject: 'Pet Store'.
```

## Current language surface

Semauri 0.1 intentionally supports only a few constructs:

```text
Create a web for a pet store.
Create a web called Hello World.

Create a web for a pet store.
Add a title called Happy Paws.
```

Synonyms such as `Make`, `website` and `named` are normalized by the English vocabulary layer.

## CLI

```text
semauri tokens FILE
semauri ast FILE
semauri explain FILE
semauri check FILE
semauri build FILE [-o PATH]
semauri version
```

During development, invoke the CLI with `ruby bin/semauri ...`. Packaging as a Ruby gem is planned once the core language surface stabilizes.

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
semantic resolver
    ↓
IR
    ↓
backend registry
    ↓
HTML (today), other targets later
```

The important rule is that **backends do not parse natural language**. They consume Semauri's internal representation. Likewise, the parser does not generate HTML. This separation keeps future PRs local and reviewable.

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) and [docs/LANGUAGE.md](docs/LANGUAGE.md).

## Development

Requirements:

- Ruby 3.2+
- no runtime gems for the compiler core

Run the test suite:

```bash
ruby -Ilib -e 'Dir["test/test_*.rb"].sort.each { |file| require_relative file }'
```

Run syntax checks:

```bash
find lib test bin -name '*.rb' -o -path 'bin/semauri' | xargs -n1 ruby -c
```

## Design principles

1. **Specification before convenience.** New syntax should have documented semantics.
2. **Explicit ambiguity errors.** The compiler must reject ambiguity rather than silently guess.
3. **Small core, extension points at boundaries.** Backends and vocabularies are registered/injected.
4. **No semantic work in code generators.** Inference belongs to semantic analysis.
5. **Tests are executable language documentation.** Every language feature needs positive and negative tests.
6. **Compatibility matters.** Breaking syntax/semantic changes will be documented as the language matures.

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request. In particular, language changes should include a specification update and tests.

## Roadmap

The near-term roadmap is deliberately language-oriented rather than feature-oriented:

- richer diagnostics and source spans
- references and pronouns with explicit ambiguity handling
- variables and scope
- conditions and iteration
- a type/constraint system
- semantic domains beyond web
- additional output backends
- formatter and language server
- additional controlled-natural-language surface vocabularies

See [docs/ROADMAP.md](docs/ROADMAP.md) for milestones.

## License and attribution

Copyright 2026 Eduardo J. Barrios.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
