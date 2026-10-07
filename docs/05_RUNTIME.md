# Runtime planning

Semauri separates compilation, planning and execution.

Operations whose result cannot exist at compile time lower into an immutable `RuntimePlan`. The compiler does not execute them merely because source code was compiled.

## Runtime values

A non-`unit` operation can produce a typed value consumed by later operations:

```text
%1 : ml.dataset = ml.open_dataset("./images")
%2 : ml.model = ml.load_model("resnet18")
%3 : ml.training_run = ml.train(%2, %1, ...)
```

Each result is represented by `RuntimeValueRef` with a stable producer ID, semantic type and source span.

```bash
semauri plan program.sema
```

## Compile-time/runtime boundary

Compile-time-known expressions are evaluated during lowering. Runtime references may flow directly between semantic-domain operations.

Runtime-dependent arithmetic, comparisons, `If` conditions and iteration are currently rejected with `S335`; those constructs require a future runtime control-flow representation.

```text
Typed HIR
  ↓
optimization
  ↓
runtime operations + typed references
  ↓
future CFG / basic blocks
  ↓
capability validation
  ↓
execution runtime
```

`plan` shows what would execute, `effects` shows required capabilities, and `explain` shows compiler interpretation.

The CLI executes authorized console and filesystem plans. ML and other runtime-value domains remain planning-only.

Console output is an explicit effect:

```text
Let answer be 6 times 7.
Print answer.
```

```sh
semauri run --allow console_write program.sema
```
