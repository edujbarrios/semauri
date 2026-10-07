# Semantic domains

Semantic domains are Semauri's extension boundary for application-specific meaning. Domains add vocabulary, type contracts, operation patterns, effects and backend recommendations without adding domain-specific parser branches.

## Declarative artifact domains

```rust
use semauri::{DomainSpec, Type};

let canvas = DomainSpec::new("canvas")
    .with_default_backend("canvas")
    .with_artifact("canvas", "canvas")
    .with_element("badge", "badge")
    .with_property("tone", "tone", Type::Color);
```

These terms become generic `DOMAIN_ARTIFACT`, `DOMAIN_ELEMENT` and `DOMAIN_PROPERTY` tokens. External declarative artifacts lower to `GenericArtifact` / `GenericElement`.

## Declarative operations

```rust
use semauri::{OperationSpec, PatternSegment, Type};

let write = OperationSpec::new(
    "write",
    ["write"],
    vec![
        PatternSegment::slot("content", Type::String),
        PatternSegment::literal("to"),
        PatternSegment::slot("path", Type::String),
    ],
    Type::Unit,
    ["filesystem_write"],
);

let domain = DomainSpec::new("example")
    .with_operation(write);
```

Registered verbs become generic `DOMAIN_ACTION` tokens. Typed HIR validates slot types and preserves effects. A slot type mismatch reports `S329`.

Value-returning operations produce typed runtime references. External operation domains use the generic runtime-plan lowering path.

## Domain scopes

Action verbs may overlap across domains. Ambiguous actions must be qualified:

```text
Within filesystem:
  Delete "tmp.log".
End.
```

Multiple unqualified candidates fail with `S240`. Domain scopes are lexical variable scopes.

Artifact, element and property surface terms remain globally unique; action/action overlap is allowed.

## Registration

```rust
use semauri::{Compiler, DomainRegistry};

let mut domains = DomainRegistry::new();
domains.register(canvas)?;

let compiler = Compiler::with_domains(domains);
```

Registration is transactional. Reserved grammar words and primitive color names cannot be overridden. A failed registration leaves the registry unchanged.

## Custom backends

```rust
use semauri::{Artifact, BackendRegistry, Compiler};

let mut backends = BackendRegistry::builtins();
backends.register("summary", |artifact: &Artifact| {
    Ok(format!("{:?}", artifact))
});

let compiler = Compiler::with_registries(domains, backends);
```

A domain without a default backend requires explicit backend selection (`S404`). A single backend override is rejected for multi-domain output (`S405`).

## Property typing and shorthand

Properties declare accepted semantic types with `with_property`. HIR validates them before lowering and reports `S313` on mismatch.

Shorthand such as `Make it blue.` is accepted only when exactly one loaded property can accept the value. Ambiguity reports `S236`.

## Multi-domain IR

One source file may produce independent units from multiple domains. `CompilationResult.outputs` exposes each rendered output while domain identity remains explicit throughout AST, HIR and IR.

## Built-in domains

- **Web:** web artifacts, buttons/images, color property, HTML backend.
- **Structured Data:** schemas, fields, datatype/required properties, JSON Schema backend.
- **Filesystem:** typed paths, file operations, explicit effects, POSIX-shell backend.
- **ML:** typed datasets/models/devices/configuration and runtime-plan operations.

Run `semauri domains` to inspect loaded metadata.
