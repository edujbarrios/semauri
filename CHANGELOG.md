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
- Generic `DOMAIN_ARTIFACT`, `DOMAIN_ELEMENT` and `DOMAIN_PROPERTY` lexical categories.
- Built-in Web domain implemented through the same extension contract available to external domains.
- Domain surface-term collision detection and cross-domain operation diagnostics.
- `semauri domains` for inspecting loaded domain vocabulary.
- `semauri optimize FILE` for inspecting optimized HIR and per-pass statistics.

### Changed
- `Compiler` now dependency-injects one semantic domain registry through vocabulary, parser, HIR construction and HIR lowering.
- Web-specific artifact/element/property vocabulary is no longer hardcoded in the lexer/parser.
- Syntax AST and Typed HIR carry explicit domain identity for artifact, element, named-reference and property operations.
- Domain implementations own property type contracts and domain-IR construction/mutation hooks.
- `Compiler#analyze` and `explain` lower unoptimized Typed HIR so diagnostics remain source-oriented.
- `Compiler#compile` / `build` lower optimized Typed HIR before domain IR generation.
- `CompilationResult` exposes both raw and optimized HIR.
- Optimized HIR may omit compile-time-only `Let` statements while preserving the full semantic symbol table for tooling.
- Entity reference resolution exposes a data-oriented API shared by legacy AST resolution and HIR lowering.
- Name resolution and type checking are structural HIR phases; short-circuiting skips RHS value evaluation, not name/type validation.
- The AST-based semantic resolver remains temporarily as a regression/reference implementation only.
