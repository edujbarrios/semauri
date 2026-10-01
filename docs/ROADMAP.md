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

## 0.5 — optimization and semantic domains ✅

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

### 0.5.3 — Structured Data domain ✅

- second built-in semantic domain implemented through the public extension API
- `schema` artifacts and `field` entities
- typed `datatype:string` and `required:boolean` properties
- independent `SchemaDocument` / `SchemaField` immutable domain IR
- JSON Schema Draft 2020-12 backend
- domain-specific datatype validation (`S328`)
- semantic domains declare default backends
- `build` infers `html` for Web and `json-schema` for Structured Data

## 0.6 — universal semantic operations — in progress

### 0.6.0 — declarative domain operations ✅

- domains declare action verbs without adding lexer keywords
- deterministic operation phrase patterns with typed expression slots
- generic `AST::DomainOperation` and `HIR::domain_operation`
- operation result types and effect/capability metadata in HIR
- operation-only domains can lazily create semantic plans without `Create ...`
- generic immutable `IR::OperationPlan` / `IR::Operation`
- optimizers propagate constants into operation arguments while preserving effectful operations
- Filesystem domain as a non-CRUD proof of the extension model
- POSIX shell backend generated from filesystem plans; compilation never performs filesystem effects
- external-domain contract test proving a new verb reaches HIR without parser-specific code

### 0.6.1 — multi-domain ProgramIR ✅

- immutable `IR::Program` composed of independent semantic-domain units
- separate domain artifacts/plans and entity tables during lowering
- Web, Structured Data and Filesystem may coexist in one source file
- `Semantics::Result` exposes `program_ir` while preserving single-domain compatibility
- `CompilationResult.outputs` renders each domain with its own default backend
- single-domain `output` / `backend` behavior remains backwards compatible
- CLI prints multi-domain outputs separately or writes them to a directory with deterministic names
- one global backend override is rejected for multi-domain programs (`S405`)
- procedural operations do not steal contextual focus from declarative artifacts

### 0.6.2 — explicit semantic domain scopes ✅

- `Within <domain>: ... End.` lexical semantic scopes
- action verbs may be shared by multiple semantic domains
- unqualified ambiguous actions fail deterministically with `S240`
- scoped action resolution selects exactly the operation owned by the named domain
- scope identity is preserved through AST, Typed HIR and optimization passes
- HIR lowering verifies scoped operation/domain consistency (`S332`)
- domain scopes are also lexical variable scopes
- non-action vocabulary remains globally collision-free for now

### Next

- nominal domain types (`filesystem.path`, `http.url`, `sql.rowset`, `ml.tensor`, ...)
- operation expressions that can produce runtime values
- effect-aware validation and capability policies
- formal plugin/package discovery for external domains and backends
- runtime-value design: inputs, CFG/basic blocks and the compile-time/runtime boundary

See [`UNIVERSAL_DOMAINS.md`](UNIVERSAL_DOMAINS.md) for the architecture target.

## Later

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
- performing external side effects merely because source code was compiled
