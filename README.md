# Semauri

**SEMAURI — Semantic Expression Mapping and Unified Runtime Interpretation**

**Natural to write. Deterministic to run.**

> Status: **0.6.x / experimental**

Semauri is an experimental open-source programming language for writing deterministic programs with controlled natural language. The compiler parses a defined language, builds typed compiler structures, resolves meaning explicitly and lowers semantic programs to target backends.

Semauri is **not** an LLM wrapper and does not ask an AI model to guess what source code means.

## Install Semauri

Semauri is currently experimental, so the supported installation path is directly from the open-source repository.

### Requirements

- Ruby 3.2 or newer
- Git

### Install from source

```bash
git clone https://github.com/edujbarrios/semauri.git
cd semauri
ruby bin/semauri help
```

No runtime gem dependencies are required by the compiler core.

To update an existing checkout:

```bash
git pull
```

### Quick installation — planned

A release-based quick installer is planned once the language and CLI become stable enough to distribute safely. The intended experience is a small installer, for example through `curl` or an equivalent release mechanism, that installs a versioned Semauri CLI without requiring users to work directly inside the repository.

Until that exists, the repository is the canonical installation source. The project will not document a one-line remote installer before there is a versioned, reviewable installation path behind it.

## Use the language

Create a file named `shop.sema`:

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

Build it:

```bash
ruby bin/semauri build shop.sema
```

Inspect what the compiler understood:

```bash
ruby bin/semauri explain shop.sema
ruby bin/semauri hir shop.sema
ruby bin/semauri optimize shop.sema
ruby bin/semauri domains
```

The generated HTML for the example contains a red `Buy` button because `total` deterministically evaluates to `22` and the compiler selects the first branch.

## Editor support — planned

A **Visual Studio Code extension** is planned if the language/tooling architecture remains suitable for it. The intended path is to build editor support on top of compiler-owned information rather than duplicate language semantics inside the extension.

Potential tooling includes:

- syntax highlighting
- diagnostics with source spans
- hover information for semantic types and symbols
- go-to-definition and references
- semantic domain/operation inspection
- formatting
- safe rename
- HIR / compiled-plan inspection

A Language Server Protocol layer is the likely long-term integration point. This remains planned work, not a currently released extension.

## Why Semauri exists

Most natural-language programming approaches eventually delegate meaning to a probabilistic model. Semauri takes the opposite approach: natural-looking syntax is only accepted when the compiler can assign deterministic semantics to it.

The project is built around a few rules:

- natural syntax does not mean ambiguous semantics
- the compiler must fail instead of guessing
- no LLM is required by the deterministic compiler core
- compiling source must not silently perform external side effects
- semantic work happens before code generation
- new language features require tests and documented semantics
- extension points should remain explicit and maintainable

Created by **Eduardo J. Barrios** and open sourced from the beginning under the **Apache License 2.0**.

## A compiler with semantic domains

Semauri separates the language core from application-specific meaning through semantic domains.

```text
semantic domains
  ↓
lexer
  ↓
tokens
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
domain IR / operation plans
  ↓
backend / runtime
```

Built-in domains currently include Web, Structured Data and Filesystem. Domains can contribute vocabulary, nominal types, typed operations, effects, domain IR and default backends without adding domain-specific parser branches.

Example filesystem program:

```text
Within filesystem:
  Write "build metadata" to "build.txt".
  Copy "build.txt" to "backup.txt".
End.
```

The compiler builds a filesystem operation plan and may lower it to POSIX shell. Compilation itself does not execute those filesystem effects.

## Current features

- numbers, strings, booleans and colors
- immutable `Let` bindings
- homogeneous `List<T>` values and `For every` iteration
- arithmetic and parentheses
- typed comparisons
- `and`, `or`, `not` with short-circuit evaluation
- lexical scopes and shadowing
- stable semantic symbols
- typed HIR
- constant propagation/folding, dead control flow and dead-binding elimination
- `If / Otherwise / End`
- extensible semantic-domain registry
- explicit `Within <domain>: ... End.` semantic scopes
- multi-domain ProgramIR
- declarative domain operations with typed arguments and effects
- nominal semantic types such as `filesystem.path`
- Web domain with web documents, buttons, images and typed properties
- Structured Data domain with schemas, fields and domain-specific validation
- Filesystem operation domain that compiles plans to POSIX shell without executing filesystem effects during compilation
- automatic domain-to-backend selection
- HTML, JSON Schema and POSIX shell backends
- explicit references and constrained `it` resolution
- source-aware diagnostics

