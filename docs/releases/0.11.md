# Semauri 0.11 — practical workflows

Semauri 0.11 focuses on making the existing language useful for coherent automation rather than only isolated demonstrations. 0.11.1 corrects the directory-creation vocabulary collision present in 0.11.0.

## Filesystem workflow vocabulary

The built-in `filesystem` domain supports:

```text
Mkdir "build".
Write "content" to "build/file.txt".
Append "more" to "build/file.txt".
Copy "build/file.txt" to "build/copy.txt".
Move "build/copy.txt" to "build/archive.txt".
Touch "build/complete.marker".
Delete "build/old.tmp".
```

All operations remain planning operations during compilation. They declare explicit filesystem effects and the default POSIX backend renders shell commands using escaped arguments.

`Append` writes the supplied string exactly and does not add an implicit newline. `Mkdir` uses recursive/idempotent directory creation semantics in the POSIX backend.

## 0.11.1 correction

0.11.0 accidentally registered `make` as a filesystem-domain verb even though `make` is reserved by the core English vocabulary. That prevented the default vocabulary from initializing. 0.11.1 uses the unambiguous domain verb `mkdir` and adds regression coverage against the public compiler API.

## Why this release matters

The language already has typed expressions, bindings, control flow, collections, semantic domains, effects, HIR and runtime planning. 0.11 makes one domain substantially more useful end-to-end: a Semauri source file can describe a small file-generation or build workflow and compile it into an inspectable executable script.

Semauri is still experimental. General-purpose readiness requires an explicit runtime, runtime-dependent control flow, reusable typed procedures, structured values and broader I/O. Those are the next architectural milestones rather than reasons to bypass the existing type/effect model.
