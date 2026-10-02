# ML semantic domain

The built-in `ml` domain is Semauri's AI-native semantic domain. It describes machine-learning intent as typed compiler semantics before any framework or accelerator runtime executes the workload.

## Current types

```text
ml.dataset
ml.model
ml.device
ml.training_config
ml.training_run
ml.inference_run
```

ML runtime objects are nominal values. They are produced by semantic operations rather than forged from primitive strings.

## Load or construct a model

A model may come from an external identifier:

```text
Let model be Load model "resnet18".
```

or from Semauri model-construction semantics:

```text
Let model be Build cnn for 2 classes input channels 3.
```

Both produce `ml.model`, but their provenance remains different in the RuntimePlan.

CNN construction currently validates statically-known class and input-channel counts and rejects invalid values with `S337`.

## Immutable model transforms

Model configuration and fine-tuning preparation are represented as immutable derivations:

```text
Let model be Build cnn for 2 classes input channels 3.
Let frozen be Freeze model component "features".
Let adapted be Apply lora to frozen rank 16 alpha 32.
```

Conceptually:

```text
%1 : ml.model = ml.build_cnn(classes=2, input_channels=3)
%2 : ml.model = ml.freeze_component(%1, "features")
%3 : ml.model = ml.apply_lora(%2, rank=16, alpha=32)
```

The compiler does not mutate an in-memory neural network. Each operation produces a new typed model reference, making model lineage explicit and inspectable.

Current validation includes positive CNN dimensions, non-empty frozen component names, positive LoRA rank and positive LoRA alpha. Invalid model configuration reports `S337`.

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

No dataset is opened, model is loaded or training is started merely because the source file is compiled.

## Typed training configuration

For a more explicit training plan, create an opaque `ml.training_config`:

```text
Within ml:
  Let dataset be Open dataset "./images".
  Let model be Build cnn for 2 classes input channels 3.
  Let device be Select device "cuda".
  Let config be Configure training for 12 epochs using optimizer "adamw" learning rate 0.0003 batch size 32 seed 42.
  Let run be Fit model using dataset on device with config.
End.
```

`ml.training_config` must be produced by the ML domain. Current static validation includes:

- optimizer: `adam`, `adamw` or `sgd`
- positive integer epochs
- positive learning rate
- positive integer batch size
- non-negative integer seed

Invalid statically-known training configuration reports `S336`.

## Inspect before execution

Inspect a runtime plan:

```bash
ruby bin/semauri plan train.sema
```

Inspect required effects/capabilities:

```bash
ruby bin/semauri effects train.sema
```

The current ML domain may declare effects such as:

```text
filesystem_read
model_load
compute
model_training
model_inference
```

Model-construction and configuration operations are currently pure planning operations; training, inference and external access remain observable effects.

## Framework boundary

Semauri source describes semantic concepts rather than framework calls. A future runtime may implement the same plan using PyTorch, Transformers, ONNX Runtime, OpenVINO, JAX or another system.

Framework-specific behavior should be an explicit runtime/extension concern rather than silently defining the language semantics.

## Direction

The ML plan will continue to grow around typed concepts:

```text
model
├── architecture / source revision
├── frozen components
├── trainable components
└── adapters

dataset
├── provenance / fingerprint
├── split
└── transforms

training
├── optimizer / learning rate
├── precision
├── batch size / gradient accumulation
├── seed
└── checkpoint policy

resources
├── device
├── memory constraints
└── distributed strategy
```

Planned work includes richer CNN/model architecture definitions, additional adapter strategies such as QLoRA, dataset transforms/splits, evaluation metrics, checkpoint lineage and hardware-aware validation.

## Explainability and provenance

Because model derivations are explicit plan nodes, Semauri can preserve deterministic answers to questions such as:

- where did this model originate?
- which model value was frozen or adapted?
- which LoRA configuration produced the trained model input?
- which dataset/config/device flowed into training?
- what effects/capabilities are required?
- which backend/runtime ultimately executed the plan?

This is explainability of the **program and model lifecycle**. Model-internal interpretability such as Grad-CAM, Integrated Gradients, attention inspection or feature attribution should be added later as explicit typed analysis operations.

## Non-goals of 0.7.2

0.7.2 does not yet:

- import PyTorch or Transformers
- execute training or inference
- download models during compilation
- inspect real GPUs
- generate arbitrary CNN layer graphs
- implement QLoRA or arbitrary adapter targets
- persist checkpoints
- provide runtime `If`/loops over ML results

Those features should build on the typed RuntimePlan boundary rather than bypass it.
