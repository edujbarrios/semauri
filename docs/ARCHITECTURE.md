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
                  ┌──────┴──────┐
                  ▼             ▼
          Semantic Context      IR
          / reference table      │
                                 ▼
                         Backend Registry
                                 │
                                 ▼
                          Target Artifact
```

## Components

### Vocabulary — Strategy

`Vocabulary::English` maps surface forms into stable lexical categories. It classifies words but does not make contextual semantic decisions. For example, `make` becomes `MAKE`; the parser decides whether a particular `MAKE` statement is creation compatibility syntax or a mutation.

### Lexer

The lexer owns character-level concerns and source coordinates. It never resolves pronouns or selects program entities.

### Parser — recursive descent

The handwritten parser converts controlled language into syntax-oriented AST nodes. In 0.2 the main nodes include `CreateWeb`, `SetTitle`, `CreateElement` and `SetProperty`.

### AST — Visitor

AST nodes expose `accept(visitor)`. Semantic resolution and serialization are visitors. `AST::Reference` is an immutable value object representing unresolved references such as `it`, `the button`, or `the button called Buy`.

### Semantic resolver

The semantic resolver converts syntax into deterministic meaning. It owns rules such as `title := subject` and delegates entity lookup to `Semantics::Context`.

### Semantic context — Symbol Table / Resolver

`Semantics::Context` is the first symbol-table-like component in Semauri. It assigns stable internal IDs to referable entities and resolves references.

A reference succeeds only when it has exactly one candidate. Zero candidates and multiple candidates are semantic errors. This rule is central to Semauri's philosophy: **natural syntax must not imply probabilistic semantics**.

Future scopes, variables, collections and pronouns should extend this component rather than adding ad-hoc lookup logic to the parser.

### IR

The intermediate representation contains resolved program meaning. `IR::WebDocument` owns immutable `IR::Element` values. Mutations in source create new IR values rather than mutating previously resolved nodes in place.

### Backends — Strategy + Registry

Backends consume IR, never source or AST. The HTML backend maps semantic button color to `background-color`; another backend may render the same semantic property differently.

## Dependency direction

```text
CLI → Compiler → Lexer / Parser / Semantics → IR ← Backends
                                  │
                                  ▼
                           Semantic Context
```

Avoid dependencies in the other direction:

- AST must not depend on HTML.
- IR must not depend on parser tokens.
- Backends must not parse source text.
- Core semantics must not depend on the CLI.
- Reference resolution must not be implemented inside a backend.

## Why no LLM in the compiler core?

The reference compiler is deterministic. An optional future free-form-language adapter could translate unrestricted prose into strict Semauri source, but it would sit before the compiler boundary:

```text
free-form language → optional adapter → strict Semauri → compiler
```

The generated Semauri source would still be parsed, checked and compiled deterministically.
