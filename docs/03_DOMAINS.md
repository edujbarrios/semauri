# Semantic domains

Semantic domains are Semauri's extension boundary for application-specific meaning. A domain can contribute vocabulary, typed properties, deterministic operation patterns, nominal types, effects and a recommended backend without adding domain-specific branches to the lexer or parser.

## Declaring an artifact domain

Use `DomainSpec` to describe surface vocabulary and type contracts:

```rust
use semauri::{DomainSpec, Type};

let canvas = DomainSpec::new("canvas")
    .with_default_backend("canvas")
    .with_artifact("canvas", "canvas")
    .with_element("badge", "badge")
    .with_property("tone", "tone", Type::Color);
```

The lexer sees generic categories:

```text
canvas → DOMAIN_ARTIFACT(domain=canvas, kind=canvas)
badge  → DOMAIN_ELEMENT(domain=canvas, kind=badge)
tone   → DOMAIN_PROPERTY(domain=canvas, kind=tone)
```

External declarative artifact domains lower to `GenericArtifact` / `GenericElement`. Built-in domains may use specialized IR where their target semantics require it.

## Declarative operations

Operations contribute one or more verbs and a deterministic phrase pattern:

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

The generic parser can then recognize the registered verb without a dedicated parser branch. Typed HIR validates slot types and preserves declared effects. A slot mismatch is `S329`.

Value-returning operations lower to typed runtime references. External operation domains use the generic runtime-plan lowering path; specialized built-in domains may additionally provide compile-time backend IR.

## Explicit semantic-domain scopes

Action verbs may overlap across domains. Ambiguous actions require explicit qualification:

```text
Within filesystem:
  Delete "tmp.log".
End.
```

Outside a scope, multiple candidates fail with `S240`. Inside a scope, an action not owned by the selected domain also fails with `S240`. Domain scopes are lexical variable scopes.

Artifact, element and property surface terms remain globally unique. Action/action overlap is intentionally permitted.

## Effects

Operations declare stable semantic effect identifiers such as `filesystem_read`, `filesystem_write`, `compute` or `model_training`. Effects are preserved in Typed HIR and analyzed before optimization.

Declaring an effect never authorizes or performs it.

## Property typing

Property type contracts are part of the domain declaration:

```rust
let canvas = DomainSpec::new("canvas")
    .with_property("enabled", "enabled", Type::Boolean);
```

The HIR builder validates property values before lowering. Mismatches fail with `S313`.

## Registration

Registration is explicit and transactional:

```rust
use semauri::{Compiler, DomainRegistry};

let mut domains = DomainRegistry::new();
domains.register(canvas)?;

let compiler = Compiler::with_domains(domains);
```

A failed registration does not partially modify the registry.

The vocabulary rules are deterministic:

- artifact, element and property surface terms are globally unique;
- action/action collisions are allowed and become candidate sets;
- actions may not collide with non-action terms;
- core grammar words and primitive color literals are reserved;
- ambiguous actions require `Within <domain>:` qualification.

## Custom backends

Backends are independent of semantic domains:

```rust
use semauri::{Artifact, BackendRegistry, Compiler};

let mut backends = BackendRegistry::builtins();
backends.register("summary", |artifact: &Artifact| {
    Ok(format!("{:?}", artifact))
});

let compiler = Compiler::with_registries(domains, backends);
```

A domain without a default backend requires an explicit backend selection (`S404`). A single backend override is rejected for multi-domain output (`S405`).

## Implicit properties

Shorthand such as:

```text
Make it blue.
```

is accepted only when exactly one loaded property can accept the value type. Ambiguity fails with `S236`.

## Multi-domain Program IR

One source file may produce multiple independent domain units:

```text
Create a web called Shop.
Write "build metadata" to "build.txt".
```

Each unit keeps explicit domain identity and is rendered with its own backend. `CompilationResult.outputs` exposes all generated outputs.

## Built-in domains

**Web** provides web artifacts, buttons/images, `color : color`, specialized Web IR and the `html` backend.

**Structured Data** provides schemas, fields, `datatype : string`, `required : boolean`, specialized schema IR and the `json-schema` backend.

**Filesystem** provides write/append/copy/move/mkdir/touch/delete operations, nominal paths, explicit filesystem effects, operation-plan IR and the `posix-sh` backend.

**ML** provides typed datasets, models, devices, resource/training configuration and runtime-plan operations with explicit compute/training/inference/evaluation effects.

Run `semauri domains` to inspect the active domain metadata.
