# Semauri type system

Semauri uses semantic types to keep controlled natural-language programs deterministic across domains.

## Primitive and parametric types

The compiler currently exposes `unit`, `number`, `boolean`, `string`, `color`, `entity_ref`, `opaque`, and `list<T>`.

Primitive types are structural. Lists are homogeneous and carry their element type.

## Nominal domain types

Domains may define values that share a primitive representation but have different semantic meaning.

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

This produces the semantic type:

```text
filesystem.path
```

A different domain can define a string-backed type such as `http.url`. The two nominal types remain distinct even when their underlying data representation is the same.

## Explicit promotion

A nominal type may opt into promotion from its exact base type. Filesystem paths use this for ergonomic string literals:

```text
Write "hello" to "notes.txt".
```

Typed HIR makes the boundary explicit:

```text
domain_operation filesystem.write : unit
├── content : string = "hello"
└── path : filesystem.path
    └── promote string -> filesystem.path
        └── "notes.txt"
```

The promotion is a compiler node, not an invisible coercion. Nominal values are never reinterpreted as another nominal type merely because their bases match.

## Assignment rules

The current assignment relation is:

- exact type equality → exact assignment;
- primitive base → nominal type with `promote_from_base = true` → promotion;
- everything else → invalid.

Examples:

```text
string          -> filesystem.path   promote
filesystem.path -> filesystem.path   exact
filesystem.path -> http.url          invalid
http.url        -> filesystem.path   invalid
number          -> filesystem.path   invalid
```

## Runtime representation

Promotion changes semantic type identity without changing the underlying data. A `filesystem.path` carries string data during compilation/lowering while its semantic `Type` remains `filesystem.path`.

Runtime values preserve the same semantic type in `RuntimeValueRef`, so later operations can consume typed results without knowing a concrete value at compile time.

## Domain declaration

External domains can publish nominal types through `DomainSpec`:

```rust
use semauri::{DomainSpec, NominalType, Type};

let path = NominalType::new("storage", "path", Type::String, true);

let domain = DomainSpec::new("storage")
    .with_type(path);
```

Operation patterns reference semantic types directly:

```rust
use semauri::{PatternSegment, Type};

let path_slot = PatternSegment::slot(
    "path",
    Type::Nominal(NominalType::new(
        "storage",
        "path",
        Type::String,
        true,
    )),
);
```

`semauri domains` serializes nominal type metadata structurally for tooling.

## Optimizer contract

Nominal promotion is pure but semantically meaningful. Optimizers may propagate constants through it or remove an entirely dead pure expression, but must preserve a live nominal boundary.

## Design direction

Nominal types make cross-domain contracts explicit for concepts such as filesystem paths, URLs, responses, queries, rowsets, models, tensors and cloud resources without collapsing every external value into `string` or `opaque`.
