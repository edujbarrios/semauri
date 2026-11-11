# Semauri — language and tooling guide

Full reference for the [short Quick Start](../README.md). Commands and examples cover Semauri v0.72.0; consult the current release notes for changes.

## Agent quick start

When generating or changing a Semauri program:

1. Use a `.suri` filename and lowercase language keywords.
2. End every statement with a period.
3. Declare a value before using it. Values are immutable and lexically scoped.
4. Run `semauri fmt FILE --check`, then `semauri check FILE`.
5. Run `semauri explain FILE` to understand the compiler's semantic decisions.
6. Inspect `semauri effects FILE` and `semauri plan FILE` before execution.
7. Grant only the effects the program needs when using `run`.
8. Use `semauri test PATH` for assertion-based test files.

Never invent syntax. Prefer a small checked program over a plausible-looking unsupported construct.

## Install and verify

Download the archive for your platform from the [latest release](https://github.com/edujbarrios/semauri/releases/latest), extract it, and put the binary on `PATH`.

```sh
semauri version
```

From a source checkout:

```sh
cargo build --release
cargo test
cargo run -- check examples/hello.suri
```

## Semauri in Jupyter Notebook / JupyterLab

This repository includes a native Rust, experimental
[Semauri Jupyter kernel](../integrations/jupyter/README.md).
The host Jupyter application is external; the Semauri kernel requires ZeroMQ development libraries.
After installing the Rust `semauri` executable on your `PATH`:

```sh
cargo build --release --manifest-path integrations/jupyter/Cargo.toml
./integrations/jupyter/target/release/semauri-jupyter install --user
jupyter lab
```

Open [the example `.ipynb`](../integrations/jupyter/examples/hello_semauri.ipynb)
and select **Semauri** as the notebook kernel. In a cell, enter:

```text
let numbers be range from 1 to 5.
let squares be map every n in numbers to (square of n).
print squares.
```

Expected output: `[1, 4, 9, 16, 25]`.

Notebook-only directives (on the first nonblank line) include `%%check`,
`%%explain` (also `--json`), `%%effects`, `%%plan`, `%%tokens`,
`%%ast`, `%%hir`, `%%optimize`, `%%symbols`, `%%graph` and
`%%fmt`. These inspect code without executing application effects.
`%%fmt` displays the canonical source without writing files.

**Each cell is a standalone Semauri program.** Bindings do not persist
between cells. Replay would repeat filesystem/AI effects; compiler-backed
persistent sessions are a separate future feature. Default permission:
`console_write` only. To grant additional effects set
`SEMAURI_ALLOW_EFFECTS` explicitly *before* starting Jupyter.
Set `SEMAURI_BINARY` to the executable's full path if needed.
No `anywidget` dependency is required for this integration.

## Start a new Semauri project

Create a ready-to-run project with reusable typed functions and tests:

```sh
semauri init my-project
cd my-project
semauri check main.suri
semauri run --allow console_write main.suri
semauri test tests --allow-console
```

The initializer creates `main.suri`, `lib/math.suri`,
`tests/math.suri`, `.semauri-root`, and a project `README.md`.
It requires a **new directory** and refuses to overwrite an existing
project. The entry and test files demonstrate safe imports, typed functions,
capability checks, and executable assertions.

## Understand programs with `check` and `explain`

Humans and coding agents can inspect **how Semauri interprets source code**
without running the application. `semauri check FILE` validates syntax,
types, imports and compiler rules, reporting `OK` when validation succeeds.
`semauri explain FILE` also performs semantic analysis and prints a numbered,
human-readable trace of decisions made while lowering the program, such as
domain selection, artifact defaults, symbol bindings, reference resolution,
selected branches and planned operations.

For a reproducible walkthrough, open
[`examples/explain_walkthrough.suri`](../examples/explain_walkthrough.suri):

```text
within web:
  create a web called Example.
  add a button called Start.
end.
```

Check the file and request its explanation:

```sh
semauri check examples/explain_walkthrough.suri
semauri explain examples/explain_walkthrough.suri
```

The `check` command prints:

```text
OK
```

The `explain` command prints the following **exact output** for that file
(pinned by `tests/explain_walkthrough.rs`):

```text
1. Entered semantic domain scope 'web'.
2. 'web' resolved to an HTML web document (default web backend).
3. Title explicitly set to 'Example'.
4. Added button 'Start' as button-1.
```

Agents and CI tools can request structured explanations without parsing
the numbered text:

```sh
semauri explain examples/explain_walkthrough.suri --json
```

```json
{
  "explanations": [
    "Entered semantic domain scope 'web'.",
    "'web' resolved to an HTML web document (default web backend).",
    "Title explicitly set to 'Example'.",
    "Added button 'Start' as button-1."
  ],
  "ok": true,
  "status": 0
}
```

The `explanations` array preserves the order of the human-readable trace,
without numeric prefixes. JSON failures use the same source-aware
`diagnostics` envelope as `semauri check FILE --json`. Neither mode
executes application effects.

This shows that the compiler enters the `web` semantic domain, chooses the
HTML backend, retains the explicitly specified document title and assigns
a stable identifier to the new button. The trace describes *compiler
decisions*, **not program stdout** or a complete step-by-step execution
debugger. `check` and `explain` do not render the HTML or run the program.

For additional machine-readable analysis, use `semauri hir FILE` for typed
expressions, `semauri optimize FILE` for optimized HIR,
`semauri effects FILE` to review required permissions and
`semauri plan FILE` or `semauri graph FILE` to inspect runtime work.
Inspect effects and grant only the minimum necessary capabilities
before a separate `run` command.

## Project verification for local development and CI

`semauri verify PROJECT` is a single quality gate for multi-file projects.
It requires a normal `.semauri-root` marker, `main.suri`, and a
`tests/` directory, such as those generated by `semauri init`.

```sh
semauri verify .                  # check formatting, compile all sources and run assertions
semauri verify . --json           # machine-readable CI report
semauri verify . --allow-console  # explicitly allow test prints
```

Verification checks **all** project `.suri` files for canonical formatting
and type/compile errors (including imported files), inspects the entry-point
effect list, and runs the existing assertion-only test runner. The application
itself is **never executed**. Tests cannot require filesystem, network, or
other non-console effects; console output in tests needs `--allow-console`.
The command returns `0` on success, `1` on verification failures, and `64`
on incorrect usage or invalid project layouts. The `--json` output contains
a success flag, file count, effect list, test results, and diagnosed issues.

The verifier skips symlinks and build/cache directories, refuses a project
larger than 256 source files or 4 MiB of total source, and does not modify
any files. Use `semauri fmt PROJECT --check` to list all formatting problems or
`semauri fmt PROJECT --write` to format every source in one operation.
Directory formatting validates all sources before writing the first file,
excludes symlinks and generated/cache directories, and is limited to 256
files and 4 MiB total source. Without `--json`, single-file and directory
formatting retain their existing text output.

Use `semauri fmt FILE --json` to preview canonical source in the `formatted`
field without changing the file. For CI and editors, combine `--json` with
`--check` or `--write` for a file or directory. Reports contain `ok`,
`status`, `mode` (`preview`, `check`, or `write`), `files_checked`,
`files_changed`, and the ordered `changed_files` path list. Check mode
returns status 1 when formatting is needed (and `ok: false`), while write
mode reports the number of files actually rewritten. Invalid source reports
structured diagnostics and leaves files unmodified.

Rerun
verification. This is a local project quality gate, not a dependency/package
manager or a substitute for full integration and deployment tests.

## External and complementary source libraries (v0.72.0)

**In development (unreleased):** use `import "library:NAME" as alias.` or
`import "relative/file.suri" as alias.` and invoke a function with
`invoke alias.function to VALUE`. Aliases isolate imported **function names**,
not top-level variables or side effects. Existing unaliased imports still work.
See [namespaced imports](external-libraries.md#namespaced-imports-development-unreleased).


Projects can now declare third-party Semauri `.suri` libraries in
`.semauri-libraries.json` and import them using a named library path:

```text
import "library:math_kit".
print invoke square to 7.
```

The library's own `.semauri-library.json` provides a matching name,
`MAJOR.MINOR.PATCH` version and relative entry file. The loader accepts
explicitly declared local paths outside the application directory, but
confines all nested imports to their respective canonical library roots.
Imports still participate in compile-time type checking, effect analysis,
source-located diagnostics and normal runtime permission checks.

Use `semauri libraries PROJECT|FILE` to inspect and validate installed
dependencies without executing them. This initial support is fully offline:
it does not download packages, load native plugins or provide registry
resolution or lockfile integrity. The full manifest schema, security
restrictions and working example are in the
[external libraries guide](external-libraries.md).

## Multi-file programs and imports

Semauri projects can share statically typed functions and other definitions
across local `.suri` files. Write imports at the start of your entry file:

```text
# lib/math.suri
define function square taking n as number to (n times n).

# main.suri
import "lib/math.suri".
assert invoke square to 7 is equal to 49.
```

File-based CLI compilation resolves imports recursively with each path
relative to the **importing file**: `semauri check main.suri`,
`semauri run main.suri --allow console_write`, `semauri test main.suri`,
and inspection commands such as `hir`, `effects`, and `plan`.
`fmt` formats only the original file and preserves import strings.

Imports are compile-time source dependencies, not runtime filesystem reads.
Every imported declaration and effect is type-checked and permission-checked
within the final program. Files are imported only once by canonical path.
Cycles, imports after executable statements, absolute paths, paths outside
the active project root (even through symlinks), and files without a
`.suri` extension are rejected with `S416`. Maximums are
**64 files**, **32 import levels**, and **4 MiB** of total UTF-8 source.
Diagnostics report the original file and line when available.

For a project with tests or modules in nested directories, create a regular
empty `.semauri-root` file at the project root. The compiler uses the
**nearest ancestor** with this marker to define its import boundary.
Without a marker, the entry file's own directory is the boundary.
This allows test files to import `../lib/math.suri` within a marked project
without opening unrestricted access to unrelated directories.

Imports currently share one symbol namespace: duplicate definitions are
errors. Only CLI commands that take a file path resolve imports; in-memory
`Compiler::parse` and `Compiler::hir` expect expanded source text.

## Smallest program

Create `hello.suri`:

```text
print "Hello from Semauri".
```

Then check and run it:

```sh
semauri check hello.suri
semauri run --allow console_write hello.suri
```

## Language ergonomics (v0.72.0)

These features are implemented in Rust and have no new runtime effects or
permissions. They are included in Semauri v0.72.0.

### Natural negated ordering

Use `is not greater than`, `is not less than`,
`is no greater than` or `is no less than` for inclusive numeric limits.
Adding `or equal to` after `not greater than` or `not less than`
negates the inclusive comparison instead:

| Expression | Meaning |
| --- | --- |
| `a is not greater than b` / `a is no greater than b` | a ≤ b |
| `a is not less than b` / `a is no less than b` | a ≥ b |
| `a is not greater than or equal to b` | a < b |
| `a is not less than or equal to b` | a > b |

Operands remain statically checked numbers. The compiler uses the existing
exact integer and finite-decimal comparison semantics, and evaluates each
operand once. `no` remains an identifier outside the contextual phrase.

### Inclusive ranges through a bound

`range from A through B` is an inclusive alias for `range from A to B`.
It supports ascending, descending and singleton ranges, using the same
checked integer endpoints and maximum range limits:

```text
let values be range from 1 through 4.
assert values is equal to (a list of 1, 2, 3, 4).
```

The word `through` stays contextual: `let through be 5.` remains valid.

### Optional trailing commas

A list literal can end with a comma immediately before the statement
terminator, and a parenthesized function call can end with one immediately
before its closing parenthesis. Neither adds an empty element or argument.

```text
define function combine taking left as number, right as number to (left plus right).
let xs be a list of 1, 2, 3,.
assert length of xs is equal to 3.
assert invoke combine to (2, 3,) is equal to 5.
```

Extra commas in the middle remain syntax errors, and normal function
argument-count and type checks still apply. See the
[executable example](../examples/natural_language_ergonomics.suri).

## Core syntax

```text
# immutable values and exact arithmetic
let name be "Semauri".
let answer be (6 times 7).
let values be a list of 1, 2, 3, 4.

# typed control flow
if answer is equal to 42:
  print name plus " works".
otherwise:
  print "unexpected".
end.

# pure typed function
define function double taking value as number to (value times 2).
let doubled be invoke double to 21.
assert doubled is equal to 42.
```

Comments begin with `#`, `//`, or a nestable `/* ... */` block. Keywords are case-insensitive, but lowercase is canonical. Strings preserve their case.

### Types

- `number`: checked signed integers and finite decimal values. Fractions may omit the leading zero (for example, `.5`, `.25e+2`); a standalone `.` still ends a statement. Integer literals also accept `0b` (binary), `0o` (octal), `0x` (hexadecimal), and `0z` (base 36) prefixes, in either case, with optional `_` separators between digits; e.g. `0b1010_0101`, `0o755`, `0xFF`, `0z1_z` (71). These literals must fit a signed 64-bit integer.
- `string`: UTF-8 text. Triple-quoted blocks (`"""text"""` or `'''text'''`) can span multiple lines and preserve embedded newlines, quote characters, and backslashes *literally* (no escape processing); the closing three quotes must match. Use double quotes (`"text"`) or single quotes (`'text'`); either delimiter can appear unescaped inside the other. Raw literals (`r"text"` or `r'text'`, also `R` prefix) preserve backslashes literally (no escapes), which is useful for paths and prompt templates. Add matching hash markers (`r#"He said \"hi\""#` or `r##'It's fine'##`) to include the quote delimiter inside the raw value; the closing quote must have the same number of hashes. Literal newlines remain disallowed. Escape the active delimiter with a backslash, and use `\n`, `\t`, `\r`, `\0` (null), `\b` (backspace), `\f` (form feed), `\a` (bell), `\v` (vertical tab), `\e` (ASCII escape), `\\`, `\u{HEX}`, `\uXXXX` (four hexadecimal digits, excluding surrogate code points), or `\UXXXXXXXX` (exactly eight hexadecimal digits, for example `\U0001F600`) for escaped content. You can also use `\xNN` for exactly two hexadecimal digits representing a Unicode character from U+0000 to U+00FF (for example, `\x41` is `A`).
- `boolean`: `true` or `false`.
- `color`: named color values used by declarative programs.
- `list<T>`: immutable homogeneous lists.
- nominal runtime types: typed handles produced by operations.

Semauri does not silently coerce unrelated types.

### Immutable typed dictionaries (development, not yet released)

Dictionaries are immutable **string-keyed maps of one statically inferred value
type**, unlike records whose fields may each have different types.

```text
let scores be dictionary of ("alice": 8, "bob": 10).
assert (entry "alice" of scores) is equal to 8.
print json of scores.
```

Use `dictionary of ("key": EXPR, ...)` or `a dictionary of (...)`.
Access values with `entry "key" of DICTIONARY`; keys are quoted
case-sensitive strings of 1–64 UTF-8 bytes. Unannotated dictionary literals require at least one
entry to infer the value type. **An explicitly typed empty dictionary is
valid:** `dictionary of number ()` or `dictionary of list of string ()`.
Typed nonempty dictionaries may also declare their value type, as in
`dictionary of number ("alice": 8)`. Each entry's value must have exactly
the same type as its annotation (or inferred value type). Duplicate keys, malformed syntax or more than 64
entries produce S420. Mixing incompatible value types, accessing a non-
dictionary or reading a missing key produces S421 (the latter when evaluated).

A dictionary can be inspected safely with **`has entry "key" in DICTIONARY`**,
which returns `true` or `false` even for an absent key; it does not
perform a failing lookup. **`keys of DICTIONARY`** returns a typed
`list of string` in deterministic lexicographic order. This makes it
possible to guard an entry lookup or iterate over keys:

```text
let scores be dictionary of ("alice": 8, "bob": 10).
assert not (has entry "carol" in scores).
print json of keys of scores.
```

A missing entry can also be recovered with an explicit typed default:

```text
let scores be dictionary of ("ada": 7, "grace": 9).
let score be entry "unknown" of scores with default 0.
print score.
```

`entry "key" of DICTIONARY with default EXPR` returns the stored value if
the key exists; otherwise, it evaluates `EXPR`. The fallback expression is
**only evaluated if the key is missing**, and the compiler requires it to
have precisely the dictionary's value type (S423 on mismatch or non-dictionary
input). This is a narrowly scoped *recoverable lookup*, not a general error
catching mechanism. The existing `entry "key" of DICTIONARY` still reports
missing-key errors. The contextual words `with default` do not become
reserved identifiers. Runtime/model-derived dictionaries are still subject
to the existing runtime-value limitations.

Both operations are checked against `dictionary of TYPE` by the compiler.
Using a non-dictionary produces S422. Literal `entry "key" of` still
raises S421 if a key is missing during evaluation; it does not silently
default a value. Existing `keys of json DOCUMENT` retains its meaning.

Dictionary types can be nested, used in lists, and declared as function
parameter types: `taking scores as dictionary of number`. A mismatched
argument type fails with S370. The existing `json of` expression
serializes dictionaries, including nested values, to JSON strings. Enclose
comma-separated list values in parentheses, e.g.,
`dictionary of ("a": (a list of 1, 2), "b": (a list of 3, 4))`.

These dictionaries are **immutable** and use literal string keys; dynamic
key expressions, insert/remove operations, and runtime-derived compound
values are not yet supported. The feature requires no new permissions,
dependencies or online registry. Semauri **0.72.0** remains the current
published version while development continues on an unmerged draft PR.

### Immutable typed records (development, not yet released)

Semauri now supports **structurally typed records** in the development branch.
Records are immutable: field names and the type of each value are known to the
compiler. A field lookup is checked *before execution*.

```text
let person be record of ("name": "Ada", "age": 37, "active": true).
assert (field "age" of person) is equal to 37.
print field "name" of person.
```

Add the optional article (`a record of (...)`), nest records, or include them
in lists. Field names are **quoted, case-sensitive strings** of 1–64 UTF-8
bytes; at most 32 fields are allowed, and duplicate names are rejected.
An empty record is `record of ()`. Access a value with
`field "FIELD_NAME" of RECORD` or use parentheses for nested lookups.
The record's structural type is inferred (for example,
`record<age: number, name: string>`), with S418 for malformed
declarations and S419 for missing fields, non-record access or unit fields.

