# Immutable typed list selection — `take` and `drop`

The expressions `take COUNT from LIST` and `drop COUNT from LIST` return a new `list<T>`, retaining element order and type while leaving the original input untouched. The count must evaluate to an integer `i64` greater than or equal to zero. The operand must be a typed concrete list.

```sema
let readings be a list of 9, 3, 1, 8.
let first_two be take 2 from readings.
let rest be drop 2 from readings.
assert length of first_two is equal to 2.
assert first of rest is equal to 1.
```

`take` selects up to COUNT elements from the beginning; `drop` skips up to COUNT elements. A zero or greater-than-length count is valid and **clamped**, so these operators can create typed empty lists even though source list literal syntax still requires one element. Nested typed lists are supported.

Type violations raise `S363` at compile time. Negative, decimal or otherwise invalid counts raise source-located `S364` when evaluated (the compiler statically distinguishes `number`, not integer versus decimal). The expression binds COUNT as the next numeric unary/primary and LIST as the next primary; parenthesize larger operands, e.g. `take (1 plus 2) from (reverse of readings)`. Keywords `take`, `drop` and `from` are newly reserved in v0.23.0.

All transformations are pure over compiler-known list values. They do not grant permissions or implement streaming ML dataset operations.

```sh
semauri check examples/list_windows.sema
semauri run --allow console_write examples/list_windows.sema
```
