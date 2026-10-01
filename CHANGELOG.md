# Changelog

All notable changes to Semauri are documented here.

## Unreleased

### Added
- Stable semantic symbols and typed HIR.
- Homogeneous `List<T>` collections and static `For every` iteration.
- HIR value environments that separate semantic symbol identity from current values.
- HIR expression evaluator with short-circuit behavior and runtime-only checks such as division by zero.
- HIR-to-domain-IR lowering for the production compiler pipeline.
- Full-program symbol tables that include declarations in statically unselected branches.
- Composable HIR optimization `PassManager`.
- Constant propagation and folding for immutable compile-time values.
- Dead conditional branch elimination and empty-loop elimination infrastructure.
- Backwards symbol liveness/use analysis and dead immutable-binding elimination.
- Semantic domain extension API (`Domains::Definition`, `Domains::Registry`, `Domains::Term`).
- Generic `DOMAIN_ARTIFACT`, `DOMAIN_ELEMENT`, `DOMAIN_PROPERTY` and `DOMAIN_ACTION` lexical categories.
- Declarative semantic operations with typed expression slots, return types and effect metadata.
- Generic `AST::DomainOperation` and typed `HIR::domain_operation` nodes.
- Explicit `Within <domain>: ... End.` semantic scopes and `AST/HIR::domain_scope` representation.
- Ambiguous action-candidate diagnostics (`S240`) and lowering scope-consistency diagnostic (`S332`).
- Nominal semantic domain types through `Semantics::NominalType`.
- Explicit primitive-to-nominal `HIR::promote` nodes.
- Domain type registration/introspection and structured operation-signature type metadata.
- `filesystem.path` as the first built-in nominal domain type.
- Static effect analysis with per-use domain, operation and source provenance.
- Immutable capability allow-list policies with denied-capability diagnostic `S334`.
- `semauri effects FILE` and capability-aware `semauri check` validation.
- Generic immutable `IR::OperationPlan` / `IR::Operation` for operation-oriented domains.
- Immutable multi-domain `IR::Program` / ProgramIR container.
- `CompilationResult.outputs` with one typed output per semantic domain/backend.
- `CompilationResult.effects` with the conservative source-oriented effect manifest.
- Operation-only domains that can lazily initialize semantic plans without an explicit artifact statement.
- Built-in Web domain implemented through the same extension contract available to external domains.
- Structured Data domain with `schema`, `field`, `datatype` and `required` semantics.
- Filesystem domain with `write`, `copy`, `delete` / `remove` operations.
- Experimental ML semantic domain for dataset/model/training/evaluation/checkpoint/interpretability planning.
- Immutable `IR::MLPlan` with semantic validation of named model and dataset references.
- CNN initialization, pretrained-model selection, component freezing, training, evaluation, save and explainability operations.
- AI-specific effect declarations including `model_download`, `gpu_compute`, `model_training`, `model_inference`, `checkpoint_write` and `model_explanation`.
- `ml-plan` JSON backend for inspecting AI workflows without executing them.
- ML plan validation diagnostic `S336`.
- POSIX shell backend for filesystem plans; compilation generates a script and never performs filesystem effects itself.
- Independent `IR::SchemaDocument` / `IR::SchemaField` representation.
- JSON Schema Draft 2020-12 backend.
- Domain-declared default backends and automatic backend inference in `compile` / `build`.
- Domain surface-term collision detection and cross-domain operation diagnostics.
- Domain-specific schema datatype validation (`S328`).
- Operation argument type diagnostics (`S329`), nominal assignment diagnostics (`S333`) and malformed operation-pattern diagnostics (`S238`).
- Multi-domain backend override diagnostic (`S405`).
- `semauri domains` for inspecting loaded domain vocabulary, types and operations.
- `semauri optimize FILE` for inspecting optimized HIR and per-pass statistics.

### Changed
- `Compiler` dependency-injects one semantic domain registry through vocabulary, parser, HIR construction and HIR lowering.
- Web-specific artifact/element/property vocabulary is no longer hardcoded in the lexer/parser.
- Domain-specific action verbs are classified through the same registry rather than becoming core language keywords.
- Multiple domains may own the same action verb; unqualified ambiguous actions require explicit semantic scope.
- Artifact, element and property surface terms remain globally unique for now.
- Domain scopes are lexical variable scopes and survive optimization rather than being parser-only hints.
- Syntax AST and Typed HIR carry explicit domain identity for artifacts, elements, references, properties, operations and scopes.
- Operation argument checking uses explicit assignability rules instead of raw type equality.
- Primitive values may promote to a nominal type only when they exactly match its declared base representation.
- Distinct nominal types are never implicitly reinterpreted as one another, even when they share a primitive base.
- Filesystem path arguments are semantically `filesystem.path` while existing string source syntax remains compatible through explicit HIR promotion.
- Domain implementations own property type contracts, nominal types, operation signatures, effect declarations, domain-IR construction/mutation hooks and optional default backend selection.
- Effect analysis is conservative over unoptimized Typed HIR so policy does not depend on optimizer behavior.
- Capability policies are optional during compilation because `build` does not execute effects; future execution runtimes are expected to enforce them.
- ML source currently compiles to an inspectable semantic plan only; model downloads, dataset reads, GPU work, training and checkpoint writes remain runtime concerns.
- HIR lowering accumulates independent per-domain artifacts/plans instead of enforcing one active semantic domain.
- Entity tables are isolated per semantic domain during lowering.
- Procedural domain operations do not steal declarative artifact focus used by domain-neutral metadata syntax.
- Single-domain `Semantics::Result#program`, `#domain`, `CompilationResult#output` and `#backend` remain backwards compatible.
- Multi-domain CLI builds print separate outputs, or write deterministic per-domain files when `-o` names a directory.
- Constant propagation rewrites pure operation arguments and values inside nominal promotions while preserving operation effects and nominal boundaries.
- `Compiler#analyze` and `explain` lower unoptimized Typed HIR so diagnostics remain source-oriented.
- `Compiler#compile` / `build` lower optimized Typed HIR before domain IR generation.
- `CompilationResult` exposes both raw and optimized HIR.
- Optimized HIR may omit compile-time-only `Let` statements while preserving the full semantic symbol table for tooling.
- Entity reference resolution exposes a data-oriented API shared by legacy AST resolution and HIR lowering.
- Name resolution and type checking are structural HIR phases; short-circuiting skips RHS value evaluation, not name/type validation.
- The AST-based semantic resolver remains temporarily as a regression/reference implementation only.
