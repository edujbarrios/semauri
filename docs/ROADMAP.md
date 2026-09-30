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
- contextual `make`
- explicit named references
- source spans
- source-aware diagnostics
- semantic provenance for property mutations

## 0.3 — values and lexical scope ✅

- typed color, string, number and boolean literals
- expression-oriented property values
- immutable `Let` bindings
- variable references
- parent-capable lexical scopes
- unknown/duplicate binding diagnostics
- property value type checking
- canonical `Set the color of ... to ...` assignment syntax
- reusable expression contexts

## 0.4 — structured control flow — in progress

Implemented:

- first-class `Block` AST nodes
- child lexical scopes for blocks
- first-class `BinaryExpression` AST nodes
- arithmetic precedence (`times`/`divided by` before `plus`/`minus`)
- parentheses
- strict typed equality
- numeric ordering comparisons
- boolean conditions
- deterministic `If / Otherwise / End` blocks
- semantic-time branch evaluation while all language values are compile-time known
- logical `and` / `or` / `not` with defined precedence
- short-circuit boolean evaluation

Next:

- iteration
- collections
- a control-flow IR once Semauri introduces runtime/external values

## 0.5 — semantic domains

Establish a formal extension API for domains such as web, filesystem and structured data. Domains must define vocabulary, AST/semantic rules, constraints and IR extensions without allowing arbitrary parser monkey-patching.

## Later

- functions and call frames
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
