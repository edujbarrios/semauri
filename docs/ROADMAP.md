# Roadmap

Semauri prioritizes compiler and language foundations over rapidly adding targets.

## 0.1 — compiler skeleton ✅

Lexer, handwritten recursive-descent parser, AST, semantic resolution, backend registry, HTML backend, CLI introspection and zero-dependency core tests.

## 0.2 — semantic context and references ✅

Deterministic pronouns/explicit references, ambiguity errors, entity table, immutable IR properties, source spans, diagnostics and provenance.

## 0.3 — values and lexical scope ✅

Typed primitive literals, immutable `Let`, lexical scopes/shadowing, property type checking and expression-oriented assignment.

## 0.4 — structured control flow and compiler IR — in progress

Implemented:

- arithmetic, comparisons and boolean algebra
- `If / Otherwise / End`
- short-circuit evaluation
- stable symbols and symbol introspection
- typed HIR with stable symbol references
- HIR branch preservation
- shared type rules
- homogeneous `List<T>` values
- static `For every ... in ...` iteration
- iterator symbols with one identity across all iterations
- HIR-preserved `for_each`

Next:

- migrate domain lowering to consume HIR
- explicit constant-folding/lowering passes
- collection operations (`length`, indexing/map/filter design)
- dynamic/external values
- CFG/basic blocks when runtime control flow becomes necessary

## 0.5 — semantic domains

Formal extension API for domains such as web, filesystem and structured data. Domains must extend vocabulary, semantics and IR through explicit contracts rather than parser monkey-patching.

## Later

- functions and call frames
- formatter
- LSP, safe rename and go-to-definition powered by symbol IDs
- package/extension model
- additional controlled-language surfaces and backends
- optional free-form NLP/LLM adapter outside the deterministic compiler core

## Non-goals for now

- unrestricted English
- direct LLM-to-target-code generation
- silently guessing ambiguous programs
