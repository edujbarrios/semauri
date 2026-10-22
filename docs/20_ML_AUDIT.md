# ML experiment reproducibility preflight

Semauri can audit the declarations in a typed ML runtime plan **without executing training**:

```sh
semauri audit examples/ml_versioned_inputs.sema
semauri audit examples/ml_versioned_inputs.sema --strict
```

`semauri audit FILE` prints a JSON report with stable `schema_version: 1`, `status`, `verification`, `warning_count`, `warnings`, and `declared_inputs`. Each warning includes the operation ID and source location. For CI, `--strict` returns exit code **2** if any audit warnings are present and **0** if none are present. Parsing and semantic errors preserve the normal compiler exit status of 65. Without `--strict`, advisory warnings do not affect the exit status.

Current warning codes:

| Code | Condition |
| --- | --- |
| `ML001` | Legacy `open dataset` without an explicit revision |
| `ML002` | Legacy `load model` without an explicit revision |
| `ML003` | Direct `train ... for N epochs` form without a seeded training configuration |

Declare datasets using `Source dataset ... revision ...`, models using `Resolve model ... revision ...`, and use the seeded `Configure training ...` / `fit ... with config` forms to satisfy this declaration policy.

**Critical distinction:** `declarations_complete` and `verification: declarations_only` mean the compiler found these declarations. They do NOT establish verified artifacts, bitwise reproducibility, stable third-party implementations, deterministic accelerator kernels, fixed dependencies, or successful model training. The audit does not read external files, resolve model hubs or execute any planned effects.

The report is conservative and intentionally scoped to a small set of checks. It is suitable for preflight review and CI policy, not as a certificate of reproducible science.
