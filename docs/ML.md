# ML semantic domain

The built-in `ml` domain is the first AI-native semantic domain in Semauri.

Its purpose is to describe machine-learning intent as typed compiler semantics before any framework or accelerator runtime executes the workload.

## Current types

```text
ml.dataset
ml.model
ml.device
ml.training_config
ml.training_run
ml.inference_run
```

These are nominal semantic types. ML runtime objects are not aliases for strings and cannot be fabricated accidentally from primitive values.

## Basic training and inference

```text
Within ml:
  Let dataset be Open dataset "./images".
  Let model be Load model "resnet18".
  Let device be Select device "cuda".
  Let run be Train model using dataset on device for 10 epochs.
  Let inference be Infer model on dataset using device.
End.
```

The program lowers to a runtime plan similar to:

```text
%1 : ml.dataset = ml.open_dataset("./images")
%2 : ml.model = ml.load_model("resnet18")
%3 : ml.device = ml.select_device("cuda")
%4 : ml.training_run = ml.train(%2, %1, %3, 10)
%5 : ml.inference_run = ml.infer(%2, %1, %3)
```

No dataset is opened, model is loaded or training is started merely because the source file is compiled.

## Typed training configuration

For a more explicit training plan, create an opaque `ml.training_config` value:

```text
Within ml:
  Let dataset be Open dataset "./images".
  Let model be Load model "resnet18".
  Let device be Select device "cuda".

  Let config be Configure training for 12 epochs using optimizer "adamw" learning rate 0.0003 batch size 32 seed 42.
  Let run be Fit model using dataset on device with config.
End.
```

Conceptually this becomes:

```text
%1 : ml.dataset = ml.open_dataset("./images")
%2 : ml.model = ml.load_model("resnet18")
%3 : ml.device = ml.select_device("cuda")
%4 : ml.training_config = ml.configure_training(
  epochs=12,
  optimizer="adamw",
  learning_rate=0.0003,
  batch_size=32,
  seed=42
)
%5 : ml.training_run = ml.fit(%2, %1, %3, %4)
```

`ml.training_config` is opaque: it must be produced by the ML domain and cannot be replaced with a string that merely looks like configuration.

Current static validation includes:

- optimizer must be `adam`, `adamw` or `sgd`
- epochs must be a positive integer
- learning rate must be greater than zero
- batch size must be a positive integer
- seed must be a non-negative integer

Invalid statically-known configuration is rejected with `S336` before execution.

Inspect a plan with:

```bash
ruby bin/semauri plan train.sema
```

Inspect required effects/capabilities with:

```bash
ruby bin/semauri effects train.sema
```

## Effects

The ML domain declares semantic effects such as:

```text
filesystem_read
model_load
compute
model_training
model_inference
```

These names describe observable workload categories rather than implementation details such as `torch.cuda` calls.

A future runtime can map semantic effects to concrete capabilities, resource policies and sandbox rules.

## Framework boundary

The ML language surface must remain framework-independent where practical.

Semauri source should describe concepts such as:

- dataset
- model
- device/resource target
- training configuration
- training and inference
- evaluation
- fine-tuning strategy
- checkpoint
- metrics
- provenance

A runtime/backend may then implement those concepts using PyTorch, Transformers, ONNX Runtime, OpenVINO, JAX or another system.

Framework-specific features may exist later, but they should be explicit extensions rather than leaking into the core semantics by default.

## Training model direction

The training plan will continue to grow around typed concepts rather than framework flags:

```text
model
├── architecture / source revision
├── frozen components
├── trainable components
└── adapters

dataset
├── source provenance
├── split
├── transforms
└── fingerprint

training
├── epochs / steps
├── optimizer
├── learning rate
├── precision
├── batch size
├── gradient accumulation
├── seed
└── checkpoint policy

resources
├── device
├── memory constraints
├── mixed precision
└── distributed strategy
```

The compiler should validate these plans before execution.

## Fine-tuning direction

Future Semauri should be able to represent operations such as:

```text
Freeze the vision encoder of model.
Apply LoRA to model with rank 16.
Fine tune model using dataset for 3 epochs.
```

These constructs should lower to a framework-independent semantic training plan first. A PyTorch/Transformers runtime would consume the plan afterwards.

## Explainability and provenance

ML planning should preserve enough provenance for Semauri to answer deterministic questions such as:

- which model revision was selected?
- which dataset and split were used?
- which transforms were applied?
- which parameters/components are trainable?
- what effects/capabilities are required?
- which backend/runtime lowered the plan?
- which seed, precision and hardware constraints were requested?

This is explainability of the **program and model lifecycle**. Model-internal interpretability techniques such as Grad-CAM, Integrated Gradients, attention inspection or feature attribution should be represented as separate typed analysis operations rather than claimed implicitly by the compiler.

## Non-goals of 0.7.1

0.7.1 does not yet:

- import PyTorch or Transformers
- execute training or inference
- download models
- inspect GPUs
- select optimal hyperparameters
- implement CNN architecture construction
- implement LoRA/QLoRA
- persist checkpoints
- provide runtime `If`/loops over ML results

Those features should build on the typed runtime-plan boundary rather than bypass it.
