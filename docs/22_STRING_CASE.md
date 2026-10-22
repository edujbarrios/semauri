# Typed string case expressions

Semauri provides two new string expressions:

```sema
let language be "Semauri".
let uppercase_name be uppercase of language.
let lowercase_name be lowercase of uppercase_name.
print uppercase_name.
print lowercase_name.
assert lowercase_name is equal to "semauri".
```

The source language remains lowercase by convention; `uppercase of` and `lowercase of` change the **value** of a string, not the casing of Semauri source syntax.

- `uppercase of string` returns a new `string` containing its Unicode uppercase mapping.
- `lowercase of string` returns a new `string` containing its Unicode lowercase mapping.
- Applying either operator to a non-string value fails during semantic analysis with diagnostic `S351`.
- Literals are eligible for compile-time constant folding. The operations are pure and add no capabilities.
- Case mapping follows Rust's Unicode character mapping rules. Some characters can expand when uppercased (e.g. `"straße"` to `"STRASSE"`); results are **not locale-specific** and should not be used as security-sensitive identity normalization.
- The operator binds to the next primary expression. Parenthesize larger expressions, for example `uppercase of ("hello" plus " world")`.

No runtime-derived ML values are evaluated by these core operations; Semauri's existing typed runtime-plan limitations still apply.
