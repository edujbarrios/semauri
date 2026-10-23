# Typed boolean list reductions

Semauri supports pure `any of` and `all of` expressions over concrete `list<boolean>` values.

```sema
let checks be a list of true, false, true.
let some_pass be any of checks.
let every_pass be all of checks.
assert some_pass is equal to true.
assert every_pass is equal to false.
```

Both operators have static type `list<boolean> -> boolean`, do not change the list, and require no new capabilities. `any of` is true when at least one value is true; `all of` is true when every value is true. For an empty list (if such a value is supplied by a future runtime), `any of` is false and `all of` is true by the standard identity rules. Current list-literal syntax does not permit empty literals.

A non-list or wrong-element-type operand receives diagnostic `S356` during type checking. The operators bind to the next primary; parenthesize complex operands such as `all of (reverse of checks)`. List elements are evaluated as normal before the reduction, so this is **not short-circuit evaluation** of effectful element expressions.

`any` and `all` become reserved words in Semauri 0.21.0 and cannot be bare variable identifiers. These operations act only on concrete, typed boolean lists, not framework tensors.

```sh
semauri check examples/boolean_reductions.sema
semauri run --allow console_write examples/boolean_reductions.sema
```
