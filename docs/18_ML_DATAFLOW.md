# Validated ML runtime dataflow graphs

Semauri can export a **typed, deterministic SSA dependency graph** for its runtime operations:

```sh
semauri graph examples/ml_workflow.suri
semauri graph --json examples/ml_workflow.suri
```

The `Compiler::runtime_graph` API and `RuntimePlan::dataflow_graph` produce the same JSON schema (`runtime_dataflow_graph`, `schema_version: 1`). Nodes include operation ID, domain, operation name, sorted direct dependencies, typed arguments, optional output value, effects and source span. Edges identify the producing and consuming operations, the receiving argument and the SSA value reference.

The dataflow validator rejects missing, forward, cyclic or otherwise malformed producer IDs, mismatched SSA value IDs, unassignable reference types, duplicate/out-of-order operation IDs and inconsistent unit/result typing with diagnostic `S347`.

Use the graph as an intermediate representation for visualization, planning, dependency analysis or future execution backends. **It does not execute training, load datasets, check whether model artifacts exist, or grant any effect capabilities.** The graph is a declaration of dependencies and required effects, not evidence that the work has completed.

Example:

```sema
within ml:
  let dataset be open dataset "./data".
  let model be load model "resnet18".
  let device be select device "cpu".
  let run be train model using dataset on device for 3 epochs.
end.
```

The graph captures dataset, model and device producers as distinct inputs to the training node. Machine-readable consumers should rely on `producer`, `consumer`, `argument`, and `type` rather than source-code formatting.
