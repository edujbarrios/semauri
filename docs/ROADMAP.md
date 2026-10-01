# Roadmap

Semauri prioritizes compiler and language foundations over rapidly adding targets.

## 0.1 — compiler skeleton ✅

Lexer, handwritten parser, AST, backend registry, HTML backend, CLI introspection and zero-dependency core tests.

## 0.2 — semantic context and references ✅

Deterministic references, ambiguity diagnostics, entity table, immutable IR, source spans and provenance.

## 0.3 — values and lexical scope ✅

Typed primitive values, immutable bindings, lexical scope, shadowing and property type checking.

## 0.4 — structured control flow and compiler IR ✅

- arithmetic, comparisons and boolean algebra
- `If / Otherwise / End`
- short-circuit evaluation
- stable symbols
- typed HIR
- homogeneous `List<T>`
- static `For every ... in ...`
- HIR-preserved branches and loops
- production lowering consumes HIR rather than AST
- separate symbol identity and compile-time value environments
- full-program symbol table for tooling, including unselected branches

## 0.5 — optimization and semantic domains — in progress

### 0.5.0 — HIR optimization pipeline ✅

- explicit composable `PassManager`
- constant propagation for immutable bindings
- constant folding for arithmetic, comparisons and boolean expressions
- short-circuit-aware folding
- dead conditional branch elimination
- dead empty-loop elimination infrastructure
- `semauri optimize FILE` introspection with per-pass statistics
- production `build` lowers optimized HIR while `explain` remains source-oriented

### 0.5.1 — liveness/use analysis ✅

- backwards symbol-use analysis over statement scopes
- dead immutable `Let` elimination
- transitive removal of compile-time-only binding chains
- loop-body liveness propagation to enclosing scopes
- full semantic symbol table retained for tooling even when optimized HIR drops bindings

### 0.5.2 — semantic domain extension API ✅

- generic domain artifact, element and property tokens
- domain-aware AST and Typed HIR
- dependency-injected `Domains::Registry`
- domain-owned vocabulary terms, property typing and domain-IR construction
- built-in Web domain migrated behind the extension boundary
- cross-domain mismatch diagnostics
- domain surface-term collision detection
- `semauri domains` introspection
- contract tests proving an external domain can compile without lexer/parser changes

### Next

- fixed-point optimization scheduling and optimization verification
- move artifact metadata such as `title` behind domain contracts
- add a second built-in domain to exercise the public extension API
- explore structured-data before filesystem so the domain model is tested without introducing I/O side effects
- formal plugin/package discovery model

## Later

- dynamic/external values
- CFG/basic blocks and runtime branch/loop lowering
- functions and call frames
- collection operations
- formatter
- LSP, safe rename and go-to-definition
- package/extension model
- additional language surfaces/backends
- optional NLP/LLM adapter outside the deterministic core

## Non-goals for now

- unrestricted English
- direct LLM-to-target-code generation
- silently guessing ambiguous programs
