# Core `Length of` expressions

Semauri supports a deterministic and typed unary `Length of` expression:

```sema
Let name be "Semauri".
Let count be Length of name.
Let values be a list of 10, 20, 30.
Print count.
Print Length of values.
Print Length of (name plus " rocks").
```

`Length of` accepts a **string** or a **list** and returns a number. For strings, length counts Unicode scalar values (Rust `char`s), not UTF-8 bytes or grapheme clusters. List length counts the outer elements. Using a number, boolean, color, or incompatible nominal value is a type error (`S343`).

Use parentheses to select an expression before computing its length: `Length of ("a" plus "b")`. The operator binds to a single primary expression otherwise, for example `Length of name plus 2` means `(Length of name) plus 2`.

Known constant lengths can be folded by the optimizer. Runtime-derived values remain restricted by the existing compile-time/runtime boundary; this feature does **not** introduce dynamic string operations or runtime expression evaluation for external domain results.