Use `json of RECORD` to serialize a typed record (including nested
records and lists) into a JSON string suitable for API payloads or files.
Serialization does not implicitly grant network or filesystem permissions.
Parsing arbitrary JSON into a statically typed record is **not** implemented.

Records can contain core values and work with pure functions. **Function
parameters can declare exact structural record types** with
`taking person as record of ("name": string, "age": number)`. Nested
records and `list of TYPE` fields are supported; mismatched structures
report S370 before execution. In this first
step, *building records from runtime-dependent model/API results is not yet
supported*. They do not enable network access, file I/O or implicit effects.
For a field whose value is a comma-separated list, enclose the list
in parentheses so record field separators remain unambiguous:
`record of ("tags": (a list of "alpha", "beta"), "score": 5)`.
Record-valued functions also work across namespaced Semauri libraries.

The public language version is still **0.72.0** and these features remain
in the draft development PR.

### Expressions

Supported expression families include arithmetic, comparisons, Boolean operators, string concatenation, list indexing and concatenation, `length of`, `contains`, `uppercase of`, `lowercase of`, `reverse of`, numeric reductions, extrema, median, sorting, distinct values, bounded list operations, ranges, map/filter/count/reduce, pairwise zip, partition, flatten, repetition, conditional expressions, and pure function calls.

Use parentheses whenever generated code could be ambiguous.

### Natural-language text processing

The following pure, Unicode-aware, statically checked operations are contextual
phrases. Names such as `words` and `lines` still work as identifiers:

```text
let title be trim of "  Useful information  ".
let tokens be words of "one   two\\tthree".
let rows be lines of "a\\r\\nb\\n".
assert title is equal to "Useful information".
assert length of tokens is equal to 3.
assert (tokens at 2) is equal to "three".
assert length of rows is equal to 2.
```

- `trim of STRING` removes leading/trailing Unicode whitespace, returning `string`.
- `words of STRING` splits on Unicode whitespace, omitting empty fields, returning `list<string>`.
- `lines of STRING` splits on LF/CRLF, preserving interior blank lines and omitting a trailing empty field, returning `list<string>`.

Non-string operands are rejected (`S351`), literals can be constant-folded, and
none of these transformations grants or requires runtime capabilities.

### Unicode-aware first-letter capitalization

`capitalize of STRING` uppercases the **first Unicode scalar** while
leaving the remainder of the text byte-for-byte unchanged. Unicode case
mapping can expand a character (for example, `ß` becomes `SS`).
Empty text stays empty, and leading whitespace is not skipped.

```text
assert (capitalize of "hello WORLD") is equal to "Hello WORLD".
assert (capitalize of "élève") is equal to "Élève".
assert (capitalize of "ßeta") is equal to "SSeta".
```

It is pure, works with functions and `map every`, and is constant-folded
for literals. Non-strings use the existing located case-transformation
diagnostic `S351`. The name `capitalize` remains a normal identifier
unless followed by `of`.

### Unicode case swapping

`swapcase of STRING` uppercases lowercase Unicode scalars, lowercases
uppercase scalars, and leaves uncased characters (digits, punctuation,
spaces) unchanged. Unicode case mappings can expand to multiple scalars.

```text
assert (swapcase of "HeLLo 123!?") is equal to "hEllO 123!?".
assert (swapcase of "ßéta") is equal to "SSÉTA".
```

