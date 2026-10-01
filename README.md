# Semauri

**SEMAURI — Semantic Expression Mapping and Unified Runtime Interpretation**

**Natural to write. Deterministic to run.**

> Status: **0.7.x / experimental**

Semauri is an experimental open-source programming language for writing deterministic programs with controlled natural language. It is **not** an LLM wrapper: source code is parsed, typed, resolved and lowered through compiler-owned semantics instead of asking a model to guess what the program means.

Created by **Eduardo J. Barrios** and licensed under **Apache License 2.0**.

## Install

Semauri is experimental, so the canonical installation source is currently this repository.

Requirements:

- Ruby 3.2+
- Git

```bash
git clone https://github.com/edujbarrios/semauri.git
cd semauri
ruby bin/semauri help
```

The compiler core has no runtime gem dependencies.

A versioned quick installer (for example via `curl` or an equivalent release mechanism) is planned once releases and the CLI are stable enough to distribute safely.

## Use Semauri

Create `shop.sema`:

```text
Let price be 18.
Let tax be 4.
Let total be price plus tax.

Create a web called Pet Shop.
Add a button called Buy.

If total is greater than 20:
  Set the color of the button called Buy to red.
Otherwise:
  Set the color of the button called Buy to green.
End.
```

Compile it:

```bash
ruby bin/semauri build shop.sema
```

Output:

```html
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Pet Shop</title>
</head>
<body>
  <h1>Pet Shop</h1>
    <button style="color: red">Buy</button>
</body>
</html>
```

Inspect the compiler's semantic decisions:

```bash
ruby bin/semauri explain shop.sema
```

Output:

```text
1. Bound 'price' as symbol #1 to number 18.
2. Bound 'tax' as symbol #2 to number 4.
3. Variable 'price' resolved to symbol #1 (number 18).
4. Variable 'tax' resolved to symbol #2 (number 4).
5. Bound 'total' as symbol #3 to number 22.
6. 'web' resolved to an HTML web document (default web backend).
7. Title explicitly set to 'Pet Shop'.
8. Added button 'Buy' as button-1.
9. Variable 'total' resolved to symbol #3 (number 22).
10. If condition evaluated to true; selected consequence branch.
11. Explicit reference resolved to button 'Buy' (button-1).
12. Set button-1.color to "red".
```

`explain` is derived from the compiler structures that produced the program; it is not an AI-generated explanation.

Useful inspection commands:

```bash
ruby bin/semauri check shop.sema
ruby bin/semauri hir shop.sema
ruby bin/semauri optimize shop.sema
ruby bin/semauri effects shop.sema
ruby bin/semauri domains
```

## What is implemented

Semauri currently includes:

- handwritten lexer and recursive-descent parser
- immutable syntax/semantic structures where practical
- lexical scopes, stable symbol IDs and typed expressions
- `Let`, arithmetic, comparisons and boolean algebra
- `If / Otherwise / End`
- homogeneous `List<T>` and `For every`
- constrained pronoun/reference resolution with ambiguity errors
- Typed HIR and composable optimization passes
- constant propagation/folding, dead control flow and dead binding elimination
- source spans and structured diagnostics
- semantic-domain extension registry
- explicit `Within <domain>: ... End.` domain scopes
- multi-domain ProgramIR
- declarative typed domain operations
- nominal domain types such as `filesystem.path`
- static effect manifests and explicit capability policies
- Web, Structured Data, Filesystem and experimental ML semantic domains
- HTML, JSON Schema, POSIX shell and ML-plan JSON backends

The compiler pipeline is intentionally layered:

```text
source
  ↓
lexer / parser
  ↓
AST
  ↓
name resolution + types
  ↓
Typed HIR
  ↓
optimization
  ↓
semantic-domain IR / plans
  ↓
backend / future runtime
```

## AI / ML direction

Semauri 0.7 introduces an experimental **ML semantic planning domain**. It can describe and validate datasets, CNN definitions, pretrained model selection, component freezing, training, evaluation, checkpoint outputs and interpretability stages as an inspectable plan.

The important boundary is deliberate: **compilation does not train a model, download weights, allocate a GPU or write checkpoints**. Those operations declare effects such as `model_download`, `gpu_compute`, `model_training`, `model_inference` and `checkpoint_write`, which can already be inspected through `semauri effects` and capability policies.

The next AI milestones are first-class `ml.dataset`, `ml.model`, `ml.device`, `ml.tensor` and `ml.checkpoint` values; runtime operation results; hardware/memory planning; reproducibility metadata; and a PyTorch execution runtime.

See [docs/ML.md](docs/ML.md) for the ML language design and roadmap.

## Explainability and provenance

Semauri treats explainability at two levels:

1. **program/workflow explainability** — the compiler can explain symbol resolution, semantic operations, configuration choices, effects and lowering because these decisions are explicit in its IR;
2. **model interpretability** — techniques such as Grad-CAM, Integrated Gradients or VLM token/region attribution can be modeled as explicit analysis operations and executed later by an ML runtime.

This makes Semauri useful as an XAI/provenance layer around the model lifecycle without claiming that a compiler can magically recover a neural network's internal reasoning.

## Why Ruby

The reference compiler remains in **Ruby**. At this stage, compiler throughput is not the bottleneck; language semantics, runtime design and domain architecture are. A full Rust rewrite would slow iteration without materially improving ML workloads whose dominant costs will be I/O, model loading, GPU compute and training.

Rust remains a strong future option for a sandboxed runtime, process/resource supervision, standalone distribution or any compiler/runtime component where profiling demonstrates a real performance benefit. Python is a natural integration layer for an initial PyTorch runtime.

## Effects and capabilities

Domain operations declare their external effects. Semauri can inspect those requirements conservatively before execution:

```bash
ruby bin/semauri effects program.sema
ruby bin/semauri check program.sema --allow filesystem_read --allow filesystem_write
ruby bin/semauri check pure.sema --allow-none
```

`build` generates plans/code and does not perform those effects. A future `run` runtime will require explicit capability authorization before effectful execution.

## Editor support — planned

A **Visual Studio Code extension** is planned if the tooling architecture remains suitable. The intended implementation is an LSP backed by the compiler itself, providing syntax highlighting, diagnostics, semantic hover, go-to-definition/references, formatting, safe rename and HIR/plan inspection without duplicating language semantics in the editor extension.

## Development

Run the full test suite:

```bash
ruby -Ilib -e 'Dir["test/test_*.rb"].sort.each { |file| require_relative file }'
```

CI covers Ruby 3.2, 3.3 and 3.4.

Pull requests are welcome. New language/domain features should include tests, deterministic semantics, source-aware diagnostics and documentation. See [CONTRIBUTING.md](CONTRIBUTING.md).

## Documentation

- [Language](docs/LANGUAGE.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Semantic domains](docs/DOMAINS.md)
- [Nominal types](docs/NOMINAL_TYPES.md)
- [Effects and capabilities](docs/EFFECTS.md)
- [ML domain](docs/ML.md)
- [Universal domains](docs/UNIVERSAL_DOMAINS.md)
- [Roadmap](docs/ROADMAP.md)

## License

Copyright 2026 Eduardo J. Barrios.

Licensed under the **Apache License 2.0**. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
