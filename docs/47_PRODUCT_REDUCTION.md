# Checked typed numeric product

The pure contextual expression `product of LIST` multiplies every element of a statically typed `list<number>` and returns `number`.

```sema
let factors be a list of 2, 3, 4, 5.
let factorial be product of factors.
assert factorial is equal to 120.
```

Integer-only lists use **checked exact signed 64-bit multiplication**, preserving values greater than `2^53`. Overflow produces a source-located **S387** diagnostic instead of wraparound. For mixed/decimal lists, the result follows the language's `f64` arithmetic and is guarded against non-finite inputs and results (also S387). This may be subject to floating-point rounding like other mixed-number arithmetic.

A typed empty `list<number>` produces `1` (multiplicative identity). A non-numeric input is rejected statically with **S353**. Source lists remain immutable; no effect grants are added. `product` is contextual only before `of`; existing variables and labels named `product` remain valid. Parenthesize complex inputs, e.g. `product of (take 3 from values)`.

```sh
semauri check examples/product.suri
semauri run --allow console_write examples/product.suri
```

This is a concrete-list aggregation in the Rust compiler, not a live tensor multiplication or ML training implementation.

For integer-only input the product is evaluated in list order, and any checked intermediate overflow produces S387 even if a later element would have been zero. Decimal inputs use the existing binary64 representation.
