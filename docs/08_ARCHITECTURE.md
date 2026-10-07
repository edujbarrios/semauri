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
