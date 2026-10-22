# Typed reverse expressions

Use `reverse of` with a string or homogeneous list:

```sema
let values be a list of 10, 20, 30.
let backwards be reverse of values.
assert backwards at 0 is equal to 30.
assert values at 0 is equal to 10.
print reverse of "Semauri".
```

- `reverse of string` returns a **new string** with its Unicode scalar values reversed. This is **not** grapheme-cluster-aware; combining accents and multi-codepoint emoji can be reordered.
- `reverse of list<T>` returns a **new list<T>**, preserving the element type and leaving the input list unchanged.
- The operand must be a string or list. Numbers, booleans, colors and nominal domain types fail static validation with diagnostic `S352`.
- `reverse of` is pure. Literal strings are constant-folded by the optimizer without introducing runtime effects.
- The expression binds to its next primary. Write `reverse of (some_list plus other_list)` to reverse an entire concatenation.

No framework dependencies are introduced, and runtime-derived ML reference values still cannot be used directly as inputs to pure core expressions.