## AI-native direction

A major long-term direction for Semauri is to become a programming language for **AI and machine-learning workflows** while preserving deterministic compiler semantics.

The goal is not unrestricted prose that asks an LLM to generate training code. Instead, AI capabilities should be added as semantic domains with typed operations and explicit effects.

A future Semauri program could express concepts such as:

```text
Within ml:
  Load an image dataset from "./cats-vs-dogs".
  Create a cnn called Classifier for 2 classes.
  Train Classifier for 10 epochs using the dataset.
  Evaluate Classifier on the validation split.
End.
```

Or higher-level adaptation workflows such as fine-tuning a pretrained vision model while freezing selected components, applying LoRA/QLoRA where valid, configuring reproducibility, selecting hardware constraints and recording evaluation metrics.

The compiler should lower such source into a typed **training/inference plan** before any framework executes it. Backends or runtimes could then target PyTorch, other ML frameworks, local accelerators or remote execution environments.

Important design goals for an AI domain include:

- explicit dataset and model provenance
- typed model/dataset/tensor concepts
- reproducible seeds and configuration
- hardware/resource constraints
- declared training, filesystem and network effects
- dry-run and plan inspection before expensive execution
- checkpoint and artifact lineage
- framework-independent semantic IR where practical
- deterministic compilation even when the underlying numerical training process is not bit-for-bit deterministic

This direction will be introduced incrementally; it is not part of the stable language surface yet.

## Semauri as an explainability layer

Semauri also has potential as an **explainability and provenance layer for AI workflows**.

`explain` already reports the compiler's actual semantic decisions rather than generating a narrative after the fact. The same principle can extend to AI programs: Semauri can retain why a dataset was selected, which transforms were applied, what parts of a model were frozen, which parameters were trainable, which optimizer/settings were chosen, which effects were authorized and how an execution plan was lowered to a concrete runtime.

That would make Semauri useful for **XAI around the program and model lifecycle**: reproducibility, provenance, configuration transparency and execution traces. It should not be confused with a universal explanation of a neural network's internal reasoning. Model-level interpretability techniques can later be exposed as their own typed operations and analysis domains.

## Why the compiler is still Ruby

The reference compiler is written in **Ruby** and intentionally avoids runtime dependencies in its core.

At the current stage, rewriting the language in Rust would mainly trade development speed for implementation work without addressing the project's main bottleneck. Parsing, semantic analysis and HIR optimization are currently small compared with the cost of real external workloads such as model training, inference, filesystem operations or network execution.

Rust remains a strong option for future components where it provides a measurable benefit, for example:

- a sandboxed execution runtime
- high-performance or concurrent backends
- process/resource supervision
- native ML/runtime integrations
- portable standalone distribution
- performance-critical compiler stages if profiling eventually justifies them

The architecture is intentionally layered so individual components can be replaced without rewriting the language definition.

## Explain a program

```bash
ruby bin/semauri explain shop.sema
```

Example output includes symbol bindings, reference resolution, branch selection and domain mutations. The important property is that these explanations are derived from the compiler structures that actually produced the program.

## Development

Run the complete test suite:

```bash
ruby -Ilib -e 'Dir["test/test_*.rb"].sort.each { |file| require_relative file }'
```

CI tests Ruby 3.2, 3.3 and 3.4.

Important implementation choices include:

- handwritten recursive-descent parser
- immutable AST/semantic objects where practical
- lexical scopes and stable semantic symbol IDs
- typed expressions and strict comparisons
- deterministic reference resolution
- composable HIR optimization passes
- semantic domains injected through an explicit registry
- generic artifact/element/property/action parser categories rather than hardcoded domain nouns or verbs
- declarative typed domain operations with effect metadata
- nominal domain types with explicit HIR promotion boundaries
- per-domain semantic validation and default backend selection
- explicit ambiguity and cross-domain errors
- source spans and compiler diagnostics
- tests used as executable language specification

The compiler is developed incrementally so each stage remains understandable, testable and replaceable.

## Documentation

See:

- [Language](docs/LANGUAGE.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Semantic domains](docs/DOMAINS.md)
- [Nominal types](docs/NOMINAL_TYPES.md)
- [Universal domains](docs/UNIVERSAL_DOMAINS.md)
- [Roadmap](docs/ROADMAP.md)

## Contributing

Pull requests are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Copyright 2026 Eduardo J. Barrios.

Licensed under the **Apache License 2.0**. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
