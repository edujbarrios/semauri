# Typed seeded list reduction — `reduce every`

`reduce every ITEM in LIST starting ACC as SEED to BODY` computes a single result from a typed list, evaluating the list and seed once and visiting elements in their original order. `ITEM` has the list element type; `ACC` has the seed type. The result of `BODY` **must have exactly the seed type** at compile time; no implicit coercion is introduced. Each step uses its own lexical scope with immutable bindings, so the source list and outer values are unchanged.

```sema
let numbers be range from 1 to 5.
let sum be reduce every item in numbers starting acc as 0 to (acc plus item).
assert sum is equal to 15.
```

Empty typed lists return the seed unmodified and evaluate no steps. Numeric, string, Boolean, nested list and other existing pure value types are supported by their corresponding typed operators. Call earlier-declared pure typed user functions from the step body. Parenthesize compound input and seed expressions, e.g. `reduce every item in (sorted of values) starting acc as (1 plus 2) to (acc plus item)`.

The syntax `reduce` is contextual only when followed by `every`: previous `reduce` identifiers and prose remain valid. `starting` and `as` are consumed as delimiters only within that construct.

Non-list operands produce located `S379`, mismatched seed/step types `S380`, and effectful bodies `S381`. Malformed syntax raises `S250`, and duplicate iterator/accumulator names raise `S311`. The evaluator only processes concrete, compiler-known lists; this is not a streaming dataset reduction or executed ML tensor operator.

```sh
semauri check examples/reduce_every.suri
semauri run --allow console_write examples/reduce_every.suri
```
