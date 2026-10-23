# Median of typed numeric lists

`median of` computes the median of a concrete, finite `list<number>`, returning `number`. It is pure and does not sort or mutate the input list.

```sema
let readings be a list of 12, 3, 100, 9, 7.
let center be median of readings.
assert center is equal to 9.
print center.
```

For an odd-length list, the result is the middle value in sorted order. For an even-length list, it is the arithmetic midpoint of the two middle values. Integer-only comparisons are exact even beyond `2^53`; even-sized pairs of integers use widened `i128` addition to avoid overflow, preserving exact `i64` results when the midpoint is an integer. Fractional midpoints and mixed number lists use `f64` and can round at large magnitudes.

The release also preserves exact signed `i64` values under unary `minus` in both evaluation and constant folding, rather than rounding through `f64`; overflow is diagnosed as `S358` at evaluation.\n\nThe operand must be `list<number>` (invalid types produce located diagnostic `S353`). Empty lists or non-finite values produce `S357` on evaluation. Current source syntax cannot construct empty list literals. This operator is a prefix expression binding to one primary: write `median of (reverse of readings)` to compose. `median` becomes a reserved keyword.

```sh
semauri check examples/median.sema
semauri run --allow console_write examples/median.sema
```

The operation applies to ordinary typed core-language numeric lists, not live tensor values, datasets, or unverified ML training results.
