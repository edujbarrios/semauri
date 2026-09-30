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

### Changed
- `Compiler#analyze`, `check` and `build` now lower from Typed HIR rather than executing the syntax AST directly.
- `CompilationResult` exposes the HIR used to produce the semantic IR.
- Entity reference resolution exposes a data-oriented API shared by legacy AST resolution and HIR lowering.
- Name resolution and type checking are structural HIR phases; short-circuiting skips RHS value evaluation, not name/type validation.
- The AST-based semantic resolver remains temporarily as a regression/reference implementation only.
