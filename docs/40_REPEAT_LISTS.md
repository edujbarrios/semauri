# Typed bounded repetition

`repeat COUNT of VALUE` constructs a new homogeneous `list<T>` containing COUNT copies of a single evaluated value of type `T`. Nested list values are supported, and a count of zero gives a typed empty list. The original source data remains unchanged.

```sema
let base be a list of 1, 2.
let copies be repeat 3 of base.
assert length of copies is equal to 3.
let empty be repeat 0 of true.
assert length of empty is equal to 0.
```

The count is statically typed as `number` (wrong type is `S377`), but must **evaluate to an integral signed 64-bit number between 0 and 10,000 inclusive**, with `S378` on negative/fractional/excess counts. This follows the existing integer range generator's 10,000-element compile-time memory policy. The element is evaluated once and cloned, and the result retains an exact `list<T>` type; no mutation, I/O permissions or lazy streaming behavior are introduced.

The word `repeat` is contextual: it is recognized as a prefix operation only when followed by a count-like token, preserving ordinary uses of `repeat` as a variable name outside this syntax. Parenthesize compound operands, such as `repeat (1 plus 2) of (sorted of values)`.

```sh
semauri check examples/repeat_lists.sema
semauri run --allow console_write examples/repeat_lists.sema
```
