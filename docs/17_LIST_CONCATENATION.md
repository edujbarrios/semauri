# Immutable typed list concatenation

The existing `plus` operator now works on two lists with the **same element type**, producing a new list:

```sema
let first be a list of 10, 20.
let second be a list of 30, 40.
let joined be first plus second.
print joined.
print length of joined.
```

Lists remain immutable: concatenation creates a new list and does not modify either operand. The result preserves the common element type (`list<T> plus list<T> -> list<T>`). Combining a list with a scalar or with a list of another element type is a type error (`S314`).

Numeric addition and string concatenation continue to behave as before. This adds no new effects or coercions; list elements still follow the language's existing compile-time/runtime boundary.

Two type-compatible lists can be concatenated repeatedly, e.g., `first plus second plus third`. For indexing or length of the result, use parentheses where required by precedence.
