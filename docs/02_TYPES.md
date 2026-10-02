# Semauri type system

Semauri uses semantic types to keep controlled natural-language programs deterministic across domains.

## Primitive types

The current primitive types include:

- `number`
- `string`
- `boolean`
- `color`
- `List<T>`

Primitive types are structural: two `string` values have the same type regardless of where they came from.

## Nominal domain types

Domains may define values that share a primitive representation but have different semantic meaning.

```ruby
PATH = Semauri::Semantics::NominalType.new(
  domain: :filesystem,
  name: :path,
  base_type: :string
)
```

This produces the type:

```text
filesystem.path
```

A future HTTP domain may define:

```text
http.url
```

with the same `string` base representation. The two types remain incompatible:

```text
filesystem.path != http.url
```

Nominal identity is based on the domain, type name and base representation. Semauri does not use the base representation as an implicit equivalence relation between nominal types.

## Explicit promotion

For source ergonomics, an operation may accept a primitive expression at a nominally typed boundary when the primitive exactly matches the nominal type's declared base representation.

Source:

```text
Write "hello" to "notes.txt".
```

Filesystem declares the `path` argument as `filesystem.path`, whose base representation is `string`. Typed HIR therefore makes the conversion explicit:

```text
domain_operation filesystem.write : unit
├── content : string = "hello"
└── path : filesystem.path
    └── promote string -> filesystem.path
        └── "notes.txt"
```

The promotion is a compiler node, not an invisible coercion. Tooling and optimization passes can observe the semantic boundary.

A nominal value is never automatically reinterpreted as another nominal type, even when both types have the same primitive base.

## Assignment rules

`TypeSystem.assignment_kind(actual, expected)` currently returns:

- `:exact` when the types are identical;
- `:promote` when `expected` is nominal and `actual` exactly equals its base type;
- `nil` otherwise.

Examples:

```text
string          -> filesystem.path   promote
filesystem.path -> filesystem.path   exact
filesystem.path -> http.url          invalid
http.url        -> filesystem.path   invalid
number          -> filesystem.path   invalid
```

These rules deliberately avoid transitive or best-effort coercion.

## Runtime representation

Promotion changes semantic type identity without changing the underlying value representation. A `filesystem.path` currently carries a Ruby string at compile/lowering time, but its `Semantics::Value#type` remains `filesystem.path`.

This distinction is important for future runtime IR. Backends may choose a native representation appropriate to the target while the compiler retains the nominal contract.

## Domain declaration

Domains register nominal types explicitly:

```ruby
super(
  name: :filesystem,
  types: [PATH],
  operations: [...]
)
```

Registered types are exposed by `semauri domains` and by `Domains::Definition#type` / `#types`.

Operation signatures may reference the type object directly:

```ruby
Operation.expression(:path, type: PATH)
```

The domain registry and operation metadata serialize nominal types structurally, so IDEs and external tooling do not need to parse a display string such as `filesystem.path`.

## Optimizer contract

Nominal promotion is pure, but it is semantically meaningful.

Optimization passes may:

- propagate constants into a `promote` node;
- remove an unused promotion when its entire pure expression is dead;

but they must not erase a live nominal boundary and silently replace it with the primitive value.

## Design direction

Nominal domain types are intended to support values such as:

```text
filesystem.path
http.url
http.response
sql.connection
sql.rowset
vision.image
ml.model
ml.tensor
cloud.resource
```

The purpose is not to create a large inheritance hierarchy. The purpose is to make cross-domain contracts explicit enough that Semauri can compose many programming domains without treating every external value as `string` or `object`.
