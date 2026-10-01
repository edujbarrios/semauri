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

## 0.6 — universal semantic operations ✅

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

### 0.6.3 — nominal semantic domain types ✅

- immutable `Semantics::NominalType` with domain, name and base representation
- strict nominal identity: `filesystem.path` is not interchangeable with a future `http.url`
- explicit primitive-to-nominal promotion represented as `HIR::promote`
- operation signatures accept nominal types and expose them through domain introspection
- evaluator preserves nominal identity while carrying the underlying runtime value
- optimizer rewrites values inside promotions without erasing nominal boundaries
- Filesystem path parameters migrated from raw `string` to `filesystem.path`
- backwards-compatible source ergonomics: string expressions may be promoted at a typed operation boundary

### 0.6.4 — effect analysis and capability policies ✅

- static effect analysis over unoptimized Typed HIR
- per-effect provenance records domain, operation and source span
- conservative manifests include potentially reachable effectful operations before optimization
- immutable `CapabilityPolicy` with explicit allow-list semantics
- denied capabilities fail with `S334`
- `CompilationResult` exposes its effect analysis
- `semauri effects FILE` prints the capability/effect manifest as JSON
- `semauri check FILE --allow EFFECT` validates an explicit capability policy
- compilation may optionally validate a policy but still never performs external effects
- future execution runtimes can make capability authorization mandatory before running plans

### 0.6.5 — runtime operation values ✅

- value-returning domain operations may appear inside expressions such as `Let response be Fetch ...`
- immutable `IR::RuntimePlan`, `IR::RuntimeOperation` and SSA-like `IR::RuntimeValueRef`
- runtime results can flow directly into later semantic-domain operations
- non-unit operations are planned rather than executed at compile time
- unit operations depending on runtime values are also lowered into the runtime plan
- `CompilationResult.runtime_plan` and `Compiler#runtime_plan`
- `semauri plan FILE` runtime-plan introspection
- runtime-dependent compile-time evaluation fails explicitly with `S335`
- pure runtime programs direct `build` users to `plan` with `S406`
- compilation/planning still never performs declared external effects

## 0.7 — AI / ML semantic foundations — in progress

### 0.7.0 — built-in ML domain ✅

- built-in `ml` semantic domain registered through the same public domain API
- nominal types: `ml.dataset`, `ml.model`, `ml.device`, `ml.training_run`, `ml.inference_run`
- typed dataset opening, model loading and device selection operations
- typed training and inference operations
- ML values flow through `RuntimePlan` using SSA-like runtime references
- explicit ML effects: `filesystem_read`, `model_load`, `compute`, `model_training`, `model_inference`
- compilation creates inspectable plans and never imports or runs an ML framework
- framework-independent contract documented in `ML.md`

### 0.7.1 — typed training configuration ✅

- opaque `ml.training_config` nominal type produced only by ML semantics
- nominal types explicitly declare whether primitive-to-nominal promotion is allowed
- ML runtime-produced objects reject forged primitive values
- `Configure training` captures epochs, optimizer, learning rate, batch size and seed
- configured `Fit` consumes `ml.model`, `ml.dataset`, `ml.device` and `ml.training_config`
- simple `Train ... for N epochs` remains available
- domain-level operation validation hook keeps value constraints outside the parser
- static validation for optimizer name, positive epochs/learning rate/batch size and non-negative seed
- invalid ML configuration reports `S336`

### 0.7.2 — model construction and fine-tuning transforms ✅

- `Build cnn` creates a typed `ml.model` plan with class count and input channels
- `Freeze <model> component <name>` produces a new derived model value
- `Apply lora to <model> rank <r> alpha <a>` produces a new derived model value
- model lineage is explicit through SSA-like RuntimePlan references rather than hidden mutation
- model construction/transforms are pure planning operations and do not execute a framework
- static model-configuration validation reports `S337`
- downstream `Fit` consumes the exact derived model reference selected by the source program

### 0.7.3 — standalone Semauri distribution ✅

- official Semauri installation no longer requires system Ruby
- versioned user-owned installation under `~/.semauri/versions/<version>`
- private pinned portable Ruby runtime behind the `semauri` launcher
- checksum-verifying `install.sh` for Linux/macOS x86_64 and arm64
- automatic `current` version symlink and `~/.local/bin/semauri` command
- automated release packaging with upstream runtime checksum pinning
- packaged distribution smoke tests before publication
- automatic GitHub Release/tag creation when a new Semauri version reaches `main`
- contributor source workflow remains available with Ruby 3.2+

### Next

- richer CNN architecture/layer semantics
- precision, gradient accumulation and checkpoint policy in training configuration
- dataset transforms, splits and fingerprints
- evaluation/metric plans
- additional fine-tuning semantics: QLoRA and explicit adapter targets
- hardware/resource constraints and memory planning
- runtime expressions beyond direct operation-to-operation dataflow
- SSA/CFG/basic blocks for runtime `If`, loops and arithmetic
- formal plugin/package discovery for external domains and runtimes

See [`ML.md`](ML.md), [`DISTRIBUTION.md`](DISTRIBUTION.md), [`RUNTIME.md`](RUNTIME.md) and [`UNIVERSAL_DOMAINS.md`](UNIVERSAL_DOMAINS.md) for the architecture direction.

## Later

- PyTorch/Transformers execution runtime
- checkpoint/artifact lineage
- model interpretability operations (Grad-CAM, Integrated Gradients, attention/feature attribution)
- Windows distribution support
- Homebrew/Scoop/Winget integration where maintainable
- project-local Semauri version selection
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
- coupling ML source semantics directly to one framework
