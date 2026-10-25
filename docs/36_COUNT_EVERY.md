# Predicate counting — `count every`

The scoped typed expression `count every ELEMENT in LIST if PREDICATE` returns a `number` containing the exact signed integer count of matching list elements. It behaves like `length of (filter every ELEMENT in LIST if PREDICATE)`, without allocating an intermediate filtered list.

```sema
define function positive taking n as number to (n is greater than 0).
let observations be a list of minus 1, 3, 0, 7.
let positives be count every n in observations if (invoke positive to n).
assert positives is equal to 2.
```

The compiler requires a homogeneous list (`S365` otherwise) and a statically typed Boolean predicate (`S366` otherwise). The iterator is a new scoped immutable binding; outer immutable variables remain visible and are not mutated. A matching count larger than `i64::MAX` would raise `S373` instead of wrapping, although the current concrete-list memory limit prevents realistically reaching that bound. Empty typed lists return integer zero. The input is evaluated once, and predicates execute once per element in original order. This operator is a contextual expression, so a variable named `count` remains valid when not followed by `every`.

The operation is pure and does not consume streaming datasets or perform runtime ML tensor processing.

```sh
semauri check examples/count_every.sema
semauri run --allow console_write examples/count_every.sema
```
