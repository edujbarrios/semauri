# Semauri architecture

Semauri follows a compiler pipeline with strict boundaries so language evolution remains reviewable and target generation stays separate from parsing.

## Pipeline

```text
DomainRegistry
  ├── vocabulary
  ├── property/type contracts
  └── operation signatures + effects
            │
            ▼
Source → Lexer → Tokens
                 ↓
               Parser
                 ↓
             Syntax AST
                 ↓
          Typed HIR Builder
          ├── symbols
          ├── type checking
          └── domain operations
                 ↓
           HIR Optimization
          ├── constant folding
          ├── dead control flow
          └── dead bindings
                 ↓
               Lowerer
          ├── static evaluation
          ├── domain IR
          └── RuntimePlan
                 ↓
           BackendRegistry
                 ↓
          Target Artifact
```

## Semantic domains

A `DomainSpec` contributes generic artifact, element, property and action terms plus type/effect contracts. The lexer emits generic domain token categories rather than domain-specific token kinds.

External domains are registered without changing parser code:

```rust
use semauri::{Compiler, DomainRegistry};

let mut domains = DomainRegistry::new();
domains.register(my_domain)?;

let compiler = Compiler::with_domains(domains);
```

Registration rejects reserved vocabulary and conflicting non-action terms transactionally. Action/action collisions are represented as candidates and require explicit `Within <domain>:` resolution.

`semauri domains` exposes loaded metadata for tooling.

## Syntax AST

AST nodes retain source spans and explicit domain identity. They describe source syntax but do not render target output.

## Typed HIR

HIR resolves names to stable symbol IDs, assigns types, validates property/operation contracts and preserves effects. Nominal promotions are explicit nodes.

## Optimization

Optimization consumes typed HIR and returns typed HIR. The current pipeline performs constant propagation/folding, static dead-branch elimination and conservative dead-binding elimination.

Effectful or potentially trapping expressions are not silently discarded.

## Lowering

Lowering evaluates compile-time-known values and static control flow.

Built-in Web and Structured Data domains lower to specialized IR. Filesystem operations lower to an operation plan. External declarative artifact domains lower to `GenericArtifact`; external operations lower to the generic runtime plan. ML produces typed runtime operations and references.

Runtime-dependent control flow is rejected explicitly rather than guessed.

## Program IR

A source file may produce multiple domain units. Each unit retains its domain identity and artifact/plan. A single backend override is only valid for a single-domain program.

## Backend registry

Backends consume semantic IR and never parse source text or resolve references.

```rust
use semauri::{Artifact, BackendRegistry};

let mut backends = BackendRegistry::builtins();
backends.register("summary", |artifact: &Artifact| {
    Ok(format!("{:?}", artifact))
});
```

`Compiler::with_registries` combines a custom domain registry and backend registry.

## Effects and execution

Effect analysis walks unoptimized HIR, so capability requirements do not depend on optimizer choices. Capability policies are explicit allow-lists.

Compilation does not execute effects. The current CLI executor supports authorized filesystem plans; other runtime domains remain planning-only.

## Dependency direction

```text
CLI → Compiler → Lexer/Parser → AST → HIR → Optimization → Lowering → IR ← Backends
                     ↑                                         ↑
                DomainRegistry                           BackendRegistry
```

Key rules:

- lexer/parser do not hard-code domain nouns or verbs;
- AST/HIR do not depend on target markup;
- optimization does not render targets;
- backends do not parse source text;
- domain identity and ambiguity resolution stay compiler-owned;
- optional free-form/LLM adapters must remain before the deterministic compiler boundary.
