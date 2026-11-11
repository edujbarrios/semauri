# Third-party Semauri source library example

This example demonstrates a **declared offline external dependency**. It
contains two independent directories:

- `app/`: the application with a `.semauri-root` marker, a dependency
  catalog, an entry point and an assertion test.
- `third-party/math-kit/`: an independently described complementary
  Semauri library providing reusable `square` and `triple` functions.

Run from the repository root (or use equivalent absolute paths):

```sh
semauri libraries examples/external_libraries/app
semauri check examples/external_libraries/app/main.suri
semauri run --allow console_write examples/external_libraries/app/main.suri
semauri test examples/external_libraries/app/tests
semauri verify examples/external_libraries/app
```

The program prints `15` after checking `square(7) = 49`. The external
library is selected from the catalog and imported `as math` in the app;
its
nested module import cannot leave its package root.

**Development-only:** this example also demonstrates `as math` and qualified
calls such as `invoke math.square to 7`. These namespaces are not part of
the currently published Semauri v0.72.0 binaries; build this branch to try them.

This does not download or install any packages: both directories are
included to make the example reproducible offline. See
[external-libraries.md](../../docs/external-libraries.md) for the manifest
schema, trust model, version validation and current limitations.
