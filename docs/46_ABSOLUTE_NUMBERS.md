# Exact absolute values

The pure contextual `absolute of VALUE` expression accepts a statically typed `number` and returns `number`. It works on both exact signed 64-bit integers and floating-point decimals:

```sema
let difference be minus 9007199254740993.
let positive be absolute of difference.
assert positive is equal to 9007199254740993.
```

Integer absolute values use checked `i64::abs` and **never pass through binary64**; values above `2^53` stay exact. An operand that is not numeric gets a source-located **S386** static error. Taking the absolute value of the signed 64-bit minimum or of a non-finite float emits source-located **S388** at evaluation. The optimizer folds safe literals while retaining invalid ones for runtime diagnostics.

`absolute` is a **contextual** operator only immediately before `of`: existing variables and labels named `absolute` remain valid. Parenthesize complex operands: `absolute of (a minus b)`. It composes with pure typed `map every` and functions. There are no new effect permissions or live ML tensor computations.

```sh
semauri check examples/absolute.suri
semauri run --allow console_write examples/absolute.suri
```
