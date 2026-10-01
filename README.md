# Semauri

**SEMAURI — Semantic Expression Mapping and Unified Runtime Interpretation**

**Natural to write. Deterministic to run.**

> Status: **0.7.x / experimental**

Semauri is an experimental open-source programming language for writing deterministic programs with controlled natural language. It is not an LLM wrapper: source is parsed, typed, lowered and validated by a compiler with explicit semantics.

## Install Semauri

### Linux and macOS

```bash
curl -fsSL https://raw.githubusercontent.com/edujbarrios/semauri/main/install.sh | sh
```

### Windows — PowerShell

```powershell
irm https://raw.githubusercontent.com/edujbarrios/semauri/main/install.ps1 | iex
```

The Windows installer supports native x86_64 and ARM64 packages, verifies the release checksum, installs Semauri under `$HOME\.semauri` and adds `$HOME\.semauri\bin` to your user `PATH`. Open a new terminal if the `semauri` command is not immediately visible.

Then use the language directly on every supported platform:

```text
semauri version
semauri help
```

**Ruby does not need to be installed on your system.** Official Semauri distributions include a private Ruby runtime used internally by the reference compiler.

Supported installation targets in 0.7.4:

- Linux x86_64
- Linux arm64
- macOS x86_64
- macOS arm64
- Windows x86_64
- Windows arm64

On Linux/macOS, the installer keeps versions under `~/.semauri/versions/` and exposes `~/.local/bin/semauri`. On Windows, versions live under `$HOME\.semauri\versions\` and a stable launcher is installed at `$HOME\.semauri\bin\semauri.cmd`.

Install an exact version on Linux/macOS:

```bash
SEMAURI_VERSION=0.7.4 \
  curl -fsSL https://raw.githubusercontent.com/edujbarrios/semauri/main/install.sh | sh
```

Install an exact version on Windows:

```powershell
$env:SEMAURI_VERSION = '0.7.4'
irm https://raw.githubusercontent.com/edujbarrios/semauri/main/install.ps1 | iex
Remove-Item Env:SEMAURI_VERSION
```

Re-run the installer to update to the latest release. See [Distribution](docs/DISTRIBUTION.md) for package layouts, versioning and the release process.

### Development from source

Only contributors working directly on the compiler need Ruby:

```bash
git clone https://github.com/edujbarrios/semauri.git
cd semauri
ruby bin/semauri help
```

The reference compiler currently supports Ruby 3.2+ and has no runtime gem dependencies in its core.

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
semauri build shop.sema
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
semauri explain shop.sema
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

```bash
semauri hir shop.sema
semauri optimize shop.sema
semauri symbols shop.sema
semauri effects shop.sema
semauri domains
```

Programs containing runtime operations can be inspected with:

```bash
semauri plan program.sema
```

Planning never executes the declared operations.

## How Semauri is built

The reference compiler is implemented in **Ruby**, but that is an implementation detail of the official distribution rather than an end-user dependency.

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

Official packages combine this compiler with a pinned private runtime:

```text
Semauri distribution
├── compiler source (Ruby)
├── private portable Ruby runtime
├── launcher: semauri
└── license / manifest metadata
```

This lets the compiler keep Ruby's development velocity while users interact only with `semauri`. Rust remains a possible future choice for sandboxed execution, resource supervision or other components where profiling/security requirements justify it.

## Semantic domains

Semauri keeps application-specific meaning outside the core language through semantic domains. Built-in domains currently include:

- **Web** — web documents and elements, lowered to HTML
- **Structured Data** — schemas and fields, lowered to JSON Schema
- **Filesystem** — typed filesystem operations, lowered to POSIX shell plans
- **ML** — typed AI/ML dataset, model, device, configuration, training and inference runtime plans

Domains can contribute vocabulary, nominal types, typed operations, effects and semantic IR without adding hardcoded domain branches to the lexer/parser.

## AI / ML direction

Semauri 0.7 includes a built-in `ml` semantic domain with typed concepts such as `ml.dataset`, `ml.model`, `ml.device`, `ml.training_config`, `ml.training_run` and `ml.inference_run`.

The compiler can already lower dataset/model loading, CNN construction, device selection, typed training configuration, model freezing, LoRA adaptation, training and inference intent into an inspectable `RuntimePlan` with explicit model lineage.

**No ML framework is executed during compilation.** PyTorch, Transformers, ONNX Runtime, OpenVINO or other systems belong behind future execution runtimes.

Next AI milestones include richer architecture definitions, dataset transforms/splits, checkpoint lineage, hardware/resource planning, evaluation metrics, QLoRA and model-interpretability operations.

See [ML semantic domain](docs/ML.md).

## Explainability

Semauri treats explainability as a compiler/runtime property rather than generated prose. The semantic trace and RuntimePlan can preserve model/data provenance, model derivations, trainable/frozen components, hyperparameters, effects, runtime selection, checkpoints and execution traces.

This provides explainability of the **program and model lifecycle**. Model-internal techniques such as Grad-CAM, Integrated Gradients, feature attribution or attention inspection should be explicit typed analysis operations rather than claims generated by the compiler.

## Current language/compiler features

- immutable `Let` bindings
- numbers, strings, booleans, colors and homogeneous `List<T>`
- arithmetic, comparisons and boolean logic
- lexical scopes and shadowing
- `If / Otherwise / End`
- `For every ... in ...`
- stable semantic symbols and typed HIR
- constant propagation/folding and dead-code optimizations
- explicit `Within <domain>: ... End.` semantic scopes
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
- [Distribution](docs/DISTRIBUTION.md)
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