The expression is pure and constant-foldable, works with typed functions
and list mapping, uses `S351` for non-string operands, and does not
reserve the word `swapcase` when it is an ordinary identifier.

### Unicode-aware first-letter lowercasing

`uncapitalize of STRING` lowercases the first Unicode scalar, preserving
all remaining text, including case and whitespace. Empty text stays empty,
and Unicode lowercase mappings may expand a scalar.

```text
assert (uncapitalize of "HELLO World") is equal to "hELLO World".
assert (uncapitalize of "ÉTÉ") is equal to "éTÉ".
```

The expression is pure and constant-foldable, works inside functions and
list mappings, and rejects non-string operands with `S351`. The name remains
an ordinary identifier except when followed by `of`.

### Unicode-aware title casing of words

`titlecase of STRING` uppercases the first Unicode scalar and lowercases
the remaining scalars in **each whitespace-delimited word**, preserving
original Unicode whitespace and punctuation. This deliberately does not
split words at hyphens or punctuation; Unicode mappings may expand scalars.

```text
assert (titlecase of "hELLO wORLD") is equal to "Hello World".
assert (titlecase of "ßETA ÉTÉ") is equal to "SSeta Été".
assert (titlecase of "  TWO  WORDS ") is equal to "  Two  Words ".
```

This pure, typed, constant-foldable expression accepts only strings (`S351`),
works in functions and list mapping, and leaves `titlecase` usable as an
ordinary identifier outside `titlecase of`.

### Typed JSON serialization

The pure `json of VALUE` operation produces valid JSON text from a number,
boolean, string, color, or a nested homogeneous list of supported values.
It is safe for generating JSON payloads and correctly escapes strings; it does
not decode JSON, serialize runtime handles, or perform network requests.

```text
let labels be a list of "alpha", "beta".
let payload be json of labels.
assert payload is equal to "[\"alpha\",\"beta\"]".
let enabled be json of true.
assert enabled is equal to "true".
```

`json` remains a valid variable name unless immediately followed by `of`.
Non-serializable types produce the static diagnostic `S390`.

### Text splitting and joining

Use `split of TEXT by DELIMITER` to obtain a typed `list<string>` and
`join of LIST with DELIMITER` to reconstruct text. Delimiters are treated
literally (not as regular expressions). Splitting keeps empty fields, including
at the beginning and end of input. An empty delimiter, or more than 10,000
parts, produces a source-aware `S393` diagnostic. Incorrect operand types
are reported as `S392`. Both operations are pure.

```text
let fields be split of "name,role,team" by ",".
assert length of fields is equal to 3.
let row be join of fields with " / ".
assert row is equal to "name / role / team".
```

`join` remains a legal identifier outside the operator phrase. The word\n`split` also names an existing ML-domain action, which continues to work.

### Numeric sign

Use `sign of NUMBER` to classify a numeric value as `-1`, `0` or `1`
without rounding it or converting integers to floating point. It is pure,
works with signed 64-bit integers (including the minimum integer) and finite
decimals, and composes with functions and list transformations:

```text
let measurements be a list of minus 3, 0, 2.5.
let directions be map every n in measurements to (sign of n).
assert (directions at 0) is equal to (minus 1).
assert (directions at 1) is equal to 0.
```

The keyword `sign` is contextual: `let sign be 5.` still defines a normal
variable. Non-numeric operands cause the located type diagnostic `S417`;
non-finite numeric values cause `S418` at evaluation. Signed zero returns 0.

### Numeric floor and ceiling

Use `floor of NUMBER` to round toward negative infinity and
`ceiling of NUMBER` to round toward positive infinity:

```text
assert (floor of 2.8) is equal to 2.
assert (ceiling of 2.2) is equal to 3.
assert (floor of (minus 2.2)) is equal to (minus 3).
assert (ceiling of (minus 2.8)) is equal to (minus 2).
```

Both expressions are pure and typed as `number`. Exact integer inputs
are returned unchanged, including the full signed 64-bit range. Finite
floating-point inputs are rounded using IEEE-754 floor/ceiling and produce
exact integers when safely within signed 64-bit bounds; results outside that
range remain floating point, without saturating casts. The names `floor`
and `ceiling` remain valid identifiers unless followed by `of`.
Non-number operands produce `S419`; non-finite inputs cause `S420`.
Neither operator requires additional runtime permissions.

### Numeric squares

`square of NUMBER` computes the square without routing integers through
floating point. Integers use checked signed 64-bit multiplication, including
exact results larger than 2^53. Finite decimals use floating-point
multiplication and reject non-finite results.

```text
assert (square of (minus 4)) is equal to 16.
let values be a list of 2, 3, 4.
let squares be map every n in values to (square of n).
assert (squares at 2) is equal to 16.
```

`square` remains a normal variable name unless followed by `of`.
Non-number operands raise `S421` at type checking; numeric overflow or
non-finite results raise `S422` during evaluation. The expression is pure
and requires no new runtime permissions.

### Numeric differences and error metrics

Three contextual forms calculate signed differences and common error metrics
without introducing conversions or a new runtime capability:

```text
assert (difference between 10 and 4) is equal to 6.
assert (difference between 4 and 10) is equal to (minus 6).
assert (absolute difference between 4 and 10) is equal to 6.
assert (squared difference between 4 and 10) is equal to 36.
let errors be map every predicted in (a list of 1, 2, 3) to
  (squared difference between predicted and 2).
assert (errors at 0) is equal to 1.
```

Operands must be numbers. A compound operand must be parenthesized:
`difference between (a plus 1) and (b times 2)`. These forms compile
to the existing checked subtraction, absolute-value and square operators,
so each operand is evaluated once. Exact integer arithmetic is preserved,
including values above 2^53, and non-finite decimal or signed 64-bit
overflow is rejected. Invalid operand types raise `S314`, subtraction
overflow/non-finite results raise `S470`, absolute overflow raises
`S388`, and square overflow raises `S422`.
Each word remains an ordinary identifier outside the contextual phrase.

See [numeric_differences.suri](../examples/numeric_differences.suri).

### Running aggregates and changes between readings

Numeric sequences can be analyzed without manually writing a loop:

```text
let readings be a list of 3, 5, 2, 6.

let totals be running total of readings.
let averages be running average of readings.
let changes be adjacent differences of readings.

assert (totals at 3) is equal to 16.
assert (averages at 3) is equal to 4.
assert (changes at 0) is equal to 2.
assert (changes at 1) is equal to (minus 3).
assert (changes at 2) is equal to 4.
```

- `running total of LIST`, `running sum of LIST`, `cumulative total of LIST`,
  and `cumulative sum of LIST` return the cumulative sum after each element.
- `running average of LIST` and `cumulative average of LIST` return
  the arithmetic average of every prefix.
- `adjacent differences of LIST` and `successive differences of LIST`
  return each element minus the preceding element. The result is one
  element shorter than the input.

The total, average and adjacent-difference operations require `list<number>` (`S353` for other types),
produce `list<number>`, and evaluate the input once in linear time.
An empty list produces an empty result. A single-element list yields no
adjacent differences. Cumulative integer totals use checked signed
64-bit addition, preserving integers beyond 2^53; integer or floating
overflow raises `S471`. Adjacent differences reuse checked subtraction
(`S470`). Running averages divide checked prefix totals using the
language's normal floating-point average semantics. No additional
permissions or dependencies are required. Words such as `running`,
`cumulative`, `adjacent`, and `differences` are not reserved.

See the runnable [running_statistics.suri](../examples/running_statistics.suri).

### Running extrema and products

Semauri also supports three ordered prefix scans that preserve the
intermediate minimum, maximum or product after each reading:

```text
let readings be a list of 3, 5, 2, 6.
assert (running minimum of readings) is equal to (a list of 3, 3, 2, 2).
assert (cumulative maximum of readings) is equal to (a list of 3, 5, 5, 6).
assert (running product of readings) is equal to (a list of 3, 15, 30, 180).
```

- `running minimum of LIST` and `cumulative minimum of LIST`
  keep the smallest numeric value seen so far.
- `running maximum of LIST` and `cumulative maximum of LIST`
  keep the largest numeric value seen so far.
- `running product of LIST` and `cumulative product of LIST`
  multiply all numeric values from the start to each position.

Shorter `running min/max of LIST` and `cumulative min/max of LIST`
are also supported. All expressions operate on `list<number>` in
linear time, evaluate the list once, preserve its length and order, and
return an empty list for empty input. Integer min/max comparisons are
exact even beyond 2^53 and signed prefix products use checked i64
multiplication. When a decimal factor appears, later products use
finite floating-point multiplication. Non-list/nonnumeric inputs
receive `S353`; non-finite extrema receive `S355`, and overflowing
products receive `S387`. These operations are pure and require no
new permissions or dependencies.

See [running_statistics.suri](../examples/running_statistics.suri).

### Prediction accuracy: MSE, RMSE and mean bias

Given a list of **signed residuals** (observed minus predicted values),
Semauri supports the common model-evaluation spellings:

```text
let residuals be a list of 2, (minus 3), 1.

assert (mean squared error of residuals) is equal to (14 divided by 3).
assert (mse of residuals) is equal to (mean square of residuals).
assert (root mean squared error of residuals) is equal to (sqrt of (14 divided by 3)).
assert (rmse of residuals) is equal to (rms of residuals).
assert (mean bias error of residuals) is equal to 0.
assert (mean error of residuals) is equal to 0.
assert (mbe of residuals) is equal to 0.
```

- `mean squared error of ERRORS` / `mse of ERRORS`: the average of
  squared residuals, identical to `mean square of ERRORS`.
- `root mean squared error of ERRORS` / `rmse of ERRORS`: the
  nonnegative square root of MSE, identical to `rms of ERRORS`.
- `mean bias error of ERRORS`, `mean error of ERRORS` and
  `mbe of ERRORS`: the **signed** mean of residuals, identical to
  `average of ERRORS`. Negative and positive errors may cancel.

The inputs are residual lists, **not** two raw prediction and
observation lists. Pair observations using `zip every` and
`difference between` first (see the runnable
[model_error_metrics.suri](../examples/model_error_metrics.suri)).

These expressions share Semauri's checked numeric operations:
MSE/RMSE reject non-list input (`S365`), nonnumeric entries (`S421`),
square overflow (`S422`), and aggregation overflow/empty lists
(`S354`). Signed mean error uses the ordinary average checks
(`S353` for a nonnumeric input and `S354` for empty/overflow).
The words `mse`, `rmse`, `mbe`, `bias`, and `error` remain valid
identifiers when not used as these complete expressions.

### Magnitude aggregations and mean absolute error

For a list of signed residuals, four pure, checked expressions compute
absolute-error statistics without manual mapping:

