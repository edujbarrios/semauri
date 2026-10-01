# Runtime planning

Semauri separates **compilation**, **planning** and future **execution**.

The compiler may encounter semantic-domain operations whose result cannot exist at compile time, such as a future HTTP request, model load, dataset open or training job. Those operations are not executed by the compiler. They lower into an immutable `IR::RuntimePlan`.

## Operation values

A domain operation may declare a non-`unit` return type. Such an operation can appear inside an expression:

```text
Within remote:
  Let response be Fetch "https://example.test".
  Consume response.
End.
```

The compiler lowers the program conceptually to:

```text
%1 : remote.response = remote.fetch("https://example.test")
remote.consume(%1)
```

`%1` is an `IR::RuntimeValueRef`. It has a stable producer ID and semantic type, but no concrete value during compilation.

Inspect the plan with:

```bash
ruby bin/semauri plan program.sema
```

## Compile-time/runtime boundary

Compile-time expressions continue to use the existing evaluator. Runtime values may currently flow directly between semantic-domain operations.

Runtime-dependent arithmetic, comparisons, `If` conditions and iteration are intentionally rejected with `S335` in 0.6.5. Supporting them correctly requires a runtime control-flow representation rather than pretending the compiler already knows the value.

The planned progression is:

```text
Typed HIR
  ↓
runtime operation/value discovery
  ↓
SSA-like values
  ↓
CFG / basic blocks
  ↓
runtime optimization
  ↓
capability validation
  ↓
execution backend/runtime
```

## Relationship to effects

`plan` answers **what would execute**.

`effects` answers **what capabilities that program may require**.

`explain` answers **how the compiler interpreted the source and why it produced that semantic plan**.

These views are generated from compiler-owned semantic structures. Planning never grants capabilities and never performs the declared effects.

## AI direction

The runtime boundary is required before Semauri can model useful AI programs. Operations such as model loading, dataset access, inference, training and checkpointing naturally produce values that do not exist at compile time.

A future ML program may lower to a plan resembling:

```text
%1 : ml.dataset = ml.load_dataset("./images")
%2 : ml.model = ml.load_model("resnet18")
%3 : ml.training_run = ml.train(%2, %1)
ml.save_checkpoint(%3, "./checkpoints/best")
```

The compiler can type-check, inspect, explain and capability-check this graph before a PyTorch or other execution runtime performs expensive work.

## Current non-goals

0.6.5 does not yet provide:

- runtime `If`/loops
- runtime arithmetic on operation values
- a process executor
- HTTP or ML built-in domains
- mandatory execution capabilities
- scheduling/distributed execution

Those features should build on the runtime plan rather than bypass it.
