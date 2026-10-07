# Universal semantic domains

Semauri keeps application-specific meaning in registered semantic domains rather than hard-coded parser branches.

## Core / domain boundary

```text
Semauri Core
├── deterministic lexer + parser
├── primitive / parametric / nominal types
├── symbols + lexical scope
├── control flow
├── typed HIR
├── optimization
├── effects / capabilities
└── registries
     ├── domain vocabulary
     ├── operation patterns
     ├── type contracts
     └── backends
```

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
```

The parser applies the registered pattern; it has no verb-specific branch.

## Effects and plans

Observable operations carry explicit effects. Compilation records those effects and lowers operations into semantic/runtime plans instead of secretly performing I/O.

Program IR can contain multiple domain units, and runtime references preserve typed lineage between operations.

## Domain types

Nominal types provide identities such as `filesystem.path`, `http.url`, `sql.rowset` or `ml.model`. Unrelated nominal types are not implicitly interchangeable.

## Collision strategy

Artifact, element and property terms are globally unique. Action verbs may overlap; ambiguous actions require an explicit scope:

```text
Within filesystem:
  Delete "tmp.log".
End.
```

## Extension rule

Adding a concrete domain should not require editing the lexer, adding verb-specific parser cases, changing unrelated domains or modifying existing backends. New generic extension primitives may evolve the core; concrete vocabulary registers against those primitives.
