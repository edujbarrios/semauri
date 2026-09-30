# Semauri architecture

Semauri follows a compiler pipeline with strict boundaries so language evolution remains reviewable and contributors can work on one phase without understanding every backend.

## Pipeline

```text
Source
  ↓
Vocabulary + Lexer
  ↓
Tokens
  ↓
Recursive-descent Parser
  ↓
Syntax AST
  ↓
Typed HIR Builder
  ├── lexical name resolution
  ├── stable symbol IDs
  └── type checking
  ↓
HIR Lowerer
  ├── compile-time expression evaluation
  ├── static branch/loop execution
  └── deterministic entity/reference resolution
  ↓
Semantic / Domain IR
  ↓
Backend Registry
  ↓
Target Artifact
```

## Syntax AST

The AST represents source syntax and source spans. It does not decide what a pronoun refers to and it does not contain HTML knowledge.

## Typed HIR

HIR is the compiler's semantic structural representation. Names become stable symbol references, expressions carry types, and both branches of conditionals and loop structure are preserved.

A source variable such as `price` is no longer identified by its spelling after this phase:

```text
price → symbol_ref(#3, number)
```

This is the representation future tooling and optimization passes should consume.

## HIR lowering

The executable compiler lowers HIR into domain IR. Because all current values are compile-time-known, the lowerer can currently evaluate conditions and iterate static lists.

Importantly, those execution decisions happen **after HIR construction**. HIR itself preserves program structure, so introducing runtime values later does not require redesigning the parser or symbol model.

The previous AST-based `Semantics::Resolver` remains temporarily as a regression/reference implementation during the 0.x migration, but the production `Compiler` pipeline consumes HIR.

## Symbols and value environments

Semantic symbol identity and current values are separate concerns:

```text
Symbol #4
  name: accent
  type: color
  kind: iterator

Value environment, iteration 1: #4 → red
Value environment, iteration 2: #4 → green
```

This prevents static loop execution from inventing a new declaration on every iteration and prepares the compiler for runtime frames.

## Entity table

`Semantics::EntityTable` resolves generated domain entities such as buttons and images. AST and HIR both delegate to the same data-oriented resolution logic so ambiguity diagnostics stay consistent.

## Semantic IR and backends

`IR::WebDocument` and `IR::Element` contain resolved domain meaning, not source text or target markup. Backends consume that IR and never parse natural language or resolve references.

## Dependency direction

```text
CLI → Compiler → Lexer/Parser → AST → HIR → Lowering → Domain IR ← Backends
```

Key rules:

- AST must not depend on HTML.
- HIR must not depend on a target backend.
- domain IR must not depend on parser tokens.
- backends must not parse source text.
- ambiguity resolution must not happen in a backend.
- optional NLP/LLM support must sit before the deterministic compiler boundary.

## Why no LLM in the compiler core?

An optional future free-form adapter may translate unrestricted prose into strict Semauri source:

```text
free-form language → optional adapter → strict Semauri → compiler
```

The resulting program is still parsed, typed and compiled deterministically.
