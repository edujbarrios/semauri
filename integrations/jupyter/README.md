# Semauri Jupyter kernel (experimental)

Install Semauri's Rust executable on `PATH` first, then run:

```sh
python3 -m pip install -e ./integrations/jupyter
semauri-jupyter-install --user
python3 -m pip install jupyterlab
jupyter lab
```

Select **Semauri** as the kernel when opening a notebook. Each code
cell contains a standalone Semauri `.suri` program:

```text
let numbers be range from 1 to 5.
let squares be map every n in numbers to (square of n).
print squares.
```

Expected text output: `[1, 4, 9, 16, 25]`.

**Cells are deliberately independent.** The compiler evaluates entire
programs during lowering; concatenating and replaying prior cells would
repeat side effects (including filesystem/AI work) and silently change
semantics. For now, declare reusable functions or imports inside the same
cell; notebook execution does not preserve previous cell bindings. This
limitation will only be lifted after a true compiler-backed incremental
session protocol exists.

Only the `console_write` effect is granted by default for notebook
output. To explicitly add additional capabilities, set
`SEMAURI_ALLOW_EFFECTS=console_write,filesystem_write` before launching
Jupyter; treat every additional permission as a security decision.
`SEMAURI_BINARY=/path/to/semauri` chooses the compiler executable,
`SEMAURI_CELL_TIMEOUT=60` configures the per-cell timeout (seconds).
No shell interpretation is performed. Errors use Semauri's native diagnostic
text, and temporary `.suri` source files are removed after each cell.

Use `python3 -m unittest discover -s integrations/jupyter/tests` after
installing this package to test the adapter offline.

## Notebook-specific cell directives

The first nonblank line can contain `%%check`, `%%explain`,
`%%explain --json`, `%%effects`, `%%plan`, `%%tokens`, `%%ast`,
`%%hir`, `%%optimize`, `%%symbols`, `%%graph`, `%%audit`, or `%%fmt`.
These commands inspect Semauri source without granting any additional
application effects. `%%fmt` prints canonically formatted source but never
modifies either the original cell or a source file.
The `%%audit` directive reports the compiler's ML reproducibility audit.
They are **kernel directives**, not source-language syntax.

```text
%%explain
within web:
  create a web called Example.
  add a button called Start.
end.
```

Open [the example notebook](examples/hello_semauri.ipynb), choose the
**Semauri** kernel, then execute each independent cell. Keyword
completion includes locally declared names and scientific operators, including
`log2`, `log10`, `asin`, `acos` and `atan`. These numerical functions use
real-valued domains and radians for inverse trigonometry.
Use Shift+Tab for offline contextual help on common keywords and directives;
completion and inspection never run a cell or grant capabilities.
JupyterLab and Notebook work without `anywidget`
or a separate frontend extension.

## Natural-language programming

Completions include `when`, `joined` and `different`.
For example: `let greeting be "Hello, " joined with "Ada".`

## Semauri 0.61.0 additions

Completions and contextual help include `elif`, `then`, `do` and
`in`. Typed membership uses `item in list` or `item not in list`,
and branches support `elif CONDITION then:`.

## Additional scientific functions in v0.62.0

Offline completion and help now include `asinh`, `acosh`, `atanh`,
`log1p`, and `expm1` for pure, typed numerical expressions.

## Natural predicates and boundaries (v0.63.0)

Notebook completions include `zero`, `positive`, `negative`, `even`,
`odd`, `least`, `most`, `above`, and `below`. They support
natural source such as `assert 3 is positive.` and `assert 3 is at least 2.`.

## Natural predicates in Semauri 0.64.0

Notebook completion and offline help now support `one`, `divisible`
and `multiple` expressions, including `value is one of items`,
`value is divisible by divisor`, and `value is a multiple of divisor`.

## Semauri 0.65.0 additions

Completion and offline help include `includes`, `excludes`, negative text predicates, `binary`, `octal`, `hex`/`hexadecimal`, `prime`, and eight concise typed collection/statistical functions (`avg`, `var`, `stddev`, `total`, `size`, `unique`, `head`, `final`). The kernel continues to execute independent cells with explicit effects only.

## Natural negated bounds (v0.66.0)

Offline help describes all four new numerical comparisons: `is not above`,
`is not below`, `is not at least`, and `is not at most`. These expressions
share Semauri's checked numeric operators, with no new kernel permissions.

## Inclusive sign predicates (v0.66.0)

Completions and offline help recognize `nonzero`, `nonnegative`, and
`nonpositive`, including all three `is not` spellings. These are
numeric-only, pure Semauri expressions.

## Concise collection operators (v0.66.0)

Offline completion and help recognize `rev of`, `sort of`, and `flat of`
as checked, contextual aliases for reverse, sorted, and one-level flatten.

## Semauri 0.66.0 language and collection improvements

Jupyter completion and offline help now cover natural sign and negated
numeric bounds, Unicode text boundary predicates, `rev`/`sort`/`flat`,
explicit sorting direction, `top`/`bottom` selection, cardinality,
`take first`/`drop first` and `take last`/`drop last` Unicode slicing,
sorted deduplication and unique ranked selection. Per-cell isolation and
explicit effect permissions remain unchanged.
