# Typed numeric extrema

Semauri 0.20.0 adds `minimum of` and `maximum of` for concrete `list<number>` values.

```sema
let scores be a list of 8, 3, 13, 5.
let smallest be minimum of scores.
let largest be maximum of scores.
assert smallest is equal to 3.
assert largest is equal to 13.
print largest.
```

These prefix expressions are statically typed `list<number> -> number`, read the next primary and require parentheses around larger operands (e.g. `minimum of (reverse of scores)`). They are pure, do not mutate the list, and require no extra runtime capability. Exact `i64`-to-`i64` comparisons preserve integer extrema beyond `2^53`. Mixed floating/integer comparisons use `f64` semantics and may round at extreme magnitudes. Ties retain the first value.

Non-numeric operands receive a located `S353` type diagnostic. Empty lists or non-finite floating inputs produce `S355` during evaluation (current list-literal syntax requires at least one element). As with `sum of` and `average of`, operations apply only to core-language numeric lists, not runtime-produced model tensors or datasets. The keywords `minimum` and `maximum` are reserved.

Run the example:

```sh
semauri check examples/numeric_extrema.sema
semauri run --allow console_write examples/numeric_extrema.sema
```
