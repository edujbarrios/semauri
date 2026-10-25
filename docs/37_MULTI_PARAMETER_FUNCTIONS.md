# Multi-parameter pure typed functions

Semauri supports reusable **top-level**, pure functions with **1–16 parameters**, each annotated by its expected type. Return types are inferred from the function expression.

```sema
define function weighted taking score as number, weight as number, bias as number to ((score times weight) plus bias).
let result be invoke weighted to (3, 2, 5).
assert result is equal to 11.
```

For one parameter, the previous `invoke square to 4` remains supported. A parenthesized argument tuple `invoke weighted to (3, 2, 5)` passes multiple arguments. A parenthesized single expression `invoke square to (2 plus 3)` passes one argument; commas belong at the top level of the argument tuple, so `invoke size to (a list of 1, 2)` passes one list argument. Functions support nested types such as `list of number`.

Parameters are immutable, lexically scoped, and bound together in a fresh invocation frame **after** evaluating all arguments once from left to right. Functions may capture preceding immutable outer variables and invoke previously defined functions. Calls work inside `map every`, `filter every`, and `count every`; pure functions require no capability grants. The compiler statically rejects nonmatching types with `S370` and the wrong argument count with the new source-located `S374`. Duplicate names within the same parameter list are rejected with `S311`. The parser rejects more than 16 parameters/arguments or malformed tuples with `S246`/`S247`.

No zero-argument functions, variadic signatures, forward references, recursive calls, first-class callable values, effectful bodies, or general runtime-dependent function execution are added. This remains deterministic compilation of concrete values; it does not train models or execute external datasets.

```sh
semauri check examples/multi_parameter_functions.sema
semauri run --allow console_write examples/multi_parameter_functions.sema
```
