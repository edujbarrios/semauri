# Semauri

**SEMAURI — Semantic Expression Mapping and Unified Runtime Interpretation**

**Natural to write. Deterministic to run.**

> Status: **0.12.0 / experimental**

## What is Semauri?

Semauri is an experimental open-source programming language based on **controlled natural language**. It combines an explicit grammar, typed HIR, lexical scopes, deterministic resolution, effects and semantic domains. It is not unrestricted English and it is not an LLM wrapper.

## What can it do today?

Semauri already has a useful deterministic subset:

| Area | Current capability |
| --- | --- |
| Core | immutable bindings, arithmetic, booleans, comparisons, lists, lexical scopes, `If`, static `For every` |
| Web | build typed web documents and render HTML |
| Structured data | build typed schemas and render JSON Schema |
| Filesystem | plan and explicitly execute write/append/copy/move/mkdir/touch/delete workflows |
| ML | plan typed datasets, models, devices, training, inference, LoRA/QLoRA and training configuration |
| Tooling | inspect HIR, symbols, effects, optimization and runtime plans; capability-gated `run` |

Compilation remains effect-free. Execution is a separate opt-in step and effectful programs must be authorized explicitly.

## Example: Semauri to HTML

The repository includes this example as `examples/shop.sema`:

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

Build it with:

```sh
semauri build examples/shop.sema
```

Semauri evaluates the deterministic control flow (`18 + 4 > 20`) and generates this HTML:

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

This is the key idea: the Semauri source is typed and deterministic, while the web backend turns it into ordinary HTML that can be opened by a browser.

## Example: practical filesystem automation

```text
Within filesystem:
  Mkdir "build".
  Write "hello from Semauri" to "build/notes.txt".
  Append " - deterministic automation" to "build/notes.txt".
  Copy "build/notes.txt" to "build/backup.txt".
  Move "build/backup.txt" to "build/archive.txt".
  Touch "build/complete.marker".
  Delete "build/old.tmp".
End.
```

Compile without effects:

```sh
semauri build examples/filesystem.sema
```

Or explicitly execute the supported filesystem runtime after authorizing its capabilities:

```sh
semauri run --allow filesystem_read --allow filesystem_write examples/filesystem.sema
```

Use `--dry-run` to inspect the executable plan without performing effects, and `--cwd PATH` to choose the execution directory.

## Direction

Semauri is not yet a complete general-purpose runtime language. With 0.12 it has its first explicit execution path; the next practical milestones are user-defined typed procedures/functions, runtime-dependent control flow, structured record/map values, HTTP/process domains, richer filesystem reads/queries, modules and a small standard library.

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
backend / explicit capability-gated runtime
```

## Documentation

Start with `docs/00_INSTALLATION.md`, then `docs/01_LANGUAGE.md`, `docs/02_TYPES.md`, `docs/03_DOMAINS.md`, `docs/04_EFFECTS.md`, `docs/05_RUNTIME.md` and `docs/06_ML.md`. Architecture, distribution and roadmap documents continue under `docs/`.

## Contributing

Contributions are welcome. See `CONTRIBUTING.md`.

## License

Copyright 2026 Eduardo J. Barrios.

Licensed under the **Apache License 2.0**. See `LICENSE` and `NOTICE`.
