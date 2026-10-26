# Stable typed list partition — `partition every`

`partition every ELEMENT in LIST if PREDICATE` evaluates a Boolean predicate over each element of a concrete homogeneous `list<T>` and returns `list<list<T>>` with exactly two typed sublists:

1. Index 0 contains values for which the predicate evaluates to `true`.
2. Index 1 contains values for which the predicate evaluates to `false`.

Both sublists preserve the original element order; an element occurs **exactly once** in one of the results. Input lists are not mutated and the whole expression is pure.

```sema
let values be a list of 3, 9, 1, 8.
let groups be partition every n in values if (n is greater than 5).
let high be groups at 0.
let low be groups at 1.
assert length of high is equal to 2.
assert first of low is equal to 3.
```

An empty typed input produces two typed empty lists, not a missing result. Predicates can use pure functions and enclosing immutable variables. Iterator bindings are lexically scoped and cannot leak outside the expression. The predicate is evaluated once per input value, from left to right. No intermediate Boolean mask or duplicate input list is produced; the two result buckets are created directly.

A non-list source fails type checking with `S365`, a non-Boolean predicate with `S366`. `partition` is a contextual word so old identifiers and free-form labels remain valid.

This is a transformation on compiler-known homogeneous list values, not a streaming split of a machine-learning dataset, neural-network training execution or an operation granting new capabilities.

```sh
semauri check examples/partition_every.suri
semauri run --allow console_write examples/partition_every.suri
```
