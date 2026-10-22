# Version-qualified ML inputs

Semauri supports explicit model and dataset revision declarations in its ML semantic domain:

```sema
Within ml:
  Let dataset be Source dataset "./images" revision "dataset-v3".
  Let model be Resolve model "resnet18" revision "weights-v2".
  Let device be Select device "cpu".
  Let config be Configure training for 3 epochs using optimizer "adamw" learning rate 0.0003 batch size 16 seed 42.
  Let run be Fit model using dataset on device with config.
End.
```

Both operations produce the existing nominal `ml.dataset` and `ml.model` types and preserve those type guarantees as runtime SSA references. The revision is a required, non-empty string stored verbatim in the runtime operation arguments. Empty source/identifier and revision strings fail at compile time with `S348`; non-string arguments fail with `S329`.

**Important trust boundary:** a declared revision is metadata, not a verified artifact digest or a snapshot. An execution adapter must resolve it against its storage system and verify the actual bytes before claiming reproducibility. The reference compiler does not download datasets, resolve a model hub, pin software dependencies, or run training.

The existing `Open dataset ...` and `Load model ...` syntax remains supported. The new operations keep the conservative `filesystem_read` and `model_load` capability declarations.

Inspect the declaration with `semauri plan FILE`, and its SSA dependencies with `semauri graph FILE` where available.
