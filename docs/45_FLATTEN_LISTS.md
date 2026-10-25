# One-level typed flattening — `flatten of`

The pure contextual expression `flatten of NESTED` transforms a concrete homogeneous `list<list<T>>` into a new `list<T>`, concatenating each nested list in source order. It flattens **exactly one level**, preserving inner element order and value representation (including integer versus binary64 decimal storage), without mutating the input.

```sema
let nested be a list of (a list of 2, 3), (a list of 4, 5).
let values be flatten of nested.
assert length of values is equal to 4.
assert first of values is equal to 2.
```

This composes with `partition every`: if `groups` has type `list<list<number>>`, `flatten of groups` produces a new `list<number>`. Unlike the original list, the flattened partition orders all matching elements before nonmatching elements. Typed empty inner lists are ignored, and an empty typed outer list produces an empty typed list. For deeper nested lists `list<list<list<T>>>`, the result is `list<list<T>>`; a second `flatten of` removes the next level.

Non-list and `list<T>` (where T is not itself a list) operands fail during static type checking with source-located **S383**. A malformed nested runtime list is also guarded by S383. The word `flatten` is contextual, so existing variables named `flatten` remain valid.

```sh
semauri check examples/flatten_lists.sema
semauri run --allow console_write examples/flatten_lists.sema
```

This is deterministic compilation of concrete typed list values, not a streaming tensor reshaping operation or a live ML data pipeline.
