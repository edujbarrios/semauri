# Semauri architecture

Semauri follows a compiler pipeline with strict boundaries. The goal is to make language evolution reviewable and to let contributors work on one layer without understanding every backend.

## Pipeline

```text
Source
  │
  ▼
Vocabulary ──► Lexer ──► Tokens
                         │
                         ▼
                       Parser
                         │
                         ▼
                        AST
                         │
                         ▼
                 Semantic Resolver
                         │
                         ▼
                         IR
                         │
                         ▼
                 Backend Registry
                         │
                         ▼
                  Target Artifact
```

## Components

### Vocabulary — Strategy

`Vocabulary::English` maps surface forms (`create`, `make`, `website`, `web`) into stable lexical categories. The lexer receives a vocabulary object through dependency injection.

This makes future controlled-language surfaces possible without coupling every translation to the parser.

### Lexer

The lexer owns character-level concerns and source coordinates. It never assigns contextual meaning.

### Parser — recursive descent

The parser is handwritten on purpose. Semauri is currently small enough that a recursive-descent parser keeps the grammar understandable to contributors and makes diagnostics easy to control.

The parser outputs syntax-oriented AST nodes such as `CreateWeb` and `SetTitle`.

### AST — Visitor

AST nodes expose `accept(visitor)`. Semantic resolution and serialization use visitors so that operations over syntax are separated from the data structure itself.

When a new AST node is introduced, visitors fail explicitly until they support the node instead of silently generating output incorrectly.

### Semantic resolver

This is where natural-looking syntax obtains deterministic meaning.

For example:

```text
Create a web for a pet store.
```

parses with a `subject` but no explicit `title`. The resolver applies the documented `WebDocument` rule `title := subject` and records an explanation for that decision.

The resolver should reject ambiguity rather than choose probabilistically.

### IR

The intermediate representation contains resolved program meaning. `IR::WebDocument` does not know whether its title came from `called`, `for`, a future Spanish surface syntax, or another parser construct except for optional provenance metadata useful to diagnostics.

This boundary is essential for multiple backends.

### Backends — Strategy + Registry

Backends implement a small rendering interface. `Backends::Registry` maps a backend name to a factory, allowing embedding applications and future plugins to register a backend without changing `Compiler`.

Today there is one backend: HTML.

## Dependency direction

Preferred dependency direction:

```text
CLI → Compiler → Lexer / Parser / Semantics → IR ← Backends
```

Avoid dependencies in the other direction. In particular:

- AST must not depend on HTML.
- IR must not depend on parser tokens.
- Backends must not parse source text.
- Core semantics must not depend on the CLI.

## Why no LLM in the compiler core?

The reference compiler is deterministic. An optional future free-form-language adapter could use an LLM to translate unrestricted prose into strict Semauri source, but that adapter would sit *before* the compiler boundary:

```text
free-form language → optional adapter → strict Semauri → compiler
```

The generated Semauri source would still be parsed, checked and compiled deterministically.
