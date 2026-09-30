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

### Next

- dead immutable-binding elimination with liveness/use analysis
- optimization verification and fixed-point pass scheduling where useful
- formal semantic-domain extension API
- domain vocabulary + semantic contracts without parser monkey-patching
- migrate web operations behind the domain-extension boundary
- explore filesystem and structured-data domains

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
