# Semauri

**SEMAURI — Semantic Expression Mapping and Unified Runtime Interpretation**

**Natural to write. Deterministic to run.**

> Status: **0.11.0 / experimental**

## What is Semauri?

Semauri is an experimental open-source programming language based on **controlled natural language**. It combines an explicit grammar, typed HIR, lexical scopes, deterministic resolution, effects and semantic domains. It is not unrestricted English and it is not an LLM wrapper.

## What can it do today?

Semauri already has a useful deterministic subset:

| Area | Current capability |
| --- | --- |
| Core | immutable bindings, arithmetic, booleans, comparisons, lists, lexical scopes, `If`, static `For every` |
| Web | build typed web documents and render HTML |
| Structured data | build typed schemas and render JSON Schema |
| Filesystem | plan practical write/append/copy/move/mkdir/touch/delete workflows and render POSIX shell |
| ML | plan typed datasets, models, devices, training, inference, LoRA/QLoRA and training configuration |
| Tooling | inspect HIR, symbols, effects, optimization and runtime plans |

Compilation remains effect-free: filesystem and ML effects are described before execution rather than silently performed by the compiler.

## Example: practical filesystem automation

```text
Within filesystem:
  Make directory "build".
  Write "hello from Semauri" to "build/notes.txt".
  Append " - deterministic automation" to "build/notes.txt".
  Copy "build/notes.txt" to "build/backup.txt".
  Move "build/backup.txt" to "build/archive.txt".
  Touch "build/complete.marker".
  Delete "build/old.tmp".
End.
```

The filesystem domain lowers this program to a typed operation plan with explicit `filesystem_read` / `filesystem_write` effects. Its default backend can render a POSIX shell script without performing the operations during compilation.

## Direction

Semauri is not yet a complete general-purpose runtime language. The next practical milestones are explicit `semauri run` execution with capability enforcement, user-defined typed procedures/functions, runtime control flow, structured record/map values, HTTP/process domains, richer filesystem reads/queries, modules and a small standard library.

The design rule remains: add useful power without giving up deterministic semantics, inspectable effects or the compiler/runtime boundary.

## Architecture

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
backend / explicit execution runtime
```

## Documentation

Start with `docs/00_INSTALLATION.md`, then `docs/01_LANGUAGE.md`, `docs/02_TYPES.md`, `docs/03_DOMAINS.md`, `docs/04_EFFECTS.md`, `docs/05_RUNTIME.md` and `docs/06_ML.md`. Architecture, distribution and roadmap documents continue under `docs/`.

## Contributing

Contributions are welcome. See `CONTRIBUTING.md`.

## License

Copyright 2026 Eduardo J. Barrios.

Licensed under the **Apache License 2.0**. See `LICENSE` and `NOTICE`.
