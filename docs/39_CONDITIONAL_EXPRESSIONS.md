# Pure typed conditional expressions

The expression `if CONDITION then CONSEQUENCE otherwise ALTERNATIVE` chooses exactly one value while maintaining static type safety.

```sema
define function absolute taking n as number to (if (n is less than 0) then (minus n) otherwise n).
let scores be a list of minus 4, 2, minus 1.
let magnitudes be map every score in scores to (invoke absolute to score).
print magnitudes.
```

Both branches must have the **same static type**, including identical list element types (`S376` on disagreement). The condition must be Boolean (`S375` otherwise). Both branches are checked by the HIR builder, but **only the selected branch is evaluated**. That permits `if true then 1 otherwise (1 divided by 0)` safely. The optimizer preserves the same laziness when folding a known condition.

Syntax is intentionally explicit and distinguished from the existing statement form `if EXPR: ... end.`. The `then` word is contextual and remains valid as an identifier except where required as a delimiter. Within another expression, parenthesize conditional values: `let result be (if flag then 4 otherwise 5).` Compound condition/branch expressions should be wrapped in parentheses for clarity.

Conditionals are pure compiler-known expression selectors, not general runtime-dependent branching or a substitute for executing live ML model training.

```sh
semauri check examples/conditional_expressions.sema
semauri run --allow console_write examples/conditional_expressions.sema
```
