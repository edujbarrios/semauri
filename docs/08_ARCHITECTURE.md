# Semauri architecture

Semauri follows a deterministic compiler pipeline with explicit extension boundaries.

## Pipeline

```text
DomainRegistry
  ↓
Source → Lexer → Tokens → Parser → AST → Typed HIR
                                      ↓
                                Optimization
                                      ↓
                                   Lowering
                             ┌────────┴────────┐
                          Program IR      RuntimePlan
                             ↓
                       BackendRegistry
                             ↓
                       Target Artifact
```


## Source layout

The Rust implementation uses a **Facade + staged pipeline** layout. `src/lib.rs` only exposes the public crate API; implementation details live under `src/compiler/` and are grouped by compiler responsibility:

```text
src/
├── lib.rs                     public facade
├── main.rs                    CLI entry point
└── compiler/
    ├── core/                  diagnostics, values and shared types
    ├── domains/               domain model, registry and built-ins
    ├── frontend/              lexer, AST and parser
    ├── semantic/              HIR, optimization and effect analysis
    ├── runtime/               semantic IR, lowering and runtime plans
    ├── backends/              compiler facade, renderers and executors
    ├── cli.rs                 command dispatch
    └── tests.rs               internal regression tests
```

The compiler source slices are deliberately included into one internal Rust module. This preserves the existing private-helper contracts and public API during the migration away from the former monolithic `lib.rs`, while making ownership boundaries explicit and allowing later extraction into independent Rust modules without a large behavior-changing rewrite.

## Domain registry

`DomainSpec` contributes artifact, element, property and action vocabulary plus type/effect contracts. The lexer emits generic domain token categories, so external vocabulary does not require lexer/parser edits.

```rust
use semauri::{Compiler, DomainRegistry};

let mut domains = DomainRegistry::new();
domains.register(my_domain)?;

let compiler = Compiler::with_domains(domains);
```

Registration rejects reserved vocabulary and conflicting non-action terms transactionally. Action/action collisions become candidate sets and require explicit domain scopes when ambiguous.

## AST and Typed HIR

AST nodes preserve source spans and domain identity. HIR resolves names to stable symbol IDs, assigns types, validates domain contracts and keeps effects. Nominal promotions are explicit nodes.

## Optimization

The optimizer performs constant propagation/folding, static dead-branch elimination and conservative dead-binding elimination. Effectful or potentially trapping expressions are not silently discarded.

## Lowering and IR

Lowering evaluates compile-time-known values and static control flow.

Built-in Web and Structured Data domains use specialized IR. Filesystem produces an operation plan. External declarative artifact domains use `GenericArtifact`. External operations and ML operations can produce typed runtime references.

Runtime-dependent control flow is rejected explicitly instead of guessed.

## Backend registry

Backends consume semantic IR and never parse source text or resolve references.

```rust
use semauri::{Artifact, BackendRegistry};

let mut backends = BackendRegistry::builtins();
backends.register("summary", |artifact: &Artifact| {
    Ok(format!("{:?}", artifact))
});
```

`Compiler::with_registries` combines custom domain and backend registries.

## Effects and execution

Effect analysis runs on unoptimized HIR so capability requirements do not depend on optimizer choices. Compilation does not execute effects. The current CLI executor supports explicitly authorized filesystem plans.

## Dependency rules

- lexer/parser do not hard-code domain nouns or verbs;
- AST/HIR do not depend on target markup;
- optimization does not render targets;
- backends do not parse source;
- domain identity and ambiguity resolution remain compiler-owned;
- optional free-form adapters stay before the deterministic compiler boundary.
