# Roadmap

Semauri is an experiment in deterministic natural-language-like programming. The roadmap prioritizes language and compiler foundations over rapidly adding targets.

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
- canonical property assignment
- reusable expression contexts

## 0.4 — structured control flow and semantic identity — in progress

Implemented:

- first-class `Block`, `BinaryExpression`, `UnaryExpression` and `IfStatement` AST nodes
- child lexical scopes for blocks
- arithmetic precedence and parentheses
- strict typed equality and numeric ordering
- boolean conditions
- deterministic `If / Otherwise / End`
- semantic-time branch evaluation while values are compile-time known
- logical `and` / `or` / `not`
- short-circuit boolean evaluation
- stable semantic `Symbol` identities independent from source names
- shared `SymbolTable` across lexical child scopes
- symbol IDs/types/source spans exposed to tooling with `semauri symbols`

Next compiler work:

- typed HIR (`syntax AST -> name/type resolution -> HIR`)
- symbol references in HIR rather than repeated string lookup
- collection types such as `List<T>`
- static iteration with `For every ... in ...`
- lowering static loops without losing source provenance
- a control-flow IR once external/runtime values exist

## 0.5 — semantic domains

Establish a formal extension API for domains such as web, filesystem and structured data. Domains must define vocabulary, semantic rules, constraints and IR extensions without arbitrary parser monkey-patching.

## Later

- runtime/external values and CFG/basic-block lowering
- functions and call frames
- formatter
- Language Server Protocol implementation
- safe rename / go-to-definition powered by symbol IDs
- package/extension model
- additional controlled-language surfaces
- more rendering/execution backends
- optional free-form NLP/LLM adapter outside the deterministic compiler core

## Non-goals for now

- unrestricted English
- replacing general-purpose languages
- direct LLM-to-target-code generation
- silently guessing ambiguous programs
