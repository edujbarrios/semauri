# Semauri 0.12.0 — explicit execution

Semauri 0.12 introduces the first opt-in execution path while preserving the compiler/runtime boundary.

## `semauri run`

Filesystem programs rendered by the `posix-sh` backend can now be executed explicitly:

```sh
semauri run --allow filesystem_write program.sema
semauri run --allow filesystem_read --allow filesystem_write program.sema
```

Every statically required effect must be authorized. Missing capabilities fail before execution using the existing capability-policy diagnostics. Compilation itself remains effect-free.

`--dry-run` prints the generated executable plan without performing effects. `--cwd PATH` chooses the working directory for relative filesystem paths.

The initial runtime deliberately supports only a single filesystem output. Web, schema and ML outputs continue to use build/planning paths until a domain-specific execution model is defined.

## Why this matters

Before 0.12, Semauri could describe and compile practical filesystem workflows but could not run them through the language tool itself. This release closes that loop without turning compilation into implicit execution or bypassing the effect system.

The next general-purpose milestones are typed procedures/functions, runtime-dependent control flow, structured values, process/HTTP domains and a deterministic standard library.
