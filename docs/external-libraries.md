# External and complementary Semauri libraries (v0.72.0)

Semauri can compile reusable third-party **`.suri` source libraries** from
outside a project, using an explicit dependency catalog. Libraries reuse the
existing static type checker, optimizer, import graph and effect/capability
enforcement. No package installation, network download, native code loading or
Python execution occurs during compilation.

This is the **initial offline library interface**, not a public package
registry, dependency downloader, lockfile or dynamic Rust/C ABI. Projects must
obtain and audit third-party sources separately.

## Namespaced imports (development, unreleased)

Give local source imports or external libraries an optional alias when you
want **functions with the same name** to coexist:

```text
import "library:math_kit" as math.
import "library:other_kit" as other.

let result be invoke math.square to 7.
print result.
```

For local sources, write `import "lib/helpers.suri" as helpers.`
and call `invoke helpers.function to VALUE`. Parenthesized calls with
multiple arguments work as usual. Import aliases are case-insensitive and
must begin with an ASCII letter and contain only letters, digits or
underscores (up to 64 characters). Aliases apply **only in the importing
file**; a nested module must declare any aliases it uses.

An aliased dependency gets a private internal **function namespace**, including
unaliased submodules. The same package can be imported under several aliases,
and two different packages can export identically named functions.
Unknown aliases or duplicates produce S416 diagnostics. Imports without an
alias keep their original global-function behavior.

**Current scope:** only function definitions and calls are namespaced.
Imported top-level values and executable statements are not isolated:
imports may still run effects, and those effects still require explicit
permissions. Ordinary path/version checks, symlink protection, and file
budgets are unchanged. The change is on the development branch: Semauri's
published language version remains **0.72.0** until a later release.

## 1. Declare the dependencies in the application

A project normally contains `.semauri-root` (created by `semauri init`).
In its directory, create or edit `.semauri-libraries.json`:

```json
{
  "schema": 1,
  "libraries": [
    {
      "name": "math_kit",
      "version": "1.2.3",
      "path": "../third-party/math-kit"
    }
  ]
}
```

The `path` is relative to the **project root**, not the importing source.
It may refer to a vendored directory inside the project or to an explicitly
authorized sibling directory outside it. Absolute paths are not accepted,
to keep project declarations portable. If no catalog exists, ordinary local
imports continue to work unchanged.

Library names are lowercase ASCII identifiers beginning with a letter, with
optional digits, underscores or hyphens, up to 64 bytes. Versions have exactly
three numeric components (`MAJOR.MINOR.PATCH`). The catalog supports up to
32 declared libraries, with a 64 KiB metadata limit.

## 2. The external library declares its own identity and entry

At `../third-party/math-kit/.semauri-library.json`:

```json
{
  "schema": 1,
  "name": "math_kit",
  "version": "1.2.3",
  "entry": "src/lib.suri"
}
```

The loader checks the name and version against the application's catalog
before importing it. The entry must be a real `.suri` file **within** the
library's resolved directory; a symlink cannot redirect it outside that root.

The library can have other local Semauri source modules:

```text
# src/lib.suri
import "core.suri".
define function triple taking n as number to (n times 3).
```

```text
# src/core.suri
define function square taking n as number to (n times n).
```

## 3. Import by declared name

In the application:

```text
import "library:math_kit".
print invoke square to 7.
print invoke triple to 5.
```

`library:` is a contextual import-path prefix, not a new reserved keyword.
The existing quoted-file form (`import "lib/math.suri".`) still works.
Imports must appear before executable statements.

Nested library-relative imports resolve inside their **own** library root.
Library files may also import other libraries by name, provided the application
declares each dependency. Duplicate files are loaded once, while cycles,
out-of-bounds paths and symlink escapes are rejected. Normal per-program
limits still apply (64 source files, 32 levels, 4 MiB total).

The current model merges imported definitions into one program, just as local
imports do. It does **not** yet provide namespaces or isolation against symbol
name collisions, or a stable binary plugin ABI.

## 4. Inspect and verify locally

```sh
semauri libraries app/main.suri
semauri check app/main.suri
semauri effects app/main.suri
semauri run --allow console_write app/main.suri
semauri test app/tests
semauri verify app
```

`semauri libraries PROJECT|FILE` validates the catalog, source identity,
versions, entry paths and existence and prints a JSON list of declared
libraries. Validation does not execute library code.

A library's side effects are **not trusted or permitted automatically**:
any imported console, filesystem, web or AI operations remain subject to the
same `--allow` capability policy as application code. `check` can inspect
libraries without running them. The compiler reports errors in imported
source files at their original file and line.

## Security and reproducibility boundaries

- Library locations must be explicit project declarations; arbitrary external
  `.suri` imports are still rejected.
- Libraries cannot import files outside their own canonical directory, even
  through symlinks. Application-relative imports remain project-confined.
- Invalid/missing metadata, version mismatches and undeclared aliases fail
  with `S417`; invalid local paths or import cycles report `S416`.
- **Version matching is metadata validation, not content integrity**.
  This phase does not pin a cryptographic hash, manage a lockfile, verify
  signatures, execute package hooks, or verify that a third-party source is
  benign. Review/pin the actual source independently before enabling effects.
- Rust-native plugins, shared libraries, remote package registry resolution
  and automatic dependency installation are **not supported yet**.
  Third-party Rust integrations can instead use Semauri's existing Rust
  compiler API and explicitly register semantic domains in a host application.

See the [repository example](../examples/external_libraries/) for a complete
application and complementary library.