```text
let actual be a list of 3, 6, 4.
let predicted be a list of 1, 9, 5.
let errors be zip every observed in actual with estimated in predicted
  to (difference between observed and estimated).
assert (sum of magnitudes of errors) is equal to 6.
assert (mean magnitude of errors) is equal to 2.
assert (mean absolute error of errors) is equal to 2.
assert (mae of errors) is equal to 2.
assert (max magnitude of errors) is equal to 3.
```

`sum of magnitudes of VALUES` sums the absolute values in a typed numeric
list, returning zero for an empty list. `mean magnitude of VALUES`,
`mean absolute error of VALUES`, and `mae of VALUES` are equivalent
arithmetic means of those magnitudes; `max magnitude of VALUES` finds
the greatest magnitude. **MAE consumes residual errors**, not two separate
observation/prediction lists. Use `zip every` to form residuals first.

All operators evaluate the source list once and apply the same checked
`absolute of` implementation to each element. Non-lists raise `S365`,
nonnumeric elements raise `S386`, an absolute-value overflow raises
`S388`, and a numeric aggregation overflow raises `S354`. An empty list
is invalid for mean or MAE (`S354`) and maximum magnitude (`S355`).
The names `mae`, `mean`, `magnitude`, and `error` remain valid
identifiers outside these complete phrases.

See [absolute_error_statistics.suri](../examples/absolute_error_statistics.suri).

### Squared statistics and root mean square

Semauri can compute three common squared-value aggregates directly over
`list<number>`, using checked integer squaring and the typed list operators:

```text
let values be a list of 3, 4.
assert (sum of squares of values) is equal to 25.
assert (mean square of values) is equal to 12.5.
assert (root mean square of values) is equal to (sqrt of 12.5).
assert (rms of values) is equal to (root mean square of values).
```

- `sum of squares of VALUES` computes the exact checked sum of all
  squared values (the empty-list result is 0).
- `mean square of VALUES` computes the arithmetic mean of squares.
- `root mean square of VALUES` and its short alias `rms of VALUES`
  compute the nonnegative square root of the mean square.

Each expression evaluates its source list once and squares each element once
using the same checked operations as `map every n in VALUES to (square of n)`.
An empty input is invalid for mean square and RMS (`S354`).
Non-list inputs raise `S365`; nonnumeric elements raise `S421`;
integer square overflow raises `S422`; and aggregate overflow or
non-finite results raise `S354`. The words `root`, `square`,
`squares` and `rms` remain valid variable names outside these phrases.
These pure operations need no new permissions.

See [squared_statistics.suri](../examples/squared_statistics.suri).

### Population variance and standard deviation

`variance of NUMERIC_LIST` and `stdev of NUMERIC_LIST` calculate the
**population** variance (divide by N) and standard deviation (its square root).
They use a numerically stable one-pass Welford update and return finite
decimal values. An empty list is invalid (`S468`); non-finite intermediate
values return `S469`; other types are rejected with `S353`.

```text
let values be a list of 0, 2, 4.
let spread be variance of values.
let deviation be stdev of values.
assert spread is greater than 2.
```

See [statistics.suri](../examples/statistics.suri).

### Checked exponentiation

Use `power of BASE to EXPONENT` for exponentiation. Both arguments must
be numbers. Nonnegative integer powers use **checked 64-bit arithmetic**
(`S467` on overflow) and retain integer results. Negative or fractional
exponents use finite real arithmetic; invalid real domains such as negative
bases with fractional exponents, or zero with a negative exponent, are
rejected (`S467`). `0` to the `0` power yields `1`.

```text
let area be power of 3 to 2.
let inverse be power of 2 to (0 minus 2).
assert area is equal to 9.
assert inverse is equal to 0.25.
```

See [power.suri](../examples/power.suri).

### Scientific math: exponential, logarithms and trigonometry

`exp of NUMBER` (e to the power of x), `ln of NUMBER` (natural log),
`log2 of NUMBER` (base-2 log), `log10 of NUMBER` (base-10 log),
`sin of NUMBER`, `cos of NUMBER`, `asin of NUMBER`, `acos of NUMBER`
and `atan of NUMBER` are pure numerical operators.
` sinh of NUMBER`, `cosh of NUMBER`, and `tanh of NUMBER` add hyperbolic
sine, cosine and tangent with checked finite results.
Angles use **radians**, including results of inverse trigonometry.
All results are finite decimals; logarithms require positive inputs,
`asin` and `acos` require input in [-1, 1], and exponential overflow
is rejected with diagnostic `S466`.
Non-numeric arguments are rejected with `S465`. Operators compose with
typed functions and list transforms.

```text
let growth be exp of 1.
let angle be sin of 0.
let log_one be ln of 1.
let base_two be log2 of 8.
let base_ten be log10 of 100.
let inverse be asin of 1.
assert angle is equal to 0.0.
assert log_one is equal to 0.0.
assert base_two is equal to 3.0.
assert base_ten is equal to 2.0.
assert inverse is greater than 1.5.
```

See [scientific_math.suri](../examples/scientific_math.suri).

### Typed square roots

`sqrt of NUMBER` computes a non-negative square root, returning `number`.
For integer inputs, perfect squares retain an exact integer result—even
when the square is larger than the exact integer range of f64. Non-perfect
roots return finite decimals; decimal perfect squares yield integer results.

```text
assert (sqrt of 81) is equal to 9.
assert (sqrt of (square of 12)) is equal to 12.
let estimate be sqrt of 2.
assert estimate is greater than 1.
assert estimate is less than 2.
```

The contextual word `sqrt` remains a valid variable identifier unless
followed by `of`. Wrong types produce `S423` at compile time. Negative
or non-finite operands produce a located `S424` evaluation diagnostic.
Square roots are pure, checked and can be constant-folded.

### Typed even/odd predicates

`even of NUMBER` and `odd of NUMBER` return `boolean`. Both work on
exact signed 64-bit integers and integral finite decimals within that
range, retaining correct parity even beyond 2^53 for integer values.

```text
assert even of 8.
assert odd of (minus 3).
let flags be map every n in (a list of 1, 2, 3) to (odd of n).
assert (flags at 0).
```

`even` and `odd` remain contextual identifiers; only the `of` form is
an operator. A non-number produces the located `S425` type diagnostic.
Fractional or non-finite decimals and out-of-range numeric values produce
`S426` rather than silently truncating. Both predicates are pure and
constant-foldable, requiring no runtime permissions.

### Round to nearest integer

`round of NUMBER` rounds to the nearest whole number. Halfway cases are
rounded **away from zero**, so `round of 2.5` is 3 and
`round of (minus 2.5)` is -3. Integers remain exact, even above 2^53.
Finite floating-point values become integers when the rounded value fits in
signed 64-bit range; otherwise they remain floats without saturating casts.

```text
assert (round of 2.49) is equal to 2.
assert (round of 2.5) is equal to 3.
assert (round of (minus 2.5)) is equal to (minus 3).
```

The name `round` remains a valid identifier unless followed by `of`.
Invalid operand types raise `S427` and non-finite values raise `S428`
at evaluation. Rounding is pure and constant-foldable.

### Truncate toward zero

`truncate of NUMBER` removes the fractional part without rounding.
Unlike `floor of`, negative values move **toward zero**:
`truncate of (minus 2.9)` yields -2, whereas `floor of (minus 2.9)`
yields -3.

```text
assert (truncate of 2.9) is equal to 2.
assert (truncate of (minus 2.9)) is equal to (minus 2).
```

Exact integers stay unchanged, including signed 64-bit boundary values.
Finite decimals are converted to integers only when safely representable;
out-of-range results remain floats. The contextual word `truncate`
remains a valid identifier unless followed by `of`.
A wrong operand type raises `S429`; a non-finite operand raises `S430`.
This is a pure, constant-foldable operation.

### Checked factorial

`factorial of NUMBER` computes the exact integer factorial for a
non-negative integer from **0 through 20**, returning `number`. Both integer
values and finite integral decimals in that range are supported; 20! fits
signed 64-bit integer precision but 21! does not.

```text
assert (factorial of 0) is equal to 1.
assert (factorial of 5) is equal to 120.
assert (factorial of 20) is equal to 2432902008176640000.
```

Negative, fractional, non-finite and out-of-range values raise the located
`S432` diagnostic, rather than overflowing or truncating. Non-number
operands fail static checking with `S431`. The word `factorial` is
contextual and can still be used as a variable name. Factorials are pure
and constant-foldable.

### Checked numeric cubes

`cube of NUMBER` computes a pure cube with exact signed 64-bit integer
arithmetic and finite decimal multiplication.

```text
assert (cube of 3) is equal to 27.
assert (cube of (minus 2)) is equal to (minus 8).
```

Non-number inputs raise `S433` at type checking; integer overflow or
non-finite results raise located `S434`. The contextual identifier
`cube` remains a valid variable name unless followed by `of`.

### Exact 64-bit prime predicate

`prime of NUMBER` returns a `boolean` for signed 64-bit integral values
using deterministic Miller–Rabin tests (not lossy floating-point division).
Zero, one and negative integers return `false`.

```text
assert prime of 97.
assert not (prime of 561).
assert not (prime of 1).
```

Finite integral decimals within signed 64-bit range are accepted. Non-number
operands raise `S435`; fractions, non-finite and out-of-range decimals
produce located `S436`. The identifier `prime` stays usable as a
variable unless followed by `of`. The predicate is pure.

### Exact cube roots

`cbrt of NUMBER` computes a real cube root. Perfect **integer** cubes
produce exact signed 64-bit integer roots, including the minimum
signed 64-bit integer; non-perfect integers and finite decimals yield
floating-point results. Negative inputs are supported.

```text
assert (cbrt of 125) is equal to 5.
assert (cbrt of (minus 27)) is equal to (minus 3).
let estimate be cbrt of 2.
assert estimate is greater than 1.
assert estimate is less than 2.
```

Wrong operand types receive `S437`; non-finite numeric inputs receive
`S438`. `cbrt` stays an ordinary variable name unless followed by `of`.
The operation is pure and constant-foldable.

### Numeric reciprocals

`reciprocal of NUMBER` calculates `1 / NUMBER` as a floating-point
number. It accepts signed integers and finite decimals, rejects positive and
negative zero, and rejects non-finite inputs/results instead of silently
returning infinity. The expression is pure and constant-foldable.

```text
assert (reciprocal of 4) is equal to 0.25.
assert (reciprocal of (minus 2)) is equal to (minus 0.5).
let samples be a list of 2, 4, 8.
let inverses be map every n in samples to (reciprocal of n).
```

Non-number operands produce the source-located type error `S439`; invalid
numeric values produce `S440`. `reciprocal` remains a normal identifier
when not immediately followed by `of`, and requires no new capabilities.

### Exact decimal digit counts

`digits of NUMBER` returns the count of base-10 digits in an integer's
absolute value: zero has one digit; the minus sign does not count.
Unlike converting to floating point, it retains exact results for the full
signed 64-bit range, including the most negative integer.

