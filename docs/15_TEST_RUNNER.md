# Semauri assertion test runner

Use the first-class test command to execute assertion-based `.sema` tests:

```sh
semauri test examples/test_suite
semauri test examples/test_suite --allow-console
semauri test path/to/one_test.sema
```

The command recursively discovers regular `.sema` files in a directory, skips symbolic links, sorts paths deterministically and prints a PASS/FAIL line for each file followed by a summary. Exit status is 0 when all tests pass, 1 when any test fails, and 64 for invalid arguments or an empty path.

Each test file must execute at least one `Assert` statement. Other allowed console statements include `Print`, but only if `--allow-console` is explicitly provided. Programs with filesystem, network, ML or other non-console effects are rejected during preflight, even when those operations reside in unreachable source branches. No build artifacts are permitted. A test fails on compilation errors and failed assertions (`S502`).

The runner reuses the same compiler and authorized console execution path as `semauri run`, rather than a separate interpreter. It does **not** run arbitrary project hooks, allow arbitrary capabilities, or execute source code outside the supported console runtime. Test files are not isolation sandboxes; the source and the directory tree should still be trusted.

To write tests:

```sema
let answer be 6 times 7.
assert answer is equal to 42.
assert not false.
```

Future testing features may add fixtures, named tests and test coverage once typed functions and runtime control flow exist. This initial command is deliberately small and effect-safe.
