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
- constant propagation and folding
- dead conditional/control-flow elimination
- dead immutable-binding analysis
- `semauri optimize FILE`

### 0.5.2 — semantic domain extension API ✅

- generic artifact, element, property and action parser categories
- dependency-injected `Domains::Registry`
- domain-owned vocabulary, types, operations and lowering
- Web and Structured Data domains behind the public extension boundary
- JSON Schema backend and automatic domain backend selection

## 0.6 — universal semantic operations ✅

### 0.6.0–0.6.5 ✅

- declarative typed domain operations
- Filesystem domain and POSIX shell plans
- multi-domain `IR::Program`
- `Within <domain>: ... End.` scopes
- nominal semantic domain types
- static effect analysis and capability policies
- runtime operation values
- immutable `RuntimePlan`, `RuntimeOperation` and SSA-like `RuntimeValueRef`
- `semauri effects` and `semauri plan`

See [`UNIVERSAL_DOMAINS.md`](UNIVERSAL_DOMAINS.md) and [`RUNTIME.md`](RUNTIME.md).

## 0.7 — AI / ML semantic foundations — in progress

### 0.7.0 — built-in ML domain ✅

- `ml.dataset`, `ml.model`, `ml.device`, `ml.training_run`, `ml.inference_run`
- typed dataset/model/device operations
- typed training and inference plans
- explicit ML effects without framework execution during compilation

### 0.7.1 — typed training configuration ✅

- opaque `ml.training_config`
- optimizer, epochs, learning rate, batch size and seed
- configured `Fit`
- domain-level semantic validation (`S336`)

### 0.7.2 — model construction and fine-tuning transforms ✅

- typed `Build cnn`
- immutable `Freeze` model derivation
- immutable LoRA adaptation
- explicit model lineage in RuntimePlan
- model configuration diagnostics (`S337`)

### 0.7.3 — standalone Semauri distribution ✅

- user-facing `semauri` command no longer requires a system Ruby installation
- versioned `~/.semauri/versions/<version>` distribution layout
- private pinned portable Ruby runtime behind the compiler launcher
- checksum-verifying `install.sh`
- Linux/macOS x86_64 and arm64 release targets
- automated GitHub Release publishing from version changes on `main`
- packaged-distribution smoke tests before publication
- source/Ruby workflow retained only for compiler development

### Next

- richer CNN architecture/layer semantics
- precision, gradient accumulation and checkpoint policy
- dataset transforms, splits and fingerprints
- evaluation/metric plans
- QLoRA and explicit adapter targets
- hardware/resource constraints and memory planning
- runtime expressions beyond direct operation-to-operation dataflow
- SSA/CFG/basic blocks for runtime `If`, loops and arithmetic
- formal plugin/package discovery for external domains and runtimes

See [`ML.md`](ML.md), [`DISTRIBUTION.md`](DISTRIBUTION.md), [`RUNTIME.md`](RUNTIME.md) and [`UNIVERSAL_DOMAINS.md`](UNIVERSAL_DOMAINS.md).

## Later

- PyTorch/Transformers execution runtime
- checkpoint/artifact lineage
- model interpretability operations (Grad-CAM, Integrated Gradients, attention/feature attribution)
- Windows distribution support
- Homebrew/Scoop/Winget integration where maintainable
- project-local Semauri version selection
- functions and call frames
- formatter and LSP
- package/extension model
- optional NLP/LLM adapter outside the deterministic core

## Non-goals for now

- unrestricted English
- direct LLM-to-target-code generation
- silently guessing ambiguous programs
- performing external side effects merely because source code was compiled
- coupling ML source semantics directly to one framework
