# Lowercase source code conventions

**Semauri source commands are lowercase by convention.** The compiler has always recognized keywords and registered domain operations without regard to capitalization. Starting in 0.17.0 the official teaching materials, example files and documentation use lowercase source syntax.

```sema
let answer be 6 times 7.
assert answer is equal to 42.
print "Hello from Semauri".
```

Existing `Let`, `LET`, `Print`, `PRINT` and similar programs remain valid: **there is no breaking grammar change**.

## Formatter

```sh
semauri fmt examples/hello_console.sema
semauri fmt examples/hello_console.sema --check
semauri fmt examples/hello_console.sema --write
```

- The default writes the formatted source to stdout without changing the file.
- `--check` returns 0 for already canonical source, 1 if a rewrite would be needed, and a compiler error code for invalid lexical input.
- `--write` rewrites the specified file in place only if its recognized syntax differs.
- `--check` and `--write` are mutually exclusive.

The formatter lowercases **only recognized language tokens**: built-in keywords, booleans, colors, and registered domain vocabulary. Unrecognized identifier tokens, case-sensitive free-form labels, numbers, quoted text, whitespace, and comments are left unchanged. It preserves Unicode and newline style. This makes the edit idempotent and minimizes changes to user data.

Note that an unquoted free-form phrase containing words also recognized as domain vocabulary is lexically ambiguous. For entirely case-sensitive data, use explicit quoted strings where the grammar permits them; review formatter diffs for any free-form name or label.

Only syntax spelling is normalized: Rust identifiers, GitHub workflow names, JSON schema keys, class names, system commands and quoted output are **not** forcibly lowercased. Those are different languages or data formats and changing their case would be unsafe.

## CI

Add `semauri fmt FILE --check` to a CI job to enforce lowercase commands in new source files. The formatter does not execute or authorize any program effects.
