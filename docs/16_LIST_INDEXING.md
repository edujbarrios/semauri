# Indexed access to typed lists

The binary `at` operator retrieves a value from an immutable typed list. Indexes are **zero-based**:

```sema
Let values be a list of 5, 9, 13.
Let selected be values at 1.
Print selected.
Print (values at 1) plus 3.
Assert (values at 1) is equal to 9.
```

`list<T> at number` returns `T`. Other combinations, including indexing strings directly, fail with type-check diagnostic `S345`. An index must evaluate to a non-negative integer; fractional, negative and out-of-range indexes fail with diagnostic `S346`, including a source location. Fully static expressions are checked by the compiler's existing compile-time evaluation path, so an invalid literal index may be rejected during compilation.

`at` binds at comparison precedence, not as tightly as arithmetic operators. For arithmetic using the **retrieved element**, use parentheses: `(values at 1) plus 3`. Without parentheses, `values at 1 plus 2` indexes element 3.

All lists remain immutable. The operator adds no external effects, performs no implicit casts and does not evaluate arbitrary runtime-produced lists; it is a foundation for further collection operations.