```text
assert (digits of 0) is equal to 1.
assert (digits of (minus 100)) is equal to 3.
assert (digits of 9007199254740993) is equal to 16.
assert (digits of 100.0) is equal to 3.
```

Finite, integral decimals are allowed only within the signed 64-bit
range. Non-numeric operands receive `S441`; fractions and out-of-range
numeric values receive `S442` with a source location. The expression is
pure, constant-foldable, permission-free, and `digits` remains an ordinary
identifier unless followed by `of`.

### Safe typed conversions from text

`parse number from TEXT` and `parse boolean from TEXT` return checked
`number` and `boolean` values. Surrounding whitespace is ignored. Boolean
values must be `true` or `false`, case-insensitively. Numbers accept signed
64-bit integers, finite decimals and exponent notation. Integer overflow
is rejected rather than rounded to a floating-point value.

```text
let amount be parse number from " 125.5 ".
let enabled be parse boolean from "TRUE".
assert amount is greater than 125.
assert enabled.
```

Non-string inputs are rejected during type checking (`S394`); invalid text
produces a source-aware `S395` error during evaluation. Parsing is pure,
does not silently coerce values, and `parse` remains an ordinary identifier.

### Typed JSON decoding

The `decode json as TYPE from TEXT` family reads a JSON document into a
statically checked Semauri value. Supported types are `string`, `number`,
`boolean`, `strings`, `numbers`, and `booleans` (the plural forms are
homogeneous lists). JSON strings are unescaped by the decoder; JSON numbers,
booleans, and arrays must match the requested type exactly.

```text
let names be decode json as strings from "[\\"Ada\\",\\"Grace\\"]".
assert length of names is equal to 2.
let threshold be decode json as number from "12.5".
assert threshold is greater than 12.
let flag be decode json as boolean from "true".
assert flag.
```

Decoding is pure and uses a 1 MiB JSON text limit and a 10,000-item list limit.
Malformed documents, unexpected JSON types, and unsupported shapes such as
objects or nested arrays produce a located `S397` diagnostic. A non-string
source produces a static `S396` diagnostic. JSON numeric values are finite,
and integers larger than signed 64-bit range cannot be silently rounded. This
is typed scalar and homogeneous-list decoding, not a general records/maps API.

### JSON object inspection

Use `keys of json DOCUMENT` to get a sorted `list<string>` of an object's
field names, and `has json key KEY in DOCUMENT` to check a field's presence.
Both operations parse the JSON document without external effects. Presence
checks distinguish missing fields from fields containing JSON `null`.

```text
let document be "{\\"name\\":\\"Ada\\",\\"active\\":true}".
let names be keys of json document.
assert length of names is equal to 2.
assert (names at 0) is equal to "active".
assert has json key "name" in document.
assert not (has json key "missing" in document).
```

Each operation requires string operands (`S398`). A malformed JSON value,
non-object document, document over 1 MiB, or object over 10,000 unique keys
produces the located diagnostic `S399`. Field names are matched exactly,
and keys are returned in deterministic Unicode code-point order. These
operations inspect object structure; they do not introduce mutable maps or
untyped dynamic values.

### Typed JSON field extraction

Use `json TYPE at KEY in DOCUMENT` to read a field from a JSON object without
introducing dynamic maps or implicit coercion. The six supported field types
are `string`, `number`, `boolean`, `strings`, `numbers`, and `booleans`.
Plural forms produce homogeneous lists with stable element types.

```text
let document be "{\\"name\\":\\"Ada\\",\\"score\\":42,\\"tags\\":[\\"compiler\\",\\"typed\\"]}".
let name_value be json string at "name" in document.
let score be json number at "score" in document.
let tags be json strings at "tags" in document.
assert name_value is equal to "Ada".
assert score is greater than 40.
assert (tags at 1) is equal to "typed".
```

The key and document must both be strings (`S400`). Missing fields, explicit
JSON `null`, mismatched types, oversized documents, overflowing integer
literals, and collections over the 10,000-item limit produce the located
`S401` diagnostic. Field names are exact and case sensitive. The operation
is deterministic and pure; it requires no new permissions.

### Unicode characters and code points

Pure conversions work with Unicode **scalar values**, not UTF-8 bytes:

- `characters of TEXT` gives a `list<string>` of one-scalar strings.
- `codepoints of TEXT` gives a `list<number>` of Unicode scalar numbers.
- `codepoint of TEXT` reads the scalar value of a string containing exactly
  one character.
- `character from NUMBER` constructs a single-character string from a valid
  signed integer code point.

```text
let glyphs be characters of "Aé🙂".
let numbers be codepoints of "Aé🙂".
assert length of glyphs is equal to 3.
assert (numbers at 1) is equal to 233.
let emoji be character from 128578.
assert codepoint of emoji is equal to 128578.
```

The compiler reports `S408` for incorrect operand types. Invalid Unicode
values (including surrogates, non-integer numbers, or numbers above U+10FFFF),
empty/multi-character input to `codepoint of`, and lists over 10,000
characters report `S409` at evaluation. No extra capabilities or dependencies
are needed. Multi-codepoint grapheme clusters may produce multiple list entries.

### Direct Unicode character indexing

`character at INDEX in TEXT` extracts a single Unicode scalar from a string.
The index is zero based, and indexing counts Unicode scalars rather than UTF-8
bytes. The result is a new `string`.

```text
let text be "Aé🙂".
assert (character at 0 in text) is equal to "A".
assert (character at 1 in text) is equal to "é".
assert (character at 2 in text) is equal to "🙂".
```

The compiler requires a `number` and a `string` (`S410`).
Fractional, negative or out-of-range indices produce a located `S411`
diagnostic during evaluation. No implicit rounding is allowed, and no
new effects are required. Valid literal lookups are constant-folded.

### Unicode string slicing with take and drop

The existing list operations `take COUNT from COLLECTION` and
`drop COUNT from COLLECTION` now accept strings. They return a new
string containing the first `COUNT` Unicode scalar values, or the
characters after the first `COUNT`, respectively.

```text
let name be "café".
assert take 3 from name is equal to "caf".
assert drop 3 from name is equal to "é".
assert take 100 from name is equal to name.
```

Counts must be nonnegative integers; invalid values produce `S364`.
Out-of-range counts clamp to the string length. Existing typed-list
behavior is unchanged. These pure operators count Unicode scalar values,
not bytes or extended grapheme clusters.

### Nested JSON pointer extraction

The pure expression `json at pointer POINTER in DOCUMENT` selects a
nested JSON value using an RFC 6901 pointer and returns a `string`
containing the **serialized JSON value**. Pair it with existing
`decode json as TYPE from` for a statically checked scalar or list.

```text
let document be "{\\"users\\":[{\\"name\\":\\"Ada\\"}]}".
let selected be json at pointer "/users/0/name" in document.
let name_value be decode json as string from selected.
assert name_value is equal to "Ada".
```

Paths start with `/`; an empty pointer selects the whole document.
Segments use `~1` to escape `/` and `~0` to escape `~`.
Array indexes are zero-based with no leading zeroes; missing or invalid
paths produce `S407`. Both operands must be strings (`S406`).
The JSON input is capped at 1 MiB and path depth at 64 segments.
This expression does not grant network or filesystem permissions, and
does not change the existing exact top-level `json TYPE at KEY` syntax.

### Unicode text searching

Use `occurrences of NEEDLE in TEXT` to count **non-overlapping** matches,
or `position of NEEDLE in TEXT` to find the **first match**. Positions are
zero-based Unicode scalar-value indices, not UTF-8 byte offsets. A missing
match returns `-1`, and an empty position needle returns `0`.

```text
let text be "café café".
assert occurrences of "café" in text is equal to 2.
assert position of "é" in text is equal to 3.
assert position of "missing" in text is equal to minus 1.
```

Both operations return `number` and statically require two strings (`S402`).
Counting occurrences of an empty needle is rejected with a located `S403`
diagnostic. They are deterministic, case sensitive, require no new effects,
and support compile-time constant folding. Search results count Unicode
scalar values, not extended grapheme clusters or locale-sensitive matches.

### Unicode substring positions

`positions of NEEDLE in TEXT` returns a `list<number>` with every
**non-overlapping** match position, counted in zero-based Unicode scalar
values. No matches returns a typed empty list. `last position of NEEDLE in
TEXT` returns the **rightmost** match position as a `number`, or
`minus 1` when absent. For an empty needle, `last position` returns
the Unicode scalar length of the text.

```text
let positions_found be positions of "é" in "café café".
assert length of positions_found is equal to 2.
assert (positions_found at 0) is equal to 3.
assert (positions_found at 1) is equal to 8.
assert last position of "é" in "café café" is equal to 8.
```

Both operators require string operands and reject type mismatches at compile
time (`S414`). `positions of` requires a nonempty needle and permits at
most 10,000 matches (`S415`). Matching is Unicode-aware, case-sensitive,
pure and literal (not a regular expression). Constant inputs are folded to
typed HIR expressions; invalid expressions retain diagnostics. The rightmost
match may overlap an earlier match, as in Rust's `rfind`.

### Literal substring removal

`remove all of NEEDLE from TEXT` removes every **non-overlapping literal**
match of `NEEDLE` from `TEXT`, returning a new string. Search is
case-sensitive, Unicode-aware, and does not interpret the needle as a regular
expression. If the needle is missing, the input text is unchanged.

```text
let cleaned be remove all of "é" from "café café".
assert cleaned is equal to "caf caf".
let compact be remove all of "aa" from "aaaaa".
assert compact is equal to "a".
```

Both operands must be strings (`S412`). Empty needles produce a located
`S413` error. Valid literal expressions can be constant-folded, the
operation has no external effects, and the existing `remove` domain action
remains reserved for its established operation syntax.

### Natural-language string predicates

Use `TEXT starts with PREFIX` and `TEXT ends with SUFFIX` to validate names,
file extensions, routing paths, and other structured text. Both expressions
return `boolean`, compare Unicode strings without locale-dependent coercion,
and accept empty prefixes or suffixes. Type mismatches report `S391`.

```text
let filename be "report.json".
let is_json be filename ends with ".json".
let is_report be filename starts with "report".
assert is_json and is_report.
```

These are pure operations; no new effect grants are required. Their words remain
ordinary identifiers when they are not used as the complete operator phrase.

### Control flow and scope

```text
let score be 91.

if score is greater than or equal to 90:
  print "excellent".
otherwise if score is greater than or equal to 70:
  print "pass".
otherwise:
  print "retry".
end.

unless false:
  print "runs".
end.
```

Branches and iteration bodies have lexical scope. Functions are pure, must be declared before use, accept 1–16 typed parameters, and have inferred return types.

## Effects and safe execution

Compilation is deterministic and does not grant capabilities. Inspect requirements explicitly:

```sh
semauri effects program.suri
semauri plan program.suri
semauri graph program.suri
```

