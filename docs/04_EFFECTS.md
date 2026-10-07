# Effects and capabilities

Semauri separates **what a program may do** from **what an execution environment authorizes it to do**.

Semantic-domain operations declare stable effect identifiers in Typed HIR. Merely compiling a source file does not execute those effects.

## Static effect analysis

`Compiler::effect_analysis` walks unoptimized Typed HIR and produces a conservative manifest with each effect, domain, operation and source span. Analysis intentionally happens before optimization, so a currently dead branch cannot hide a capability requirement.

Inspect a program with:

```bash
semauri effects program.sema
```

## Capability policies

`CapabilityPolicy` is an explicit allow-list:

```rust
use semauri::{CapabilityPolicy, Compiler};

let compiler = Compiler::new();
let policy = CapabilityPolicy::new([
    "filesystem_read",
    "filesystem_write",
]);

compiler.compile_with_policy(source, None, Some(&policy))?;
```

A required effect that is not allowed fails with `S334`.

For CLI validation:

```bash
semauri check program.sema \
  --allow filesystem_read \
  --allow filesystem_write
```

To require a program to have no declared effects:

```bash
semauri check program.sema --allow-none
```

## Compilation versus execution

`build` produces plans or backend output and does not perform declared external effects.

`run` currently executes supported filesystem plans only after explicit capability authorization. `--dry-run` renders the plan without performing effects.

Runtime capabilities should remain least-privilege and explicit as more execution runtimes are added.

## Domain-author contract

A domain operation must declare every externally observable effect category it may require. Effect identifiers should describe semantic behavior rather than framework-specific implementation details.

Optimizers may transform pure expressions feeding an operation, but must preserve observable operations unless a future effect-aware proof permits a transformation.
