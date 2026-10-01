# Semauri

**SEMAURI — Semantic Expression Mapping and Unified Runtime Interpretation**

**Natural to write. Deterministic to run.**

> Status: **0.7.x / experimental**

Semauri is an experimental open-source programming language for writing deterministic programs with controlled natural language. It is not an LLM wrapper: source code is parsed, typed, lowered and validated by a compiler with explicit semantics.

## Install Semauri

Semauri is still experimental, so the supported installation path is directly from the open-source repository.

### Requirements

- Ruby 3.2 or newer
- Git

### Install from source

```bash
git clone https://github.com/edujbarrios/semauri.git
cd semauri
ruby bin/semauri help
```

The compiler core has no runtime gem dependencies.

To update an existing checkout:

```bash
git pull
```

### Quick installation — planned

A versioned quick installer is planned once the CLI and release process are stable enough. The intended experience is a small `curl`-style or equivalent installer backed by reviewable releases.

Until then, the repository is the canonical installation source.

## Use the language

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

### Build

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

`total` evaluates to `22`, so the compiler deterministically selects the first branch.

### Explain the same program

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

`explain` reports semantic decisions produced by the compiler itself; it is not an AI-generated narrative.

## Compiler inspection

Useful commands include:

```bash
ruby bin/semauri hir shop.sema
ruby bin/semauri optimize shop.sema
ruby bin/semauri symbols shop.sema
ruby bin/semauri effects shop.sema
ruby bin/semauri domains
```

Programs containing runtime operations can also be inspected with:

```bash
ruby bin/semauri plan program.sema
```

Planning never executes the declared operations.

## How Semauri is built

The reference compiler is written in **Ruby** and intentionally keeps the core dependency-free.

```text
semantic domains
  ↓
lexer
  ↓
recursive-descent parser
  ↓
AST
  ↓
scopes + symbols + type checking
  ↓
typed HIR
  ↓
optimization passes
  ↓
ProgramIR / RuntimePlan
  ↓
backend / future runtime
```

Important implementation choices include stable semantic symbols, nominal domain types, immutable IR where practical, source spans, deterministic ambiguity errors, composable optimization passes, effect analysis, capability policies and explicit compiler extension points.

The compiler remains in Ruby because compilation speed is not currently the project bottleneck. Rust remains a possible future choice for sandboxed execution, process/resource supervision, standalone distribution or any component where profiling demonstrates a concrete benefit.

## Semantic domains

Semauri keeps application-specific meaning outside the core language through semantic domains. Built-in domains currently include:

- **Web** — web documents and elements, lowered to HTML
- **Structured Data** — schemas and fields, lowered to JSON Schema
- **Filesystem** — typed filesystem operations, lowered to POSIX shell plans
- **ML** — typed AI/ML dataset, model, device, training and inference runtime plans

Domains can contribute vocabulary, nominal types, typed operations, effects and semantic IR without adding hardcoded domain branches to the lexer/parser.

## AI / ML direction

Semauri 0.7 introduces the first built-in `ml` semantic domain.

The compiler can now represent typed concepts such as:

- `ml.dataset`
- `ml.model`
- `ml.device`
- `ml.training_run`
- `ml.inference_run`

and lower model loading, dataset access, device selection, training and inference intent into an inspectable `RuntimePlan`.

**No ML framework is executed during compilation.** PyTorch, Transformers, ONNX Runtime, OpenVINO or other systems belong behind future execution backends/runtimes.

The next AI milestones include structured training configuration, CNN/model construction, fine-tuning strategies such as LoRA/QLoRA, checkpoint lineage, resource planning, evaluation and interpretability operations.

See [docs/ML.md](docs/ML.md) for the current ML contract.

## Explainability

Semauri treats explainability as a compiler/runtime property rather than generated prose.

The existing semantic trace can be extended to AI workloads to preserve model and dataset provenance, transforms, trainable/frozen components, hyperparameters, effects, runtime selection, checkpoints and execution traces.

This provides explainability of the **program and model lifecycle**. Model-internal interpretability techniques such as Grad-CAM, Integrated Gradients, feature attribution or attention inspection should be exposed as explicit typed analysis operations rather than claimed automatically by the compiler.

## Effects and capabilities

Semantic operations declare effects such as filesystem access, model loading, compute or model training. Semauri can inspect these requirements before execution:

```bash
ruby bin/semauri effects program.sema
```

Capability policies can validate an explicit allow-list. Compilation itself never grants permissions or silently performs external effects.

## Current language/compiler features

- immutable `Let` bindings
- numbers, strings, booleans, colors and homogeneous `List<T>`
- arithmetic, comparisons and boolean logic
- lexical scopes and shadowing
- `If / Otherwise / End`
- `For every ... in ...`
- stable semantic symbols
- typed HIR
- constant propagation/folding and dead-code optimizations
- explicit semantic-domain scopes with `Within <domain>: ... End.`
- nominal semantic types
- multi-domain ProgramIR
- runtime operation values with SSA-like references
- static effect analysis and capability policies
- deterministic reference/ambiguity handling
- source-aware diagnostics

## Editor support — planned

A **Visual Studio Code extension** is planned if the tooling architecture remains viable. The intended integration point is a future Language Server Protocol implementation backed by compiler-owned semantics.

Potential features include syntax highlighting, diagnostics, hover types, go-to-definition, references, safe rename, formatting and HIR/runtime-plan inspection.

## Development

Run the complete test suite:

```bash
ruby -Ilib -e 'Dir["test/test_*.rb"].sort.each { |file| require_relative file }'
```

CI tests Ruby 3.2, 3.3 and 3.4.

## Documentation

- [Language](docs/LANGUAGE.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Semantic domains](docs/DOMAINS.md)
- [Nominal types](docs/NOMINAL_TYPES.md)
- [Effects and capabilities](docs/EFFECTS.md)
- [Runtime planning](docs/RUNTIME.md)
- [ML semantic domain](docs/ML.md)
- [Universal domains](docs/UNIVERSAL_DOMAINS.md)
- [Roadmap](docs/ROADMAP.md)

## Contributing

Pull requests are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Copyright 2026 Eduardo J. Barrios.

Licensed under the **Apache License 2.0**. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