Execution requires named grants:

```sh
semauri run --allow console_write program.suri
semauri run --allow filesystem_read --allow filesystem_write program.suri
```

Use `--dry-run` when you need the output plan without performing effects.

## `.config.suri`

Project configuration uses typed JSON parameters and named environment bindings:

```text
parameter model = "local/default".
parameter max_tokens = 1024.
parameter temperature = 0.2.
environment model_api_key = "SEMAURI_MODEL_API_KEY".
```

Validate it without exposing environment values:

```sh
semauri config
semauri config .config.suri --set temperature=0.0
semauri config .config.suri --check-env
```

`--set` may override only declared parameters and accepts a JSON value. Environment bindings are resolved lazily from the process environment; summaries report only the variable name and whether it is set, never its value. Do not commit credentials.

Programs read declared bindings at runtime with an explicit `config_read` effect:

```text
let model_identifier be string config "model".
let max_turns be number config "max_turns".
let safe_mode be boolean config "safe_mode".
```

`semauri run` discovers the nearest `.config.suri`, starting beside the source file and walking toward the filesystem root. Use `--config PATH` to select one explicitly or `--set NAME=JSON_VALUE` to override a declared parameter. Each binding is resolved only when the program reads it; unrelated environment secrets are never loaded.

## Assertions and tests

```text
let result be (6 times 7).
assert result is equal to 42.
```

```sh
semauri test examples/test_suite
semauri test path/to/tests --allow-console
semauri test path/to/tests --json
semauri test path/to/tests --filter module/math
```

The test runner recursively discovers `.suri` files, does not follow symbolic links, requires an executable assertion, and rejects non-console effects. Test discovery excludes build caches and configuration files and is limited to 256 sources and 32 directory levels.

Use `--json` for editor and CI integrations: the response contains `ok`, `status`, `passed`, `failed`, and an ordered `tests` array. Each test reports `path`, `ok`, and `error` for failures. Discovery/usage errors return exit status 64 with an `error` field and empty `tests`; failing suites return status 1. JSON output does not relax runtime capability checks. For a focused run, `--filter TEXT` selects test files whose relative path contains that exact, case-sensitive substring (forward slashes on every platform). No matches is a usage error with exit status 64; filtering never authorizes additional effects.

## LLM and VLM primitives

Semauri provides typed, provider-neutral primitives for language models, vision models, embeddings, tools, and bounded agents. They compile into an inspectable runtime plan. The built-in deterministic adapter executes `local/*` identifiers without network access; hosted identifiers require a provider adapter.

```text
let model_identifier be string config "language_model".
let request_budget be number config "request_budget".
cap ai requests to request_budget.
reattempt ai requests 2 attempts.
memoize ai response cache.
let model be connect language model model_identifier.
let assistant be compose agent using model system "Answer briefly." max 4 turns.
let answer be ask assistant with "Summarize the supplied context.".
print answer.
```

```text
let model_identifier be string config "vision_model".
let model be activate vision model model_identifier.
let description be inspect image "sample.png" using model with "Describe visible facts.".
print description.
```

Available primitives:

- `connect language model IDENTIFIER`
- `activate vision model IDENTIFIER`
- `cap ai requests to NUMBER`
- `reattempt ai requests NUMBER attempts`
- `memoize ai response cache`
- `prompt MODEL with TEXT`
- `inspect image PATH using MODEL with TEXT`
- `embed TEXT using MODEL`
- `declare tool NAME schema JSON_SCHEMA`
- `compose agent using MODEL system TEXT max NUMBER turns`
- `ask AGENT with TEXT`

Language-model and vision-model handles are distinct nominal types. Configuration reads, network, model loading, inference, image reads, console output, and tool execution remain explicit effects. Compilation never contacts a provider or reads credentials.

Request limits reject additional uncached inference once exhausted. The response cache is in-memory and scoped to one program run. Retry policy is stored by the runtime for provider adapters; the deterministic local adapter does not manufacture transient failures.

## Capability roadmap

Semauri is useful today for deterministic programs and inspectable local AI workflows. The larger runtime is intentionally being built in layers. Important capabilities not yet claimed as complete are:

- hosted-provider adapters with streaming, timeouts, rate limits, retry backoff, and fallback chains;
- structured records/maps, JSON decoding, typed extraction, classification, and schema validation;
- multi-turn conversation values, tool dispatch with validated arguments, execution traces, and cost accounting;
- HTTP, general file I/O, command-line input, persistent key-value storage, vector search, and text chunking;
- recoverable errors, modules/imports, packages, macros, pattern matching, and richer numeric types;
- formatter/editor services, debugger integration, a REPL, standalone builds, and WebAssembly tooling.

New runtime features should preserve the existing rules: typed inputs, immutable values, explicit capabilities, deterministic local tests, inspectable plans, and no secret values in diagnostics.

## Declarative programs

Semauri can build checked web, structured-data, and filesystem artifacts. Use a domain scope when operations need one:

```text
within web:
  create a web called Example.
  add a button called Start.
  set its color to blue.
end.
```

```sh
semauri build page.suri
semauri build page.suri -o page.html
```

## Compiler commands

| Command | Purpose |
| --- | --- |
| `config [PATH]` | Validate parameters and environment bindings |
| `check FILE` | Type-check and validate source |
| `fmt FILE [--check\|--write]` | Enforce canonical lowercase syntax |
| `tokens`, `ast`, `hir`, `optimize` | Inspect compiler stages |
| `symbols FILE` | Inspect typed names and symbol metadata |
| `explain FILE [--json]` | Explain compiler/lowering decisions as numbered text or structured JSON, without executing |
| `effects`, `plan`, `graph` | Inspect capabilities and runtime dataflow |
| `build FILE [-o PATH]` | Render build-time artifacts |
| `run FILE --allow EFFECT` | Execute supported effects explicitly |
| `test PATH` | Run assertion suites |
| `domains` | Print registered operation metadata |
| `version` | Print the compiler version |

Add `--json` to supported inspection and validation commands when consuming output programmatically.

## Repository map

- `src/compiler/frontend/`: tokens, parser, AST, and formatting.
- `src/compiler/semantic/`: typed HIR and semantic analysis.
- `src/compiler/runtime/`: lowering, effects, dataflow, and execution plans.
- `src/compiler/domains/`: registered declarative and runtime operations.
- `src/compiler/backends/`: renderers, CLI execution, and test runner.
- `examples/`: checked runnable programs.
- `learn_to_code/`: English and Spanish courses.
- `tests/`: compiler and language contracts.
- `editors/vscode/`: installable VS Code metadata and the canonical `source.semauri` TextMate grammar.

## Editor and GitHub highlighting

