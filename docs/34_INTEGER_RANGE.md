# Bounded inclusive integer ranges

Semauri 0.24.0 supports `range from START to END`, a pure expression producing a typed `list<number>` of **integer values**, inclusive of both endpoints.

```sema
let ascending be range from 1 to 5.
let descending be range from 3 to 1.
let squares be map every x in ascending to (x times x).
```

Both operands must have static type `number` (otherwise diagnostic `S367`). Runtime evaluation requires stored `i64` integers rather than fractional or floating variants, and guards against generating more than **10,000 elements** with diagnostic `S368`. Large signed boundary subtraction uses widened `i128` arithmetic and cannot overflow; returned list items retain exact `i64` precision. Descending ranges are supported; identical endpoints produce a singleton. The size bound ensures no unbounded compile-time memory or CPU work.

`range` is contextual: programs may still use a bare variable named `range`. The operator is recognized by the sequence `range from`. Bound expressions bind only one numeric unary operand; use parentheses when needed, e.g. `range from (1 plus 1) to (3 times 3)`. Ranges work with `map every`, `filter every`, `sorted of` and other concrete typed-list operations.

No new execution capabilities, dataset streaming, runtime model training or tensor implementation is implied.

```sh
semauri check examples/range_map_filter.suri
semauri run --allow console_write examples/range_map_filter.suri
```

The input to a generator is evaluated as a pair of concrete integer endpoints. Its result can be passed to nested map/filter expressions without changing the immutability or effect policy of either operator.
