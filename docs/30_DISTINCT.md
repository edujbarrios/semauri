# Typed stable list deduplication — `distinct of`

`distinct of` accepts a concrete homogeneous `list<T>` and returns another `list<T>` containing only the first occurrence of each equal element, preserving the input order. The original list is not mutated and no effects are granted.

```sema
let inputs be a list of 3, 1, 3, 2, 1.
let unique be distinct of inputs.
assert length of unique is equal to 3.
assert (unique at 0) is equal to 3.
let sorted_unique be sorted of unique.
print sorted_unique.
```

Deduplication uses Semauri's current **value equality for list membership** (the same semantics as `contains`), including nested typed lists. The representation of numbers matters: integer `1` and decimal `1.0` are different stored values and remain distinct, just as with list `contains` and `is equal to` on values. For an empty typed list, the result is empty; current literal grammar does not express an empty list.

An operand that is not a list is rejected statically with source-located `S360`. The implementation is deterministic, stable, and quadratic in the number of list elements, appropriate for current small compile-time lists; it does not stream or deduplicate external datasets. The word `distinct` becomes reserved in v0.22.0.

```sh
semauri check examples/distinct.suri
semauri run --allow console_write examples/distinct.suri
```

See [sorting](29_SORTED.md), [list membership](14_CONTAINS.md), and [the language reference](01_LANGUAGE.md).

For reproducible output, example data is evaluated once before deduplication, and source list values remain available unchanged for later statements. This pure transformation introduces no additional effect grants.