The repository includes a TextMate grammar for `.suri` and `.config.suri` files under `editors/vscode/`. It provides local VS Code highlighting now and is also published in the dedicated [semauri-language](https://github.com/edujbarrios/semauri-language) repository as the canonical public grammar for future GitHub Linguist integration.

GitHub only adds a new language after its public-usage threshold is met. Until `.suri` has sufficient independently distributed usage, GitHub will not provide official `semauri` fenced-code highlighting or a Semauri language bar entry. Do not mislabel Semauri as another language in `.gitattributes`; that would make repository statistics inaccurate.

## Contribution contract

Every language change should include parser/type/runtime coverage as applicable, a runnable `.suri` example, updates to the appropriate language guide or Quick Start, and passing tests. Preserve stable diagnostic codes and source spans. Keep effects explicit and never make compilation perform network, filesystem, model, or process work.

```sh
cargo test
cargo fmt --check
```

Licensed under Apache-2.0. See [LICENSE](../LICENSE).

### Natural-language loop variants

A typed for-each loop can now be written as `for item in values:`,
`for every item in values:`, or `for each item in values:`.
All forms use the same compiler semantics and end with `end.`.
This is a syntax convenience toward Python-like readability, not mutable Python iteration.

## Natural language and Python-like conditionals (v0.60.0)

Use `when` instead of `if`, `otherwise when` to chain alternative
conditions, and `is different from` as a synonym for `is not equal to`.
These are statically typed and do not relax the language's safety checks.

```text
let name be "Ada".
let greeting be "Hello, " joined with name.
when name is different from "Grace":
    print greeting.
otherwise:
    print "Welcome back".
end.
```

`joined with` joins two strings only and binds more weakly than arithmetic.
It is left-associative and will not silently convert numbers to text.
The three supported loop spellings are `for item in items:`,
`for each item in items:`, and `for every item in items:`,
with the customary `end.` terminator. Semauri is not Python itself.

## Membership in natural language (v0.61)

Use `item in collection`, `item not in collection`,
`item is in collection`, or `item is not in collection`.
These read like Python membership tests and reuse Semauri's existing
strictly typed `collection contains item` semantics. Lists require their
own element type, and strings support substring matching. The operators
also work inside `when` and `if` conditions. The `not in` form
negates a boolean result and retains short-circuiting of outer conditions.

```text
let names be a list of "Ada", "Grace".
assert "Ada" in names.
assert "Alan" not in names.
assert "aur" is in "Semauri".
```

## More conversational control flow (v0.61.0)

Conditions may end with optional `then` before `:`, for example
`when ready then:` or `if ready then:`. `elif condition:`
(and `elif condition then:`) is a shorter alternative to
`otherwise if condition:`. Both spellings share one `end.`
A `for` loop can optionally use `do:`, e.g.
`for each number in numbers do: ... end.`.
These forms preserve Semauri's existing typed branch and loop semantics.

## Additional pure scientific math functions (v0.62.0)

Five typed expressions use `FUNCTION of NUMBER` without granting any new
capabilities: `asinh` (inverse hyperbolic sine), `acosh` (inverse
hyperbolic cosine), `atanh` (inverse hyperbolic tangent), `log1p` (natural
logarithm of 1 + X, numerically accurate near zero), and `expm1`
(exp(X) - 1, also accurate near zero).

`acosh` requires X >= 1; `atanh` requires -1 < X < 1; `log1p`
requires X > -1. All five require finite numeric input and finite output.
Incorrect operand types raise `S465` and domain errors / overflow raise
`S466`. They are pure and constant-foldable and can be used with typed
`map every` lists; their names remain usable as variables except when
followed by `of`.

```text
let values be a list of 0, 0.1, 0.2.
let stable_logs be map every x in values to (log1p of x).
assert (acosh of 1) is equal to 0.0.
assert (atanh of 0) is equal to 0.0.
```

## Compact Python-style functions (v0.62.0)

The contextual functions `len of`, `abs of`, `mean of`, `min of`,
`max of`, `strip of`, `upper of`, and `lower of` are aliases for
Semauri's existing strict, pure operators. See the
[Python-style function guide](python-style-functions.md) and
[executable example](../examples/python_function_aliases.suri).

## Natural emptiness and blank-text predicates (v0.62.0)

Use `value is empty` or `value is not empty` to check if a typed list or
Unicode text has zero elements/characters. Use `text is blank` or
`text is not blank` to check whether a string consists only of whitespace
(including an empty string). The compiler translates these forms into the
existing typed `length of`, `trim of` and comparison expressions; there
is no implicit type conversion or new effect. A numeric or boolean value
is not a supported input. The contextual words `empty` and `blank`
remain usable as variable names outside these predicates.

```text
let names be a list of "Ada", "Grace".
assert names is not empty.
assert "" is empty.
assert "  " is blank.
assert "  hello  " is not blank.
```

## Natural numeric and parity predicates (v0.63.0)

Natural-language comparisons include `number is zero`, `number is
positive`, `number is negative`, `number is even`, and `number is odd`.
All five also accept `is not`. They reuse checked numeric `sign of`
(returns an exact integer sign for both integer and decimal values), and
`even of` / `odd of` (accept integral numbers in the signed 64-bit range).
Thus `0.0 is zero` and `minus 0.5 is negative` work as expected.
Non-number operands remain compile-time errors, and non-integral parity
values still produce `S426`. The predicate words are contextual and are
not reserved as variable names. This is a source-level spelling only; it
does not change Python-style numeric conversion rules or execute new effects.

```text
assert 0.0 is zero.
assert (minus 3) is negative.
assert 4 is even.
assert 7 is not even.
assert 3.5 is not negative.
```

## Natural comparison boundaries (v0.63.0)

You can express numeric bounds using `value is at least minimum` (inclusive
`>=`), `value is at most maximum` (inclusive `<=`), `value is above
limit` (`>`), and `value is below limit` (`<`). These are contextual
spellings of Semauri's existing type-checked comparison operators. Unlike a
repeated range check, the left-hand expression is only evaluated once per
comparison. Non-number operands produce the same compile-time diagnostic as
ordinary greater-than/less-than operations.

```text
let age be 21.
assert age is at least 18.
assert age is at most 65.
assert age is above 20.
assert age is below 25.
```

## Boolean and collection-membership predicates (v0.64.0)

Boolean conditions can be written as `flag is true`, `flag is false`,
`flag is not true` or `flag is not false`. The compiler reuses strict
`is equal to` / `is not equal to` checks; numbers and strings are not
converted to booleans.

The collection-membership phrases `item is one of collection` and
`item is not one of collection` are equivalent to the typed `in` and
`not in` expressions. Lists require an element of their declared type;
strings allow typed substring matching. No extra capabilities are granted.

```text
let names be a list of "Ada", "Grace".
let ready be true.
assert ready is true.
assert ready is not false.
assert "Ada" is one of names.
assert "Bob" is not one of names.
```

## Divisibility and multiples (v0.64.0)

Use `number is divisible by divisor` or `number is a multiple of divisor`
for a typed numerical divisibility check. Both support negation:
`is not divisible by` and `is not a multiple of`. They desugar to an
existing numeric `modulo` check against zero, preserving strict numeric
operand typing and the `S317` error on zero divisors. Integer remainder
now retains full 64-bit precision (including values greater than 2^53 and
`i64::MIN` modulo -1), in both optimization and evaluation. For fractional
numbers, divisibility follows floating-point remainder semantics, so
exact-decimal mathematical equality should not be assumed.

```text
assert 12 is divisible by 3.
assert 12 is a multiple of 4.
assert 13 is not divisible by 2.
assert 9007199254740993 is not a multiple of 2.
```

## Exact numerical ordering across integer and decimal values (v0.65.0)

Semauri compares signed 64-bit integer values without converting them to
binary64 floating point in ordering operations. This preserves exact
`greater than`, `less than` and inclusive ordering beyond 2^53,
including adjacent values near the signed 64-bit limits. Mixed integer /
decimal ordering compares the integer's exact value with the decimal's
integral and fractional parts; there is no rounding of the integer.

For backward compatibility, `is equal to`, `in`, `distinct of` and
other equality-based collection operations **retain Semauri's established
integer-versus-decimal storage distinction**: `1 is equal to 1.0` remains
false. This update intentionally changes numerical *ordering only*.
The optimizer and runtime use the same exact ordering algorithm.

```text
assert 9007199254740993 is greater than 9007199254740992.
assert 9007199254740993 is greater than 9007199254740992.0.
assert 1 is not equal to 1.0.
```

## Readable inclusion and negative text predicates (v0.65.0)

Both strings and typed lists now accept `collection includes value` and
`collection excludes value`, as natural aliases of the existing statically
typed `contains` and its boolean negation. `text does not contain value`
and `text does not include value` are additional English-like forms.

String-prefix and suffix tests support `text does not start with prefix`
and `text does not end with suffix`, complementing the existing
`starts with` and `ends with` positive forms. String-only predicates
preserve the `S391` type diagnostic, and typed membership preserves
`S344`. The compiler evaluates each input expression only once and uses
existing HIR/optimizer/runtime operations. `includes`, `excludes` and
`does` are contextual words, not reserved identifiers.

```text
let names be a list of "Ada", "Grace".
assert names includes "Ada".
assert names excludes "Alan".
assert "Semauri" does not start with "Python".
assert "Semauri" does not end with "thon".
assert "Semauri" does not contain "zzz".
```

## Integer radix string functions (v0.65.0)

Four contextual spellings provide three pure conversion functions:
`binary of value`, `octal of value`, `hex of value`, and
`hexadecimal of value`. Each converts an integral signed 64-bit number to
a lowercase text representation in base 2, 8 or 16, without prefixes.
Negative numbers use a leading `-` and the unsigned magnitude, rather
than two's-complement bits. Zero is `"0"`. Integer inputs retain exact
64-bit precision, including `i64::MIN`. Finite, integral decimals within
the signed 64-bit range are also accepted.

These operations return typed `string` values, can be used inside
`map every` expressions and are constant-folded when inputs are known.
Non-numeric operands report `S468`; fractional and out-of-range numeric
inputs report `S469`. They require no permissions and do not expose file,
network or other effects.

```text
assert (binary of 42) is equal to "101010".
assert (octal of 64) is equal to "100".
assert (hex of 255) is equal to "ff".
assert (hexadecimal of (minus 42)) is equal to "-2a".
```

## Concise collection and statistics functions (v0.65.0)

Eight additional contextual `of` functions reuse the existing fully
checked operators and optimizer paths, without changing runtime semantics:

| Short form | Canonical form |
| --- | --- |
| `avg of nums` | `average of nums` |
| `var of nums` | `variance of nums` |
| `stddev of nums` | `stdev of nums` |
| `total of nums` | `sum of nums` |
| `size of value` | `length of value` |
| `unique of list` | `distinct of list` |
| `head of list` | `first of list` |
| `final of list` | `last of list` |

Each name is still a normal identifier unless followed by `of`; existing
static typing, effect restrictions and source-aware errors are unchanged.
For example, `unique of` preserves the original order, and a numerical
list's standard deviation remains subject to the existing list checks.

## Natural primality predicates (v0.65.0)

Use `number is prime` or `number is not prime`. These are source-level
alternatives to `prime of number` and `not (prime of number)`, and share
the strict numeric type check (`S435`) and checked integral-domain errors
(`S436`). This does not introduce implicit coercions or additional effects.

```text
assert 13 is prime.
assert 21 is not prime.
let values be a list of 3, 5, 5, 7.
assert (size of (unique of values)) is equal to 3.
assert (total of values) is equal to 20.
```

## Concise median and product functions (v0.66.0)

Use `med of NUMBERS` for the checked median and `prod of NUMBERS` for the
checked product of a numeric list. Both are contextual aliases: `med` and
`prod` remain valid user-defined names unless followed by `of`.

```text
let values be a list of 2, 4, 6.
assert (med of values) is equal to 4.
assert (prod of values) is equal to 48.
```

They share the existing numeric list type checks, exact integer and
finite-value behavior, empty-list handling, optimizer paths and permissions
of `median of` and `product of`; neither introduces a new capability.

## Negative natural numeric bounds (v0.66.0)

Semauri accepts four negative forms of existing checked numeric comparisons:

| Natural predicate | Numeric meaning |
| --- | --- |
| `value is not above limit` | `value <= limit` |
| `value is not below limit` | `value >= limit` |
| `value is not at least limit` | `value < limit` |
| `value is not at most limit` | `value > limit` |

```text
let score be 75.
assert score is not below 50.
assert score is not above 100.
assert score is not at least 80.
assert score is not at most 50.
```

These are contextual spellings, not independent runtime operators: they preserve
precise signed 64-bit comparisons (including values above 2^53), source-aware
numeric type errors, optimizer behavior, and the existing effect requirements.
Each side is evaluated once; unrelated identifier names remain valid.

## Nonzero and inclusive sign predicates (v0.66.0)

For numeric values, Semauri accepts `is nonzero`, `is nonnegative` and
`is nonpositive` (as well as `is not nonzero`, `is not nonnegative` and
`is not nonpositive`). These are checked contextual operators, not reserved
identifier names:

| Predicate | Meaning |
| --- | --- |
| `value is nonzero` | `value != 0` |
| `value is nonnegative` | `value >= 0` |
| `value is nonpositive` | `value <= 0` |
| `value is not nonzero` | `value == 0` |
| `value is not nonnegative` | `value < 0` |
| `value is not nonpositive` | `value > 0` |

```text
assert 0.0 is nonnegative.
assert 0.0 is nonpositive.
assert (minus 4) is not nonnegative.
assert 8 is nonzero.
```

The implementation reuses the existing checked sign operation, preserving
full signed-64-bit precision and S417 diagnostic for nonnumeric values. No
implicit effects or new dependencies are introduced.

## Concise list and text operators (v0.66.0)

Three contextual `of` aliases make Semauri's checked collection operations
more concise without adding new runtime operations:

| Short form | Canonical form | Result |
| --- | --- | --- |
| `rev of items` | `reverse of items` | Reverses a typed list or Unicode string |
| `sort of items` | `sorted of items` | Returns a new sorted typed list |
| `flat of groups` | `flatten of groups` | Flattens one level of nested typed lists |

```text
let values be a list of 3, 1, 2.
assert (head of (sort of values)) is equal to 1.
assert (head of (rev of values)) is equal to 2.
let groups be a list of (a list of 1, 2), (a list of 3).
assert (size of (flat of groups)) is equal to 3.
```

The spellings remain valid variable names without `of`; they reuse the
existing type checks, optimizer, precision-aware sorting and capability rules.
No new dependencies or effect permissions are introduced.

## Explicit sorting direction (v0.66.0)

For a typed list, `sorted ascending of values` returns the usual sorted
order, while `sorted descending of values` returns that order reversed.
The shorter contextual `sort ascending of values` and `sort descending of
values` forms are equivalent. The existing `sorted of` and `sort of`
spellings continue to sort ascending by default.

```text
let scores be a list of 7, 1, 9, 3.
assert (head of (sorted ascending of scores)) is equal to 1.
assert (head of (sorted descending of scores)) is equal to 9.
assert (head of (sort descending of scores)) is equal to 9.
```

The order words are contextual, so `ascending` and `descending` are still
valid user-defined identifiers. Descending sorting composes the existing
checked `reverse` and `sorted` operators without evaluating the input
more than once. It preserves the original source-aware `S359` list type
checks, exact mixed integer/float ordering, purity, and capability policy.

## Natural prefix and suffix variants (v0.66.0)

In addition to `starts with` and `ends with`, Semauri supports the
contextual spellings `begins with` and `finishes with`. The negative
forms are `does not begin with` and `does not finish with`.

```text
assert "Café" begins with "Ca".
assert "Café" finishes with "fé".
assert "Café" does not begin with "ca".
assert "Café" does not finish with "fe".
```

All four forms reuse the existing strict Unicode-string `starts with` and
`ends with` semantics (case-sensitive, including empty prefixes/suffixes)
and the same source-aware S391 diagnostics for invalid operand types. Inputs
are evaluated exactly once, no runtime permissions are added, and the words
`begins` and `finishes` remain valid identifiers outside predicate syntax.

## Top-N and bottom-N typed list selection (v0.66.0)

Use `top COUNT from LIST` to select the highest values in descending
order, or `bottom COUNT from LIST` for the lowest values in ascending
order. The original list is never modified. `COUNT` must be a nonnegative
integer; zero gives an empty typed list, and a count larger than the list
returns all available values.

```text
let scores be a list of 8, 3, 12, 5.
assert (top 2 from scores) is equal to (a list of 12, 8).
assert (bottom 2 from scores) is equal to (a list of 3, 5).
```

Both words are contextual and remain valid variable names otherwise. The
compiler composes the existing checked sort and take operators: type errors
and integer/float ordering preserve their established semantics (including
values above 2^53). The count and list expressions are each evaluated once;
no additional capabilities or runtime dependencies are needed.

## Distinct cardinality of a typed list (v0.66.0)

Use `cardinality of items`, `count distinct of items`, or
`count unique of items` to calculate how many distinct typed values a
list contains. All three forms are equivalent to
`length of (distinct of items)`.

```text
let tags be a list of "rust", "python", "rust".
assert (cardinality of tags) is equal to 2.
assert (count distinct of tags) is equal to 2.
assert (count unique of tags) is equal to 2.
```

Semauri preserves the existing distinction between stored integer and
floating-point values (`1` versus `1.0`). Empty lists have cardinality
zero. These expressions require a typed list (error `S360` for other types),
evaluate the input once, and introduce no capabilities or runtime dependencies.
The new words remain ordinary identifiers outside their contextual syntax.

## Last-N slicing for lists and Unicode text (v0.66.0)

Use `take last COUNT from VALUE` to return the final N elements of a
list or Unicode scalar values of a string, in their **original order**.
Use `drop last COUNT from VALUE` to remove the final N elements. Both
forms return a new value without modifying the input.

```text
let scores be a list of 8, 3, 12, 5.
assert (take last 2 from scores) is equal to (a list of 12, 5).
assert (drop last 2 from scores) is equal to (a list of 8, 3).
assert (take last 2 from "ñ🌍a") is equal to "🌍a".
```

Like `take` and `drop`, counts must be nonnegative integers; a count
larger than the input clamps to its length. The parser reuses the existing
checked reverse and slice operators, preserving Unicode scalar handling,
type-aware diagnostics, optimizer behavior and effects. The existing
`take last from items` remains valid if `last` is a numeric variable.

## Sort and deduplicate typed lists (v0.66.0)

Use `sorted distinct of LIST` or `sorted unique of LIST` to return a
sorted list with duplicates removed. The shorter forms `sort distinct of
LIST` and `sort unique of LIST` mean the same thing. These expressions are
equivalent to `sorted of (distinct of LIST)`; they do not mutate the input.

```text
let scores be a list of 3, 1, 3, 2, 1.
assert (sorted unique of scores) is equal to (a list of 1, 2, 3).
assert (sort distinct of scores) is equal to (a list of 1, 2, 3).
```

They reuse the existing typed `distinct` and `sorted` operations, so
non-list inputs fail with the usual `S360` diagnostic, and elements must
still have a sortable type. Exact integer/decimal ordering and the original
storage-aware definition of duplicates are preserved. The list expression is
evaluated exactly once and no additional runtime permissions are needed.

## Unique top-N and bottom-N list selection (v0.66.0)

`top unique N from LIST` and `top distinct N from LIST` return the
highest N distinct values in descending order. `bottom unique N from LIST`
and `bottom distinct N from LIST` return the lowest values ascending.

```text
let values be a list of 5, 9, 5, 3, 9, 1.
assert (top unique 2 from values) is equal to (a list of 9, 5).
assert (bottom distinct 2 from values) is equal to (a list of 1, 3).
```

Counts are nonnegative integers, with zero yielding an empty list and values
above the number of unique elements returning all available elements. The
compiler composes existing checked Distinct, Sorted, Reverse and Take nodes,
preserving exact 64-bit numeric ordering, single evaluation and typed errors.
`top unique from values` still uses a numeric variable named `unique`.

## First-count slicing (v0.66.0)

`take first COUNT from VALUE` selects the first COUNT elements of a list
or Unicode scalar values of a string. `drop first COUNT from VALUE` removes
them. Both reuse existing typed take/drop, work with nonnegative integral
counts, and preserve the original value.

```text
let scores be a list of 1, 2, 3, 4.
assert (take first 2 from scores) is equal to (a list of 1, 2).
assert (drop first 2 from scores) is equal to (a list of 3, 4).
assert (take first 2 from "ñ🌍a") is equal to "ñ🌍".
```

Zero and overlong counts follow the original slicing rules. The words first
and last remain available in variable names and existing first-of syntax.


## Bounded and observable AI requests (v0.73.0)

AI code can bound prompt/input size before the runtime calls a local model:

```text
bound ai prompts to 4096 characters.
cap ai requests to 10.
memoize ai response cache.
let model be connect language model "local/example".
let response be prompt model with "Summarize the supplied text.".
let usage be report ai usage.
print usage.
```

- `bound ai prompts to COUNT characters.` accepts a nonnegative integer at runtime.
  It counts Unicode scalar values (not UTF-8 bytes or tokens). It applies to
  text generation, agent requests, image-inspection requests (including their
  image reference appended to the prompt), and embedding text. Reconfiguration
  replaces the prior limit. An overlength input fails with runtime S503 before
  using a cached response or consuming the request budget.
- `cap ai requests to COUNT.` now counts locally computed embeddings as AI
  requests, as well as generated and agent text; repeated cached text responses
  do not spend the request budget.
- `report ai usage` produces a string containing deterministic JSON:
  `requests`, `request_limit`, `prompt_character_limit`, `cache_hits`,
  and `cache_entries`. Unset limits are `null`. It never contains prompts,
  model identifiers or cached responses, needs no new effect permission, and
  reports only the current execution's state.
- These are runtime guardrails for **size and accounting**, not a guarantee
  against prompt injection, data leakage, model costs, or unsafe outputs.
  Only `local/*` model identifiers are executable without a provider adapter;
  inference and console effects still require explicit `--allow` flags.

See [examples/ai_governance.suri](../examples/ai_governance.suri).


## AI execution hardening (Unreleased, after v0.73.0)

The local AI runtime now enforces agent turn budgets and allows explicit
resource bounds and cache controls:

```text
cap ai requests to 4.
bound ai prompts to 500 characters.
restrict ai outputs to 200 characters.
memoize ai response cache.
constrain ai cache to 2 entries.
let model be connect language model "local/test".
let assistant be compose agent using model system "Be concise." max 2 turns.
let first be ask assistant with "Hola".
let second be ask assistant with "Hello".
let metrics be report ai usage.
print metrics.
reset ai response cache.
```

- `compose agent using MODEL system TEXT max N turns` validates a
  nonnegative integer `N`. Each successful `ask AGENT with TEXT` (including
  cache hits) consumes one agent turn. An additional attempt fails with S503
  before spending the global request budget. Unsuccessful attempts do not
  consume a turn. Separate agent instances have separate turn counters.
- The system message is **kept separate** from the user prompt and used to
  distinguish cache entries. The current `local/*` backend is only a
  deterministic simulator, **not** a real language model that follows system
  instructions; provider adapters are still required for real inference.
  Prompt bounds cover the system message plus the user's prompt.
- `constrain ai cache to N entries.` sets a nonnegative integer limit on
  memoized responses. The oldest inserted entry is evicted first (FIFO);
  cache hits do not refresh order. Lowering the limit immediately evicts
  excess entries. Zero disables storage without disabling model execution.
  Without an explicit limit, existing unbounded-cache behavior remains.
- `reset ai response cache.` clears cached text while retaining configured
  limits, whether caching is enabled, and cumulative usage counters.
- `restrict ai outputs to N characters.` sets a nonnegative integer
  Unicode-scalar limit on generated text (including agent and vision text),
  enforcing it for both new results and cache hits. Oversized fresh results
  consume a request but are rejected before they can be returned or cached.
  Embedding fingerprints are opaque data and are unaffected by this limit.
- `report ai usage` additionally includes `cache_capacity`,
  `cache_evictions`, `output_character_limit`, `agent_count`,
  and `agent_turns` in its JSON string. No prompts, model identifiers or
  system instructions appear in the report.

These runtime constraints do not grant permissions or provide provider-side
billing, token accounting, prompt-injection defenses, or true model inference.

## Validación de JSON estructurado para agentes (en desarrollo)

`validate ai json RESPONSE against schema SCHEMA` comprueba un documento JSON
sin invocar modelos, red ni archivos. Ambos argumentos son cadenas y el
resultado es un booleano. Por ejemplo:

```text
let contract be "{\"type\":\"object\",\"required\":[\"label\"],\"properties\":{\"label\":{\"type\":\"string\"}},\"additionalProperties\":false}".
let reply be "{\"label\":\"cat\"}".
assert validate ai json reply against schema contract.
```

La sintaxis admite **un subconjunto explícito** de JSON Schema: los tipos
`object`, `array`, `string`, `number`, `integer`, `boolean` y `null`;
`properties`, `required` y `additionalProperties` para objetos;
`items`, `minItems` y `maxItems` para listas; y `minLength` y
`maxLength` para cadenas (contando caracteres Unicode). No convierte tipos.

Un esquema erróneo o con palabras no reconocidas produce un error S503
explícito; un documento JSON inválido o incompatible produce `false`.
El documento está limitado a 1 MiB y el esquema a 64 KiB; además existen
límites de profundidad y número de propiedades. No se trata de un validador
completo del estándar JSON Schema ni protege por sí solo de inyecciones
de instrucciones. Consulta
[el ejemplo ejecutable](../examples/ai_structured_json.suri).
