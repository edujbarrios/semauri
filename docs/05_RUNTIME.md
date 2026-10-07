# Runtime planning

Semauri separates **compilation**, **planning** and **execution**.

Operations whose result cannot exist at compile time lower into an immutable `RuntimePlan`; the compiler does not execute them merely because it compiled the source.

## Runtime values

A non-`unit` domain operation can produce a typed value consumed by a later operation:

```text
%1 : ml.dataset = ml.open_dataset("./images")
%2 : ml.model = ml.load_model("resnet18")
%3 : ml.training_run = ml.train(%2, %1, ...)
```

Each result is represented by `RuntimeValueRef` with a stable producer ID, semantic type and source span.

Inspect the plan with:

```bash
semauri plan program.sema
```

## Compile-time/runtime boundary

Compile-time-known expressions are evaluated during lowering. Runtime references may flow directly between semantic-domain operations.

Runtime-dependent arithmetic, comparisons, `If` conditions and iteration are currently rejected with `S335`. Supporting those constructs correctly requires runtime control-flow representation rather than pretending the value is already known.

The intended progression is:

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

## Relationship to effects

`plan` answers **what would execute**.

`effects` answers **what capabilities the source may require**.

`explain` answers **how the compiler interpreted the source**.

Planning grants no capabilities and performs no declared effects.

## Current execution boundary

The CLI can execute filesystem operation plans through `semauri run` after explicit capability authorization. ML operations are planning-only: they produce typed runtime graphs but are not executed by the reference compiler.

Future execution support should consume the runtime plan rather than bypassing it.
