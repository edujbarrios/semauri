# Roadmap

Semauri is an experiment in deterministic natural-language-like programming. The roadmap prioritizes language foundations over rapidly adding targets.

## 0.1 — compiler skeleton ✅

- English vocabulary abstraction
- lexer with source locations
- handwritten recursive-descent parser
- AST visitor boundary
- semantic resolution and explanations
- backend registry
- HTML backend
- CLI introspection commands
- zero-dependency core tests

## 0.2 — semantic context and references ✅

- explicit AST reference model
- constrained pronoun `it`
- deterministic ambiguity errors
- semantic entity table
- immutable element properties in the IR
- contextual `make` (`Make a web...` vs `Make it blue.`)
- explicit named references (`Make the button called Buy blue.`)
- deterministic missing/duplicate reference diagnostics
- source spans with exclusive end positions
- source-aware diagnostics with code excerpts
- semantic provenance for property mutations

The 0.2 line establishes the compiler infrastructure required for later editor tooling and source-to-IR traceability.

## 0.3 — values and scope — next

- literals
- variables
- lexical scope
- assignment semantics
- reusable reference expressions

## 0.4 — structured control flow

- conditions
- iteration
- blocks
- early constraint/type checking

## 0.5 — semantic domains

Establish a formal extension API for domains such as web, filesystem and structured data. Domains must define vocabulary, AST/semantic rules, constraints and IR extensions without allowing arbitrary parser monkey-patching.

## Later

- functions
- formatter
- Language Server Protocol implementation
- package/extension model
- additional controlled-language surfaces
- more rendering/execution backends
- optional free-form NLP/LLM adapter outside the deterministic compiler core

## Non-goals for now

- unrestricted English
- replacing general-purpose languages
- direct LLM-to-target-code generation
- silently guessing ambiguous programs
