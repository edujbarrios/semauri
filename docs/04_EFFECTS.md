# Effects and capabilities

Semauri separates **what a program may do** from **what an execution environment authorizes it to do**.

Semantic-domain operations declare stable effect identifiers in Typed HIR. Compilation records effects but does not execute them.

## Static effect analysis

`Compiler::effect_analysis` walks unoptimized HIR and returns a conservative manifest with each effect, domain, operation and source span. This intentionally happens before optimization so capability requirements do not depend on an optimizer rewrite.

```bash
semauri effects program.sema
```

## Capability policies

```rust
use semauri::{CapabilityPolicy, Compiler};

let compiler = Compiler::new();
let policy = CapabilityPolicy::new([
    "filesystem_read",
    "filesystem_write",
]);

compiler.compile_with_policy(source, None, Some(&policy))?;
```

A required effect that is not explicitly allowed reports `S334`.

```bash
semauri check program.sema \
  --allow filesystem_read \
  --allow filesystem_write

semauri check pure.sema --allow-none
```

## Compilation versus execution

`build` produces plans or backend output without performing declared external effects.

`run` currently executes supported filesystem plans only after explicit capability authorization. `--dry-run` renders the plan without performing effects.

Domain authors must declare every observable effect category their operations can require. Optimizers may transform pure inputs but must preserve observable operations.
