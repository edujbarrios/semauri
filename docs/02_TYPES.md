# Semauri type system

Semauri uses explicit semantic types so programs remain deterministic across domains.

## Core types

The compiler supports `unit`, `number`, `boolean`, `string`, `color`, `entity_ref`, `opaque`, and homogeneous `list<T>` values.

## Nominal domain types

Domains can define a semantic identity on top of a primitive representation.

```rust
use semauri::{NominalType, Type};

let path = NominalType::new(
    "filesystem",
    "path",
    Type::String,
    true,
);
let path_type = Type::Nominal(path);
```

The resulting type is `filesystem.path`. Another string-backed nominal type remains incompatible unless an explicit conversion exists.

## Promotion

A nominal type may opt into promotion from its exact base type. Filesystem paths use this so:

```text
Write "hello" to "notes.txt".
```

can lower to:

```text
path : filesystem.path
└── promote string -> filesystem.path
    └── "notes.txt"
```

Promotion is an explicit HIR node, not an invisible coercion.

## Assignment rules

- identical types: exact assignment;
- primitive base to an opted-in nominal type: promotion;
- unrelated nominal types: invalid;
- all other mismatches: invalid.

The compiler deliberately avoids transitive or best-effort coercion.

## Runtime representation

Promotion changes semantic identity without changing the underlying data. A `filesystem.path` carries string data during lowering while its semantic `Type` remains `filesystem.path`. Runtime references preserve the same type.

## Domain declaration

External domains can expose nominal types:

```rust
use semauri::{DomainSpec, NominalType, Type};

let storage_path = NominalType::new(
    "storage",
    "path",
    Type::String,
    true,
);

let storage = DomainSpec::new("storage")
    .with_type(storage_path);
```

Operation signatures reference semantic types through `PatternSegment::slot`. `semauri domains` serializes domain type metadata structurally for tooling.

## Optimizer contract

Nominal promotion is pure but semantically meaningful. Optimization may propagate constants through it or remove an entirely dead pure expression, but it must preserve a live nominal boundary.
