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
                    │          │
                    │          └──► Entity Table / references
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

`Vocabulary::English` maps surface forms into stable lexical categories. In 0.2 this includes artifact words, element words, pronouns and a deliberately constrained color vocabulary.

### Lexer

The lexer owns character-level concerns and source coordinates. It assigns lexical categories but never resolves references.

### Parser — recursive descent

The parser is handwritten on purpose. It outputs syntax-oriented AST nodes such as `CreateWeb`, `AddElement`, `PronounReference` and `SetProperty`.

A key boundary is that parsing `Make it blue.` produces a reference node; it does **not** decide what `it` points to.

### AST — Visitor

AST nodes expose `accept(visitor)`. Semantic resolution and serialization use visitors so that operations over syntax remain separate from the syntax data structures.

### Semantic resolver

This layer owns deterministic inference. It resolves defaults, constructs entities and delegates contextual references to `Semantics::EntityTable`.

The resolver must reject ambiguity rather than use heuristics that could silently change program meaning.

### Entity table

`EntityTable` is the initial symbol/reference infrastructure. It assigns stable per-kind IDs (`button-1`, `image-1`) and resolves constrained pronouns.

The current `it` rule is intentionally conservative: exactly one addressable entity must exist. Later named references and lexical scopes can extend this component without adding HTML knowledge to the parser.

### IR

`IR::WebDocument` contains resolved document meaning and an immutable list of `IR::Element` values. Elements carry semantic properties, not backend markup.

For example, `Make it blue.` ultimately becomes an element property:

```text
button-1.properties[:color] = "blue"
```

represented immutably in the IR.

### Backends — Strategy + Registry

Backends consume the IR. The HTML backend renders semantic buttons/images and properties, but does not resolve pronouns or interpret source phrases.

## Dependency direction

```text
CLI → Compiler → Lexer / Parser / Semantics → IR ← Backends
```

Avoid dependencies in the other direction. In particular:

- AST must not depend on HTML.
- IR must not depend on parser tokens.
- Backends must not parse source text.
- Reference resolution must not live in a backend.
- Core semantics must not depend on the CLI.

## Why no LLM in the compiler core?

The reference compiler is deterministic. An optional future free-form-language adapter could translate unrestricted prose into strict Semauri source, but that adapter would sit before the compiler boundary:

```text
free-form language → optional adapter → strict Semauri → compiler
```

The resulting Semauri source would still be parsed, checked and compiled deterministically.
