# Semauri 0.9.0

Semauri 0.9.0 expands the ML semantic domain with explicit, typed controls for realistic training plans while preserving the deterministic compiler/runtime boundary.

## Advanced training plans

The existing `Configure training` form remains backwards compatible. A new `Plan training` operation adds precision, gradient accumulation and checkpoint cadence:

```text
Within ml:
  Let config be Plan training for 12 epochs using optimizer "adamw" learning rate 0.0003 batch size 16 seed 42 precision "bf16" accumulate 4 steps checkpoint every 250 steps.
End.
```

The operation produces the same opaque `ml.training_config` nominal type used by `Fit`, so advanced configuration composes with existing programs without introducing a second configuration type.

Supported precision values are `fp32`, `fp16` and `bf16`. Gradient accumulation and checkpoint intervals must be positive integers. Invalid statically-known advanced settings report diagnostic `S338`.

As with the rest of the ML domain, these controls describe an inspectable RuntimePlan. Compilation does not allocate accelerators, create checkpoints or execute a training framework.

## Compatibility

`Configure training`, `Train`, `Fit`, model construction, LoRA transforms, filesystem planning and all existing semantic domains retain their existing source forms.
