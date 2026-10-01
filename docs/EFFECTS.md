# Effects and capabilities

Semauri separates **what a program may do** from **what an execution environment authorizes it to do**.

Semantic-domain operations declare stable effect identifiers in Typed HIR. Examples include:

```text
filesystem_read
filesystem_write
network
model_download
gpu_compute
model_training
checkpoint_write
```

Effects are descriptive compiler metadata. Merely compiling a source file does not execute them.

## Static effect analysis

`Compiler#effect_analysis` walks the unoptimized Typed HIR and produces a conservative manifest containing every declared effect plus its semantic provenance:

```text
effect
├── domain
├── operation
└── source span
```

The analysis intentionally runs before optimization. An effectful operation inside a branch that is currently constant-foldable still appears in the manifest. Capability policy should describe what the source program can request, not depend on a particular optimizer rewrite.

Inspect a program with:

```bash
ruby bin/semauri effects program.sema
```

## Capability policies

`Effects::CapabilityPolicy` is an immutable allow-list policy.

```ruby
policy = Semauri::Effects::CapabilityPolicy.allow(
  :filesystem_read,
  :filesystem_write
)

compiler.compile(source, capability_policy: policy)
```

A required effect that is not explicitly allowed fails with `S333`.

For CLI validation:

```bash
ruby bin/semauri check program.sema \
  --allow filesystem_read \
  --allow filesystem_write
```

To assert that a program is pure with respect to declared domain effects:

```bash
ruby bin/semauri check program.sema --allow-none
```

## Compilation versus execution

`build` does not require capabilities by default because compilation only produces semantic plans or backend output. It does not perform the declared external effects.

A future `run` command/runtime should enforce a capability policy before executing effectful operations. Runtime capabilities should be least-privilege and explicit.

This distinction is particularly important for future AI workloads. A training program may eventually declare capabilities such as:

```text
filesystem_read
network
model_download
gpu_compute
model_training
checkpoint_write
```

The compiler can explain and validate those requirements before an expensive or privileged workload starts.

## Domain author contract

A domain operation must declare every externally observable category of effect it may require. Effects should be stable semantic identifiers rather than backend-specific implementation details.

For example, a model training operation should declare `model_training` and `gpu_compute` when appropriate rather than something like `pytorch_cuda_call`.

Optimizers may transform pure expressions feeding an operation, but must preserve observable effectful operations unless a future effect-aware proof explicitly permits a transformation.
