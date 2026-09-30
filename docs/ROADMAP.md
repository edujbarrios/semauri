# Roadmap

Semauri is an experiment in deterministic natural-language-like programming. The roadmap prioritizes language and compiler foundations over rapidly adding targets.

## 0.1 — compiler skeleton ✅

- lexer and English vocabulary abstraction
- handwritten recursive-descent parser
- AST visitor boundary
- semantic resolution and explanations
- backend registry + HTML backend
- CLI introspection and zero-dependency core tests

## 0.2 — semantic context and references ✅

- deterministic `it` and explicit references
- ambiguity errors
- entity table and immutable IR properties
- source spans and source-aware diagnostics
- semantic provenance

## 0.3 — values and lexical scope ✅

- typed color/string/number/boolean literals
- immutable `Let` bindings
- lexical scopes and shadowing
- property type checking
- canonical `Set ... to ...` assignment

## 0.4 — structured control flow and compiler IR — in progress

Implemented:

- `Block`, unary/binary expressions and `IfStatement`
- arithmetic precedence and parentheses
- strict comparisons and boolean conditions
- `If / Otherwise / End`
- `and` / `or` / `not` with short-circuit evaluation
- stable semantic symbols and shared symbol tables
- `semauri symbols` tooling
- first typed HIR representation
- HIR symbol references use stable IDs rather than source-name lookup
- HIR preserves both conditional branches independently from semantic-time branch execution
- shared type rules for evaluation and HIR construction
- `semauri hir` tooling

Next:

- migrate semantic/domain lowering to consume HIR
- introduce first-class type objects (`List<T>` is the first target)
- collection literals
- static `For every ... in ...` iteration
- lowering/constant folding as explicit passes rather than implicit HIR construction
- CFG/basic-block IR once external/runtime values are introduced

## 0.5 — semantic domains

Formal extension API for domains such as web, filesystem and structured data. Domains must extend vocabulary, semantics and IR through explicit contracts rather than parser monkey-patching.

## Later

- runtime/external values and CFG lowering
- functions and call frames
- formatter
- LSP, safe rename and go-to-definition powered by symbol IDs
- package/extension model
- additional controlled-language surfaces and backends
- optional free-form NLP/LLM adapter outside the deterministic compiler core

## Non-goals for now

- unrestricted English
- replacing general-purpose languages
- direct LLM-to-target-code generation
- silently guessing ambiguous programs
