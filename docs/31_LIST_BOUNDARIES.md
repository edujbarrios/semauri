# Typed first and last list elements

Semauri supports the pure expressions `first of` and `last of` on concrete homogeneous `list<T>` values, producing the element type `T`. They are source-order selections, not sorting operations.

```sema
let readings be a list of 8, 3, 5.
let oldest be first of readings.
let newest be last of readings.
assert oldest is equal to 8.
assert newest is equal to 5.
```

Neither operation mutates the input or grants runtime permissions. They work on lists of any homogeneous element type, including nested lists. Compose transformations by parenthesizing: `first of (sorted of readings)`. A non-list operand fails static type checking with source-located `S361`. Evaluation of an empty typed list (only possible through future runtime data sources; literal syntax currently requires an element) produces source-located `S362`, rather than panicking. Both operator names are **contextual keywords**: `first of` and `last of` are parsed as expressions, but existing variables named `first` or `last` remain valid.

These are operations on concrete compiler-known lists; neither operator reads a dataset, executes a streaming pipeline or trains a model.

```sh
semauri check examples/list_boundaries.suri
semauri run --allow console_write examples/list_boundaries.suri
```
