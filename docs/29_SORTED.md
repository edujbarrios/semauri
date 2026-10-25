# Typed immutable sorting — `sorted of`

`sorted of` takes a concrete `list<number>`, `list<string>` or `list<boolean>` and returns a **new list of the same type** in ascending order. The original binding and contents remain unchanged. It is pure and requires no capabilities.

```sema
let readings be a list of 10, 2, 8, 4.
let ordered be sorted of readings.
assert (ordered at 0) is equal to 2.
assert (readings at 0) is equal to 10.
```

- **Numbers:** ascending numerical order. Finite `i64` values compare exactly even beyond `2^53`; mixed integer/float comparisons are also exact with respect to the represented numeric values, avoiding `f64` rounding of integers for comparisons.
- **Strings:** Rust Unicode scalar lexicographical order, case-sensitive, not language-specific locale or natural sorting.
- **Booleans:** `false` before `true`.
- Sorting is stable; equal elements preserve their relative input order. An empty typed list (if supplied by a future runtime) sorts to an empty list, though current list literals require elements.

Operands of any other type produce located `S359` during type checking. Non-finite numeric list members are rejected at evaluation with `S359`. Results are not evaluated over external ML dataset/tensor handles.

The same precision-safe comparator also strengthens `median of` when a list mixes large integers and floating-point values. Parentheses combine prefix expressions: `sorted of (reverse of readings)`. The word `sorted` is reserved from Semauri 0.22.0 onward.

```sh
semauri check examples/sorted.sema
semauri run --allow console_write examples/sorted.sema
```
