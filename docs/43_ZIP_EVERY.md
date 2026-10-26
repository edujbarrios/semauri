# Typed elementwise zip mapping

`zip every LEFT in FIRST_LIST with RIGHT in SECOND_LIST to BODY` maps two concrete typed homogeneous lists element by element, binding two immutable scoped variables. The result has type `list<U>`, where `U` is inferred from `BODY`. The lists may have different element types. The output length is the **minimum input length**: trailing unmatched elements are discarded; an empty typed input yields an empty typed result.

```sema
let xs be a list of 1, 2, 3.
let ys be a list of 10, 20.
let sums be zip every x in xs with y in ys to (x plus y).
print sums.
```

Output is `[11, 22]`. Both sources are evaluated once, before any pair is processed. Each pair is handled in input order with fresh lexical bindings. The original lists and outer variables remain unchanged. The body may call earlier pure user functions; zips compose with `map every`, `filter every`, `reduce every` and `range from`.

Both operands must be typed lists (source-located `S383`). The result must be a concrete, non-unit value (`S384`). Duplicate iterator names are rejected as `S311`. `zip` is a contextual keyword only before `every`, so old `zip` identifiers remain valid. The word `with` separates the two input bindings in this construct, without becoming a reserved token. Parenthesize complex list operands.

Operations work with compiler-known values, not external dataset streams or ML tensor training. No additional effect permissions.

```sh
semauri check examples/zip_every.suri
semauri run --allow console_write examples/zip_every.suri
```
