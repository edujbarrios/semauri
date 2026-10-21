# Machine-readable CLI diagnostics

Semauri's `tokens`, `ast`, `hir`, `optimize`, `symbols`, `effects` and `plan` commands already print JSON on success. Supply `--json` to those commands to also receive **JSON on errors**:

```sh
semauri plan --json examples/shop.sema
semauri effects examples/filesystem.sema --json
```

For `check`, `--json` emits a result envelope on both success and failure:

```sh
semauri check examples/shop.sema --json
semauri check examples/filesystem.sema --allow-none --json
```

Successful check:

```json
{"ok": true, "status": 0, "diagnostics": []}
```

Failed check or inspection (illustrative data):

```json
{
  "ok": false,
  "status": 65,
  "diagnostics": [{
    "code": "S334",
    "kind": "semantic",
    "message": "A required capability was not authorized",
    "span": null,
    "hint": null
  }]
}
```

Exact messages and source spans depend on the compiler error. When present, `span` has `start` and `end` objects with 1-based `line` and `column` fields.

**Contract:** In JSON diagnostic mode, stdout contains a single JSON document, while the process exit status remains 64 for usage errors, 65 for compilation errors and 70 for runtime errors. Human-readable output remains unchanged without `--json`. The seven inspection commands retain their existing success payloads, not the check envelope.

The `diagnostics` array currently contains at most one error. It leaves room for future multi-error recovery without changing the JSON shape. `explain` remains human-readable.
