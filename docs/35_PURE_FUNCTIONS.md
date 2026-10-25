# Pure typed user-defined functions (v0.25.0)

A function is a reusable, **single-argument pure expression** with an explicit parameter type and an inferred return type. It may refer to earlier immutable top-level bindings, call functions declared earlier in the source, and be used inside map/filter list pipelines.

```sema
define function square taking n as number to (n times n).
let scores be a list of 2, 3, 4.
let squares be map every x in scores to (apply square to x).
print squares.
```

The supported signature form is `define function NAME taking PARAM as TYPE to EXPRESSION.`; the call is `apply NAME to PRIMARY`. Parenthesize compound arguments, e.g. `apply square to (2 plus 3)`. Types are `number`, `string`, `boolean`, `color`, or `list of TYPE` (recursively nested). The result type is derived from the statically checked body. One argument is currently supported: there are **no variadic parameters, function values, recursion, mutual recursion or forward references**. Define functions at top level before use. The `define`, `function`, `taking`, `as` and `apply` words are contextual, rather than breaking existing identifiers.

The compiler records a typed function-call HIR node, carrying its checked body and parameter symbol. Evaluation binds the argument once in a fresh lexical environment, executes the pure body, and returns a value of the inferred type. Both original bindings and list input data remain unchanged. Calls over runtime-only values are not implemented; this is compile-time-known data transformation, not arbitrary runtime execution.

## Diagnostics

- `S246`: malformed function definition or type annotation;
- `S247`: malformed function call syntax;
- `S369`: function not declared earlier;
- `S370`: function argument type mismatch;
- `S371`: duplicate function definition;
- `S372`: a function declaration outside the top-level scope, or a function body using domain effects (including I/O or ML operations).

Pure calls require no new permissions. Any effectful operation invoked from a function is rejected; outer effectful top-level statements are subject to the existing capability policy.

```sh
semauri check examples/functions.sema
semauri run --allow console_write examples/functions.sema
```
