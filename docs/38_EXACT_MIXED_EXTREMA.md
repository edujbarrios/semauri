# Exact mixed-number minimum and maximum

When a typed numeric list mixes 64-bit integers and binary64 decimals, `minimum of` and `maximum of` use the same exact **transitive** finite comparator as `sorted of` and `median of`. This avoids rounding large integers to floating-point values before comparing.

```sema
let values be a list of 9007199254740992.0, 9007199254740993.
print minimum of values.
print maximum of values.
```

Expected values are decimal `9007199254740992.0` and the exact integer `9007199254740993` respectively. Integers remain integers in the returned result; decimals remain decimal values. The comparison also handles signed extremes and f64 values outside the i64 range by checking bounds before integer casts.

Both operators require a non-empty `list<number>` (the existing S353/S355 diagnostics apply), reject non-finite values and do not mutate the input list. For the detailed shared comparator and sorting semantics, see [immutable sorting](29_SORTED.md), [numeric extrema](26_NUMERIC_EXTREMA.md) and [median](28_MEDIAN.md).

The fix makes compiler-known numeric data more reproducible; it does not introduce tensor arithmetic or live ML computation.
