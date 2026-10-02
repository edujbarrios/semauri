# Semauri

**SEMAURI — Semantic Expression Mapping and Unified Runtime Interpretation**

**Natural to write. Deterministic to run.**

> Status: **0.7.4 / experimental**

## What is Semauri?

Semauri is an experimental open-source programming language based on **controlled natural language**.

Its source code is designed to read like clear instructions, while still behaving like a real programming language: it has an explicit grammar, semantic types, lexical scopes, deterministic name/reference resolution, compiler diagnostics and well-defined lowering rules.

Semauri is **not unrestricted English** and it is **not an LLM wrapper**. The compiler does not guess what a program means. If a phrase is unsupported or ambiguous, compilation fails instead of choosing an interpretation probabilistically.

## Goal

Semauri explores a simple idea:

> Make programs easier to read and express without giving up deterministic compiler semantics.

The language separates general programming constructs from application-specific meaning. Core syntax handles values, bindings, expressions and control flow; **semantic domains** contribute concepts such as web elements, filesystem operations or ML models without forcing those concepts into the core grammar.

## What does a Semauri program look like?

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

This is natural-looking syntax, but every construct has explicit semantics:

- `Let` creates an immutable typed binding.
- `price plus tax` is a typed numeric expression.
- `Create a web` and `Add a button` are provided by the Web semantic domain.
- `If / Otherwise / End` is structured control flow.
- `the button called Buy` is an explicit deterministic reference.
- `color` accepts a typed `color` value rather than arbitrary text.

With the same Semauri version, source, domain registry and backend, the same valid program must resolve to the same semantic result.

## How does it work?

```text
Semauri source
    ↓
vocabulary + lexer
    ↓
recursive-descent parser
    ↓
syntax AST
    ↓
name resolution + type checking
    ↓
typed HIR
    ↓
optimization
    ↓
semantic/domain IR or RuntimePlan
    ↓
backend / execution runtime
```

The important boundary is that **meaning is resolved before a backend renders or a runtime executes anything**.

The compiler can therefore inspect the program at several levels (`hir`, `symbols`, `effects`, `plan`, `explain`) without asking an AI model to interpret the source.

### Core language

The current language includes:

- immutable `Let` bindings;
- `number`, `string`, `boolean`, `color` and homogeneous `List<T>` values;
- arithmetic, comparisons and boolean logic;
- lexical scopes and shadowing;
- `If / Otherwise / End`;
- `For every ... in ...`;
- explicit semantic scopes with `Within <domain>: ... End.`;
- deterministic reference and ambiguity handling;
- typed HIR, semantic symbols and source-aware diagnostics;
- static effect analysis and capability policies;
- runtime operation values represented through inspectable plans.

### Semantic domains

Application-specific concepts are added through semantic domains instead of expanding the core parser with special cases.

Built-in domains currently include:

| Domain | Purpose | Typical output / plan |
| --- | --- | --- |
| **Web** | web documents and elements | HTML |
| **Structured Data** | schemas and fields | JSON Schema |
| **Filesystem** | typed filesystem operations | POSIX shell plan |
| **ML** | datasets, models, devices, training and inference intent | RuntimePlan |

A domain can contribute vocabulary, nominal types, typed operations, effects and semantic IR while the core compiler keeps the same deterministic pipeline.

### Effects are planned, not silently executed

Compilation can describe operations such as filesystem access or ML execution and expose their effects, but compiling a Semauri program does not automatically perform those external effects. Runtime-dependent work is lowered into an inspectable `RuntimePlan`.

## Documentation — reading order

The documentation is numbered so the intended reading order stays obvious as the language grows:

1. [`00_INSTALLATION.md`](docs/00_INSTALLATION.md) — install Semauri, update it, or run the compiler from source.
2. [`01_LANGUAGE.md`](docs/01_LANGUAGE.md) — language grammar and implemented semantics.
3. [`02_TYPES.md`](docs/02_TYPES.md) — primitive and nominal semantic types.
4. [`03_DOMAINS.md`](docs/03_DOMAINS.md) — semantic-domain extension model.
5. [`04_EFFECTS.md`](docs/04_EFFECTS.md) — effects and capability policies.
6. [`05_RUNTIME.md`](docs/05_RUNTIME.md) — runtime planning and runtime values.
7. [`06_ML.md`](docs/06_ML.md) — built-in ML semantic domain.
8. [`07_UNIVERSAL_DOMAINS.md`](docs/07_UNIVERSAL_DOMAINS.md) — longer-term multi-domain architecture.
9. [`08_ARCHITECTURE.md`](docs/08_ARCHITECTURE.md) — compiler architecture and phase boundaries.
10. [`09_DISTRIBUTION.md`](docs/09_DISTRIBUTION.md) — release packages, private runtime and distribution model.
11. [`10_ROADMAP.md`](docs/10_ROADMAP.md) — implemented milestones and future direction.

When a new documentation topic is added, insert it where it belongs conceptually and renumber the sequence so the directory remains readable from top to bottom.

## Contributing

Contributions are welcome. See [`CONTRIBUTING.md`](CONTRIBUTING.md).

## License

Copyright 2026 Eduardo J. Barrios.

Licensed under the **Apache License 2.0**. See [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).
