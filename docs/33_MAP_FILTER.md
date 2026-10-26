# Lexically scoped typed list pipelines

Semauri 0.24.0 supports two pure expressions operating on compiler-known homogeneous lists:

- `map every ELEMENT in LIST to EXPRESSION` applies an expression to every element. It infers `list<T> -> list<U>` from the static type of the body.
- `filter every ELEMENT in LIST if PREDICATE` retains elements whose predicate evaluates to `boolean`, with type `list<T> -> list<T>`.

```sema
let readings be a list of 9, 1, 5, 3, 8.
let selected be filter every reading in readings if (reading is greater than 3).
let transformed be map every reading in selected to (reading times 2).
print sorted of transformed.
```

Each ELEMENT is an immutable lexical binding scoped to the body. It can shadow an outer variable with the same spelling but is never visible after the expression. Outer immutable bindings remain visible. Each body is evaluated once per element, in order, and the input is evaluated once; source lists are never mutated. Empty input or no passing items yields a typed empty output.

Input must be `list<T>` (type error `S365`). The filter predicate must be `boolean` (type error `S366`); a map body must produce a value rather than `unit`. Existing names `map` and `filter` remain valid bare variables, because these operators are **contextual syntax** followed by `every`.

To avoid precedence surprises, parenthesize LIST when it is an expression and BODY when it contains operations, particularly comparisons. A body may use core-language arithmetic, list operations, Boolean conditions and outer bindings. This is **not a generic lambda/function system**, and the compiler cannot run effects, live dataset handles, arbitrary runtime tensors or model training in map/filter bodies. The operations evaluate concrete core-language lists during compilation, rather than emitting a runtime streaming pipeline. No new capabilities are granted.

```sh
semauri check examples/map_filter.suri
semauri run --allow console_write examples/map_filter.suri
```
