# Typed numeric list aggregations

Semauri can aggregate numeric data directly in readable, statically typed expressions:

```sema
let measurements be a list of 2, 4, 6.
let total be sum of measurements.
let mean be average of measurements.

assert total is equal to 12.
assert mean is equal to 4.
print total.
print mean.
```

- `sum of list<number>` returns a `number`. When every element is an integer, integer addition is **checked** for signed 64-bit overflow and preserves exact integer results, including integers above the precise `f64` range.
- `average of list<number>` returns a `number`, computed using floating-point arithmetic and the number of elements. Results can be fractional and are subject to floating-point rounding.
- Both expressions are pure and do not modify the source list or grant additional runtime capabilities.
- Lists of strings, booleans, colors and non-list operands are rejected by the type checker with diagnostic `S353`.
- Integer overflow, non-finite arithmetic and empty-list averages return a source-located evaluation error `S354`.
- Parenthesize larger expressions: `sum of (a plus b)`. The operators bind to the next primary and can compose with `reverse of`.

Limitations: Semauri's typed runtime-plan references cannot yet be aggregated as executable data. These functions process concrete core-language lists, not external ML tensor values. Floating-point operations are not compensated sums, so scientific workloads should not infer numerical-stability guarantees beyond the reference interpreter's `f64` arithmetic.
