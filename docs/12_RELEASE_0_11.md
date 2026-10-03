# Semauri 0.11.0 — practical workflows

Semauri 0.11.0 focuses on making the existing language useful for coherent automation rather than only isolated demonstrations.

## Filesystem workflow vocabulary

The built-in `filesystem` domain now supports:

```text
Make directory "build".
Write "content" to "build/file.txt".
Append "more" to "build/file.txt".
Copy "build/file.txt" to "build/copy.txt".
Move "build/copy.txt" to "build/archive.txt".
Touch "build/complete.marker".
Delete "build/old.tmp".
```

All operations remain planning operations during compilation. They declare explicit filesystem effects and the default POSIX backend renders shell commands using escaped arguments.

`Append` writes the supplied string exactly and does not add an implicit newline. `Make directory` uses recursive/idempotent directory creation semantics in the POSIX backend.

## Why this release matters

The language already has typed expressions, bindings, control flow, collections, semantic domains, effects, HIR and runtime planning. 0.11 makes one domain substantially more useful end-to-end: a Semauri source file can describe a small file-generation or build workflow and compile it into an inspectable executable script.

Semauri is still experimental. General-purpose readiness requires an explicit runtime, runtime-dependent control flow, reusable typed procedures, structured values and broader I/O. Those are the next architectural milestones rather than reasons to bypass the existing type/effect model.
