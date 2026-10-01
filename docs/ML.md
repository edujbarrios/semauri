# ML semantic domain

Semauri 0.7 introduces an experimental `ml` semantic domain for describing AI and machine-learning workflows in controlled natural language.

The current domain is **plan-only**. Compilation validates the workflow and produces an immutable semantic ML plan. It does not download models, read datasets, allocate a GPU, train a network or write checkpoints merely because source code was compiled.

## CNN training plan

```text
Within ml:
  Import dataset "./cats-vs-dogs" as "training".
  Initialize cnn as "classifier" with 2 classes.
  Train "classifier" using "training" for 10 epochs.
  Evaluate "classifier" using "training".
  Save "classifier" to "./checkpoints/classifier".
  Explain "classifier" using "gradcam".
End.
```

`semauri build` currently lowers this program to the `ml-plan` JSON backend. The plan can be inspected before a future execution runtime is allowed to perform any effects.

## Pretrained model / fine-tuning plan

```text
Within ml:
  Import dataset "./birds" as "training".
  Use model "google/vit-base-patch16-224" as "classifier".
  Freeze "classifier" component "backbone".
  Train "classifier" using "training" for 8 epochs.
  Evaluate "classifier" using "training".
  Explain "classifier" using "integrated-gradients".
End.
```

The current representation validates semantic aliases and ordering. A model must be introduced before it can be frozen, trained, evaluated, saved or explained; a dataset must be imported before training/evaluation can reference it.

## Current operations

| Operation | Meaning | Declared effects |
| --- | --- | --- |
| `Import dataset SOURCE as NAME.` | Register a local dataset source | `filesystem_read` |
| `Initialize cnn as NAME with N classes.` | Define a CNN training target | none |
| `Use model SOURCE as NAME.` | Select a pretrained model source | `network`, `model_download` |
| `Freeze MODEL component COMPONENT.` | Mark a named component frozen | none |
| `Train MODEL using DATASET for N epochs.` | Add a training stage | `filesystem_read`, `gpu_compute`, `model_training` |
| `Evaluate MODEL using DATASET.` | Add an evaluation stage | `filesystem_read`, `gpu_compute`, `model_inference` |
| `Save MODEL to PATH.` | Add a checkpoint/artifact output | `filesystem_write`, `checkpoint_write` |
| `Explain MODEL using METHOD.` | Add a model-interpretability stage | `gpu_compute`, `model_inference`, `model_explanation` |

These effects are semantic requirements, not actions performed by the compiler. Inspect them with:

```bash
ruby bin/semauri effects train.sema
```

## Explainability

The ML domain treats explainability as two separate layers:

1. **program/workflow explainability** — Semauri can explain how the source resolved into models, datasets, stages, effects and outputs because those decisions exist explicitly in compiler IR;
2. **model interpretability** — methods such as Grad-CAM or Integrated Gradients are represented as typed/declared analysis operations and will later be executed by an ML runtime.

Semauri should not claim to recover a neural network's hidden reasoning. Its goal is reproducible, inspectable orchestration and explicit provenance around model behavior and interpretability tools.

## Why aliases are strings in 0.7.0

The first planning domain uses validated semantic names such as `"classifier"` and `"training"`. This lets the domain exercise the extension/effect/plan architecture without pretending runtime values already exist.

The runtime-value milestone will introduce first-class typed results such as:

```text
ml.dataset
ml.model
ml.checkpoint
ml.device
ml.tensor
ml.metrics
```

and operations that return symbolic values, for example conceptually:

```text
Let dataset be Load dataset "./birds".
Let model be Load model "google/vit-base-patch16-224".
Train model using dataset.
```

Those values must survive compilation as runtime references rather than being evaluated by the compiler.

## Runtime direction

A future execution pipeline is expected to look roughly like:

```text
Semauri source
  ↓
Typed HIR
  ↓
ML semantic plan
  ↓
Runtime IR / capabilities
  ↓
PyTorch (first target)
  ↓
execution trace + checkpoints + metrics + provenance
```

Compilation and execution remain separate. The future runtime should require an explicit capability policy before network access, GPU compute, training or filesystem writes occur.

## Planned ML evolution

- first-class `ml.dataset`, `ml.model`, `ml.device`, `ml.tensor` and `ml.checkpoint` types
- runtime operation results and SSA-style references
- dataset splits/transforms and typed preprocessing pipelines
- optimizers, schedulers, losses and metrics
- CNN architecture specification without coupling the language directly to `torch.nn`
- pretrained vision/VLM/LLM model descriptors
- LoRA/QLoRA and component-level fine-tuning policies
- hardware/memory planning and dry-run validation
- reproducibility metadata: seeds, revisions, hashes, framework/CUDA versions
- PyTorch execution backend/runtime
- execution tracing and artifact lineage
- typed interpretability operations such as Grad-CAM, Integrated Gradients and VLM token/region attribution
