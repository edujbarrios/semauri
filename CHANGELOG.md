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
- Generic immutable `IR::OperationPlan` / `IR::Operation` for operation-oriented domains.
- Immutable multi-domain `IR::Program` / ProgramIR container.
- `CompilationResult.outputs` with one typed output per semantic domain/backend.
- Operation-only domains that can lazily initialize semantic plans without an explicit artifact statement.
- Built-in Web domain implemented through the same extension contract available to external domains.
- Structured Data domain with `schema`, `field`, `datatype` and `required` semantics.
- Filesystem domain with `write`, `copy`, `delete` / `remove` operations.
- POSIX shell backend for filesystem plans; compilation generates a script and never performs filesystem effects itself.
- Independent `IR::SchemaDocument` / `IR::SchemaField` representation.
- JSON Schema Draft 2020-12 backend.
- Domain-declared default backends and automatic backend inference in `compile` / `build`.
- Domain surface-term collision detection and cross-domain operation diagnostics.
- Domain-specific schema datatype validation (`S328`).
- Operation argument type diagnostics (`S329`) and malformed operation-pattern diagnostics (`S238`).
- Multi-domain backend override diagnostic (`S405`).
- `semauri domains` for inspecting loaded domain vocabulary and operations.
- `semauri optimize FILE` for inspecting optimized HIR and per-pass statistics.

### Changed
- `Compiler` dependency-injects one semantic domain registry through vocabulary, parser, HIR construction and HIR lowering.
- Web-specific artifact/element/property vocabulary is no longer hardcoded in the lexer/parser.
- Domain-specific action verbs are classified through the same registry rather than becoming core language keywords.
- Syntax AST and Typed HIR carry explicit domain identity for artifacts, elements, references, properties and operations.
- Domain implementations own property type contracts, operation signatures, effect declarations, domain-IR construction/mutation hooks and optional default backend selection.
- HIR lowering now accumulates independent per-domain artifacts/plans instead of enforcing one active semantic domain.
- Entity tables are isolated per semantic domain during lowering.
- Procedural domain operations do not steal declarative artifact focus used by domain-neutral metadata syntax.
- Single-domain `Semantics::Result#program`, `#domain`, `CompilationResult#output` and `#backend` remain backwards compatible.
- Multi-domain CLI builds print separate outputs, or write deterministic per-domain files when `-o` names a directory.
- Constant propagation rewrites pure operation arguments but preserves the operation node and its effects.
- `Compiler#analyze` and `explain` lower unoptimized Typed HIR so diagnostics remain source-oriented.
- `Compiler#compile` / `build` lower optimized Typed HIR before domain IR generation.
- `CompilationResult` exposes both raw and optimized HIR.
- Optimized HIR may omit compile-time-only `Let` statements while preserving the full semantic symbol table for tooling.
- Entity reference resolution exposes a data-oriented API shared by legacy AST resolution and HIR lowering.
- Name resolution and type checking are structural HIR phases; short-circuiting skips RHS value evaluation, not name/type validation.
- The AST-based semantic resolver remains temporarily as a regression/reference implementation only.
