# AI engineering case study: inspectable ML workflows

**Project:** Semauri, a statically typed Rust compiler for a readable programming language.

## Engineering problem

An ML workflow definition often conflates *what should run* with proof that data, weights and training actually ran. Semauri explores a compiler-driven separation: validate an ML plan and its dependencies before effects can be authorized, and export the result for external tooling. The design emphasizes static contracts, source-level diagnostics and reviewable machine-readable artifacts.

## Implemented architecture

A source file passes through lexing, recursive-descent parsing, typed HIR, name resolution, effect analysis and lowering. The ML domain describes dataset/model inputs, device selection, seeded training configuration and a training plan. `semauri graph` exports the planned operations as a deterministic, versioned typed dependency graph (SSA references and producer/consumer edges). `semauri audit` reports missing declared revisions and direct unseeded training forms. Explicit capability validation separates inspection from execution.

## Reproducible demo (no external model downloads)

The checked-in [ML example](../examples/ml_versioned_inputs.sema) has revision-qualified inputs and a training seed. Run from the project root after [building Semauri](DEVELOPMENT.md):

```sh
cargo test --test ml_portfolio_contract
cargo run -- check examples/ml_versioned_inputs.sema
cargo run -- effects examples/ml_versioned_inputs.sema
cargo run -- graph examples/ml_versioned_inputs.sema
cargo run -- audit examples/ml_versioned_inputs.sema --strict
```

The integration contract verifies five planned ML operations, typed producer dependencies, deterministic graph generation, revision metadata and a warning-free *declaration* audit. It also verifies that analysis does not confer permission to execute training. CI exercises this contract.

## Technical decisions

- **Static types before effects:** reject incompatible terms prior to backend execution.
- **SSA-like planning:** explicit edges are useful for lineage, inspection and future adapters.
- **Opt-in capabilities:** declaring a compute effect is not authorization to perform it.
- **Structured audit:** a stable JSON schema and nonzero strict-mode exit status support policy checks in CI.
- **Honest verification boundary:** declared revisions and seeds are not validated hashes, deterministic CUDA kernels or proof of model results.

## Scope and next steps

This is a compiler, validation and ML workflow planning project; it **does not train models, read model hubs or execute PyTorch/TensorFlow workloads**. Further production work would require execution adapters, artifact digest verification, dependency/environment locking, isolated workers, metrics capture and end-to-end training integration tests.

## CV-ready project description

> Built Semauri, a typed programming language and Rust compiler with a domain-aware ML planning layer. Implemented machine-readable dependency graphs, static effect analysis, revision-aware ML preflight audits and CI-tested reproducibility contracts. Focused on typed intermediate representations, explicit execution permissions and traceable workflow specifications.

Use this description only for contributions you can substantiate from the repository history, tests and code.
