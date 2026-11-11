## Unreleased

### Boolean logic and dynamic expression polish (core language)
- Add the contextual `xor` operator with static boolean typing, clear precedence between `and` and `or`, constant folding, compatible formatting and regression tests. The word remains usable as a variable name.
- Permit runtime-dependent `not`, `xor`, and typed equality/inequality through effect-free SSA `core` operations, instead of rejecting all such expressions with S335.
- Keep lazy `and` / `or` semantics unchanged pending control-flow lowering; no new dependencies or implicit permissions, and no version bump.


### Strict structured AI JSON validation (development; no release)
- Safely allow nominal `ai.response` values to flow into ordinary string parameters (including JSON validation) without weakening nominal model/tool/agent type separation.
- Reject duplicate JSON object keys at every depth in both input data and schema contracts, rather than silently taking the final value.
- Add pure `validate ai json RESPONSE against schema SCHEMA`, a typed Boolean expression for structurally validating AI JSON before use.
- Support strict JSON Schema-like primitive, object and array types, required and nested properties, unknown-field rejection and Unicode-aware size constraints. Unsupported schema keywords raise S503 instead of being ignored; malformed response data returns false.
- Bound schema and document sizes, schema nesting and property counts. Add executable examples, Rust regression tests and Jupyter completion vocabulary; no additional permissions or packages.
- Support bounded string enumerations and `normalize ai json RESPONSE against schema SCHEMA` for deterministic JSON serialization after strict validation; mismatches raise S503 without silent data repair.


### AI runtime correctness and resource safety (development; no release)
- Enforce configured per-agent maximum turns, retain distinct system contexts for cache isolation, and reject invalid turn counts instead of silently ignoring them. The local model remains a deterministic simulator, not an instruction-following provider.
- Add explicit FIFO cache limits (`constrain ai cache to N entries.`) and cache reset (`reset ai response cache.`); report evictions and capacity without revealing prompts.
- Add runtime output guards (`restrict ai outputs to N characters.`) for generated and cached text, plus expanded AI usage metrics and regression tests.
- Preserve existing effects, default cache behavior and v0.73.0 package metadata; work remains unpublished on a development PR.


## 0.73.0 — 2026-11-11

### Safer, observable AI workflows
- Add `bound ai prompts to COUNT characters.` to enforce a runtime length budget on prompts, image requests and embedding inputs using Unicode scalar counts; violations fail before consuming request budget or serving cached text.
- Add pure `report ai usage` returning stable JSON for completed AI requests, configured limits, text-response cache hits and cache entries; it exposes no prompt contents.
- Count locally computed embeddings against `cap ai requests to COUNT.` and enforce the same prompt bound on embedding inputs.
- Keep explicit `--allow` effect gates, existing local-model backend limits and no additional Rust dependencies.

### Namespaced library imports
- Support `import "library:NAME" as alias.` and `invoke alias.function to ...` with collision-safe resolution across nested imports, local files and pinned external libraries.
- Reject malformed, reserved, duplicated and unknown aliases with S416; preserve existing unaliased import behavior and source-aware diagnostics.

### Immutable typed records and dictionaries
- Add immutable structural `record of ("key": VALUE)` literals, exact record-parameter typing, `field "key" of RECORD` and JSON conversion.
- Add homogeneous `dictionary of ("key": VALUE)`, `entry "key" of DICTIONARY`, typed empty dictionaries, safe membership and sorted keys queries.
- Add lazy recoverable `entry "key" of DICTIONARY with default EXPR`, with static fallback typing and diagnostics S420–S423.
- Add examples, optimizer and formatter handling, effect accounting and Rust regression tests.

## 0.72.0 — 2026-11-11

### External and complementary libraries
- Add optional project-level `.semauri-libraries.json` catalogs for up to 32 explicitly declared offline third-party `.suri` source libraries, including libraries installed outside the project root.
- Match each dependency's canonical name and pinned `MAJOR.MINOR.PATCH` metadata to its `.semauri-library.json` descriptor, and resolve `import "library:NAME".` at compile time.
- Scope nested local imports to each canonical library root; reject undeclared dependencies, identity mismatches, import cycles and symlink/path escapes (S417/S416), without granting runtime capabilities.
- Include `semauri libraries PROJECT|FILE` validation and introspection, empty catalogs in new projects, Rust regression tests and a complete example.
- This first stage does not include remote resolution, registry publication, cryptographic lockfiles or native plugin loading.


### Additional natural-language syntax
- Add statically checked numeric comparisons `is not greater than`, `is not less than`, `is no greater than`, `is no less than`, and strict negations of inclusive greater/less comparisons. Reuse exact checked ordering operators, with no new effects.
- Allow `range from A through B` as a contextual, inclusive alias for `range from A to B`, preserving existing range validation.
- Allow a final comma in `a list of 1, 2,.` and `invoke combine to (1, 2,)`; reject double commas and preserve function arity checks.
- Add Rust regression coverage, formatting tests, an executable example and language-guide documentation.

### Rust-only internals and development tools
- Replace the Python Jupyter adapter and installer with a native Rust ZeroMQ kernel, including HMAC-authenticated messages, independent cell execution, capability controls, read-only directives, completions, and Rust unit/protocol tests.
- Replace Python benchmark and report scripts with `semauri_bench`, migrate GitHub Actions workflows, and remove all repository-owned Python source files.
- Accept a leading UTF-8 BOM in direct source and imported modules, while keeping embedded BOM characters invalid, with Rust regression tests.
- The compiler, kernel metadata and release tooling are versioned together for v0.72.0.

## 0.71.0 — 2026-11-10

### Numerical sequences: cumulative and adjacent operations
- Add `running total/sum` and `cumulative total/sum` of numeric lists, preserving checked i64 precision for integer prefix totals and reporting S471 on overflow or non-finite results.
- Add `running average` and `cumulative average` of numeric lists, returning each prefix mean with checked accumulation.
- Add `adjacent differences` and `successive differences` to compute the checked signed change from each previous element (S470).
- Add `running minimum/maximum/product` and `cumulative minimum/maximum/product`, with contextual `min` and `max` spellings. Extrema retain exact mixed integer/decimal ordering; cumulative products use checked integer multiplication, switching to finite decimal arithmetic when a decimal occurs (S355/S387).
- All numeric scans are pure, statically checked `list<number>` to `list<number>` operations. They evaluate the source once, run in linear time, preserve order, and return an empty list for empty input. Invalid types report S353.

### Model evaluation: squared and absolute error metrics
- Add `sum of squares of LIST`, `mean square of LIST`, `root mean square of LIST`, and `rms of LIST`, implemented using existing checked square, typed mapping, aggregation, and square root.
- Add `sum of magnitudes of LIST`, `mean magnitude of LIST`, `max magnitude of LIST`, `mean absolute error of LIST`, and `mae of LIST`; checked absolute values preserve exact signed integer behavior.
- Add `mean squared error of ERRORS` / `mse of ERRORS` and `root mean squared error of ERRORS` / `rmse of ERRORS`.
- Add signed `mean bias error of ERRORS` / `mean error of ERRORS` / `mbe of ERRORS`. These consume lists of residuals (observed minus predicted), not two independent input lists.
- All expressions reuse existing arithmetic, type checking, and error diagnostics; they add no capabilities, external dependencies, or reserved identifiers. Empty inputs are accepted for sums but rejected for mean, RMS, minimum, and maximum where applicable.

### Examples, tests, and tooling
- Add executable `examples/squared_statistics.suri`, `examples/absolute_error_statistics.suri`, `examples/model_error_metrics.suri`, and `examples/running_statistics.suri`.
- Add Rust regression suites covering static typing, source locations, exact large integers, decimal semantics, overflow, empty inputs, contextual names, canonical formatting, HIR/optimizer behavior, and capability/effect analysis.
- Update the language guide, VS Code syntax highlighting, and Jupyter completion and offline help.

## 0.70.0 — 2026-11-10

### Checked numeric difference expressions
- Add contextual `difference between A and B`, `absolute difference between A and B` and `squared difference between A and B` for numeric expressions, especially model loss/measurement calculations.
- Correct the shared subtraction implementation to preserve exact signed 64-bit integers (without f64 round trips) and reject signed overflow or non-finite results with S470; constant folding uses the same checked operation. Reuse the existing absolute-value, square, and S314/S388/S422 diagnostics, with one-time evaluation of each operand.
- Add runnable examples, Rust conformance/error/formatter tests, Jupyter completions and help, VS Code highlighting, and language documentation. No new runtime effects or Rust dependencies.

## 0.69.0 — 2026-11-10

### Machine-readable formatter reports
- Add `semauri fmt FILE --json` to preview formatted source in a structured report without modifying the file.
- Add `semauri fmt FILE|DIRECTORY --check --json` and `--write --json` with stable `ok`, `status`, `mode`, `files_checked`, `files_changed`, and `changed_files` fields for editors and CI. Preview reports also contain `formatted`.
- Preserve text-mode output, exit statuses, bounded directory discovery, symlink exclusions, preflight-before-write behavior, and structured source diagnostics. No language syntax, effects, or dependencies change.

## 0.68.0 — 2026-11-08

### Runtime and compiler evaluation performance
- Replace quadratic scan-based `distinct of`, `cardinality of` and unique counting with hash-bucket deduplication and exact equality verification. Preserve first-occurrence order, nested-list semantics, and integer-vs-decimal identity.
- Remove an unconditional per-item clone from mapped, filtered, counted, quantified and partitioned list transformations by moving the immutable original item out of the temporary iterator frame.
- Add large-input and storage-sensitive regressions plus paired same-runner benchmarks against the pull request base. No new dependencies, syntax, permissions, or effect changes.

## 0.67.0 — 2026-11-08

### Four-digit Unicode literals and ASCII escape
- Accept four-digit `\uXXXX` Unicode BMP scalar escapes in ordinary single- and double-quoted strings alongside `\u{HEX}` and `\UXXXXXXXX` forms; reject incomplete, non-hexadecimal, and surrogate values with source-located S104 diagnostics.
- Add `\e` for ASCII escape (U+001B), equivalent to `\x1B`. Raw and triple-quoted strings retain literal backslashes.
- Add regression tests, a checked example, VS Code highlighting and updated language documentation, with no new dependencies or runtime capabilities.

## 0.66.1 — 2026-11-07

### Consolidated patch release
- Rebuild and republish the complete v0.66.0 language feature set as v0.66.1, including PRs #316–#327. No language/runtime behavior changes after v0.66.0.
- Update CLI/library package and citation metadata, keep Jupyter adapter v0.11.0, and produce six platform archives with SHA256SUMS.


## 0.66.0 — 2026-11-07

### Explicit first-count Unicode and list slicing
- Add contextual `take first COUNT from VALUE` and `drop first COUNT from VALUE` as synonyms for checked prefix take/drop.
- Preserve typed S363/S364 diagnostics, nonnegative bounds, Unicode scalar slicing, effects, and identifiers named first.
- Add Rust tests, example, formatter, Jupyter offline help/completion and guide; no new capabilities or version bump.

### Unique top-N and bottom-N list selection
- Add `top/bottom unique COUNT from LIST` and `top/bottom distinct COUNT from LIST`, selecting highest or lowest distinct elements in sorted order.
- Reuse checked `Take(Sorted(Distinct(LIST)))` (with `Reverse` for `top`), including strict S360/S363/S364 diagnostics, exact integer comparison and one-time evaluation.
- Add an executable example, Rust tests, formatter and Jupyter help, and language guide; no new permissions, dependencies or release changes.


### Checked sorted-distinct collection expressions
- Add contextual `sorted distinct of LIST`, `sorted unique of LIST`, `sort distinct of LIST`, and `sort unique of LIST` to deduplicate then sort a typed list.
- Reuse existing `Sorted(Distinct(LIST))` HIR: preserve strict type checks, exact numeric sorting (including mixed integer/float values), duplicate storage semantics and one-time evaluation.
- Include Rust regressions, executable example, formatter, Jupyter offline help/completion, and guide. No new effects, permissions or dependencies.


### Suffix slicing of lists and Unicode strings
- Add `take last COUNT from VALUE` and `drop last COUNT from VALUE`, preserving the original order of the resulting list or Unicode text.
- Lower each to checked `Reverse(Take/Drop(COUNT, Reverse(VALUE)))` AST operations without reevaluating either expression; keep count bounds, source-aware diagnostics, effects and existing `last` identifiers.
- Include a runnable example, Rust tests, formatter regression coverage, Jupyter help/completion and guide. No new dependencies or permissions.

### Distinct cardinality for typed lists
- Add `cardinality of LIST`, `count distinct of LIST`, and `count unique of LIST` as contextual expressions mapping to existing statically checked `Length(Distinct(LIST))`.
- Preserve distinct storage semantics (integers and decimals remain distinguishable), stable S360 list-type diagnostics, optimizer and effects without repeating evaluation.
- Include regression tests, executable example, formatter, VS Code and Jupyter completion/help and guide. No new runtime permissions.

### Checked top-N and bottom-N list selection
- Add contextual `top COUNT from LIST` and `bottom COUNT from LIST` expressions, composing typed `sorted`, `reverse`, and `take` operators without evaluating the source twice.
- Keep exact mixed integer/float ordering, stable list behavior, count bounds and source-aware `S359`/`S363`/`S364` errors.
- Provide Rust regressions, runnable example, formatter, VS Code, Jupyter completion/help, and guide. No new effects or runtime dependencies.

### More natural Unicode prefix and suffix predicates
- Support `begins with` / `finishes with` and `does not begin with` / `does not finish with` using existing checked string-prefix and string-suffix operators.
- Preserve case-sensitive Unicode behavior, S391 type diagnostics, single operand evaluation, contextual identifiers, optimizer and effects.
- Update formatter, executable examples, Rust tests, VS Code highlighting and Jupyter help/completion without new dependencies.

### Explicit ascending and descending typed sorting
- Add `sorted ascending of LIST` and `sorted descending of LIST`, plus the contextual `sort ascending/descending of LIST` forms.
- The ascending form reuses checked `sorted` and the descending form composes `reverse` over `sorted`; preserve exact signed 64-bit ordering, type diagnostics, and single list evaluation.
- Extend executable examples, regression tests, formatting, editor highlighting, Jupyter completions, and guide without new runtime effects.

### Concise list and text operators
- Add contextual `rev of`, `sort of` and `flat of` as pure, statically typed aliases for `reverse of`, `sorted of` and `flatten of`.
- Retain original string/list behavior, one-level flattening, strict operand diagnostics, exact numeric sorting and single evaluation.
- Extend examples, regression tests, source formatter, VS Code and Jupyter completion/help.

### Natural nonzero and inclusive sign predicates
- Add contextual `is nonzero`, `is nonnegative`, and `is nonpositive`, along with all three `is not` variants, using existing typed sign comparisons.
- Preserve exact 64-bit sign behavior, S417 numeric diagnostics, optimizer, formatting, effects, and valid contextual identifiers.
- Add executable example, Rust regression tests, VS Code grammar, Jupyter completion/help and language guide.

### Natural negated numerical bounds
- Add `is not above`, `is not below`, `is not at least`, and `is not at most` as contextual exact typed numeric comparisons.
- Reuse existing inclusive/strict ordering and S314 type checks without extra evaluations, implicit effects, or runtime dependencies.
- Add boundary and precision tests, a runnable example, source formatting, language guide, and Jupyter offline help coverage.

### Short median and product functions
- Add contextual `med of` and `prod of` aliases for existing statically typed `median of` and `product of` numerical list operations.
- Preserve source-located type and evaluation checks, exact signed integer behavior, canonical formatting and effect limits.
- Include runtime/type/format tests, an executable example, VS Code recognition, Jupyter completion/help and guide coverage.

## 0.65.0 — 2026-11-07

### Compact collections, statistics and primality
- Add contextual `avg of`, `var of`, `stddev of`, `total of`, `size of`, `unique of`, `head of`, and `final of` as typed aliases of existing pure operations.
- Add `is prime` / `is not prime`, reusing checked `prime of` and negation; retain S435/S436 diagnostics.
- Extend VS Code, Jupyter completion and help, and add runnable examples and type/format/optimizer regression coverage.


### Checked integer radix conversions
- Add `binary of`, `octal of`, `hex of`, and `hexadecimal of` to format signed 64-bit integers as typed strings without losing precision.
- Support zero, signed negatives (including i64::MIN), integral decimal inputs and source-aware `S468`/`S469` errors for invalid operands.
- Wire type checking, constant folding, runtime, VS Code grammar, Jupyter completion/help and executable regression examples.


### Inclusion and negated text predicates
- Add contextual `includes` / `excludes` for typed lists and text; add `does not contain`, `does not include`, `does not start with` and `does not end with`.
- Reuse typed Contains/StartsWith/EndsWith and Not operators with no duplicate evaluation, runtime dependency or new effects; add comprehensive Rust examples and editor help.

### Exact numerical ordering
- Compare signed 64-bit integers exactly in greater-than, less-than and inclusive ordering, including adjacent values above 2^53, without silently rounding through binary64.
- Compare integer/decimal ordering without losing integer precision at signed boundaries.
- Preserve established storage-sensitive equality, collection membership and distinct semantics; test optimizer/runtime agreement.

## 0.64.0 — 2026-11-06

### Typed mathematical divisibility
- Add `is divisible by`, `is not divisible by`, `is a multiple of`, and `is not a multiple of` as English-like numeric predicates.
- Preserve exact i64 modulo arithmetic in optimizer and runtime rather than rounding integers through binary64; retain zero-divisor diagnostic `S317`.
- Add tests for signed boundaries, high-precision integer parity, contextual identifiers, and strict numeric types.

### Natural Boolean and typed membership tests
- Add `is true`, `is false`, `is not true` and `is not false` as strict Boolean comparisons.
- Add `is one of` and `is not one of` as contextual, type-checked membership aliases, preserving existing `S344` diagnostics.
- Extend formatter, docs, executable examples, Rust regression tests and editor/kernel assistance.

## 0.63.0 — 2026-11-03

### Readable numeric conditions
- Add statically typed `is zero`, `is positive`, `is negative`, `is even`, and `is odd`, including `is not` variants.
- Reuse existing sign, parity, comparison, finite-domain checks and optimizer paths; preserve contextual identifiers.

### English-like numeric bounds
- Add `is at least`, `is at most`, `is above`, and `is below` with existing inclusive/strict numeric ordering checks.
- Preserve strict type checking and single evaluation, and extend formatter, docs, notebook and VS Code support.

## 0.62.0 — 2026-11-03

### Typed math expressions
- Add pure `asinh of`, `acosh of`, `atanh of`, `log1p of`, and `expm1 of` with finite results, mathematical domain checks, and typed diagnostics.
- Extend AST/HIR, constant folding, evaluation, formatter, VS Code, Jupyter and the language guide with regression tests and examples.
- Preserve finite-domain diagnostics S465/S466 without new dependency or capability grants.

### Python-friendly function aliases
- Add contextual shorthand `len of`, `abs of`, `mean of`, `min of`, `max of`, `strip of`, `upper of` and `lower of` for existing typed, pure operators.
- Document short-form semantics while preserving legacy operators, type errors, evaluation, and optimization.

### Natural empty and blank predicates
- Add `is empty`, `is not empty`, `is blank`, and `is not blank`. Reuse existing typed Length/Trim/Equal operators and preserve compile-time diagnostics.
- Cover Unicode whitespace, typed list and string inputs, source formatting and Jupyter/VS Code tooling.

## 0.61.0 — 2026-11-03

### Natural-language and Python-like syntax
- Add `item in collection` / `item not in collection` with optional `is` for typed list membership and Unicode substring search. Existing `contains` semantics and `S344` diagnostics are preserved.
- Add `elif` as a natural Python-style conditional branch, optional `then:` after conditions, and optional `do:` in for-each loops.
- Preserve `otherwise if`, `if`, `when`, and conventional `for` loop spellings.
- Include compiler and runtime regression tests, examples, docs, editor highlighting, and notebook completion/help updates.

## 0.60.0 — 2026-11-01

### Natural language and typed expressions
- Resolve the historical PR #109 against the Rust compiler, preserving already-existing typed `is not equal to`.
- Add `joined with` as left-associative, string-only concatenation, type diagnostics `S343`, optimized folding and formatter support.
- Introduce `when` / `otherwise when` for conditionals and `is different from` for strictly typed inequality.
- Expand VS Code syntax, Jupyter completions/help, examples, tests and reference docs.
- Include Python-like `for item in values` and `for each item in values` from merged PR #299.

## 0.59.0 — 2026-11-01

- Add checked, pure `sinh of`, `cosh of` and `tanh of` numeric operators with type diagnostics, optimizer support and regression tests.

## 0.58.0 — 2026-11-01

### Scientific language enhancements
- Introduce pure, statically typed `log2 of NUMBER` and `log10 of NUMBER` operations for binary and decimal logarithms.
- Add inverse trigonometric functions `asin of NUMBER`, `acos of NUMBER`, and `atan of NUMBER` (all angle results expressed in radians).
- Guard real domains and finite results: logarithms require strictly positive arguments; arcsine and arccosine require values between -1 and 1; invalid values report compiler evaluation error `S466`. Nonnumeric operands report `S465`.
- Integrate new expressions into AST serialization, HIR analysis, constant folding, runtime evaluation, formatter, VS Code highlighting, Jupyter completion and contextual help.

### Quality and release
- Add executable examples, positive/negative boundary tests, type error checks, contextual-name compatibility tests, formatter and effect-analysis tests.
- Extend the language guide, editor tests, Jupyter tests and packaging metadata; Jupyter kernel adapter version 0.3.0.
- No new dependencies or implicit runtime effects. Keep a deliberately bounded scientific numerics domain, not arbitrary-precision or complex arithmetic.

## 0.57.0 — 2026-11-01

### Language and numerical safety
- Accept ASCII bell (`\\a`) and vertical tab (`\\v`) escapes in ordinary single- and double-quoted string literals; keep raw strings literal, invalid-escape diagnostics and the canonical formatter consistent.
- Fix a numeric conversion boundary: a floating-point value of 2^63 is not silently saturated to `i64::MAX`. Add regression coverage for both signed 64-bit bounds.
- Extend VS Code escape highlighting and tests for the new language literals.

### Notebook usability
- Add cell-local identifier completion (immutable bindings and defined functions) and completion of scientific/numeric operators.
- Add offline contextual keyword help, plus a read-only `%%audit` directive for the compiler ML audit.
- Extend Jupyter adapter tests, keeping cells independent and effect grants unchanged; adapter package version 0.2.0.

### Compatibility and release
- Align crate, lockfile, citation, README and Jupyter language version with **0.57.0**.
- No new Rust dependencies, I/O privileges or implicit cross-cell state.

The interpreter remains a statically typed language with explicit effects, not yet a complete replacement for general-purpose languages. AI training declarations remain inspectable plans rather than actual model training.

## 0.56.0 — 2026-10-30

### Scientific language features
- Add finite `exp of`, `ln of`, `sin of`, `cos of` with explicit diagnostics, regression tests and examples ([#291](https://github.com/edujbarrios/semauri/pull/291)).
- Add checked integer and finite fractional exponentiation, `power of BASE to EXPONENT` ([#292](https://github.com/edujbarrios/semauri/pull/292)).
- Add numerically stable population `variance of` and `stdev of` over typed numeric lists ([#293](https://github.com/edujbarrios/semauri/pull/293)).

### Packaging
- Align crate, lockfile, README, citation and Jupyter version with **0.56.0**.
- Preserve the concise Quick Start and detailed guide; no new runtime permissions or dependencies.

This release broadens numerical programming but does not implement arbitrary
mutation, complete standard-library I/O or production ML training.

## 0.55.0 — 2026-10-30

### Expanded language literal syntax
- Parse leading-dot fractions (`.5`, `.25e+2`) while preserving statement periods and checked numeric diagnostics ([#284](https://github.com/edujbarrios/semauri/pull/284)).
- Add multiline triple-quoted UTF-8 text blocks with literal newlines, quotes and backslashes for prompts and templates ([#285](https://github.com/edujbarrios/semauri/pull/285)).
- Decode fixed-width eight-digit `\UXXXXXXXX` Unicode scalar escapes in ordinary strings, checking surrogates and out-of-range values ([#286](https://github.com/edujbarrios/semauri/pull/286)).

### Developer tooling and releases
- Extend VS Code highlighting for new fractions, Unicode escapes and triple-quoted text; retain grammar regression coverage ([#287](https://github.com/edujbarrios/semauri/pull/287)).
- Align Rust crate, lockfile, Jupyter language version, README citation and `CITATION.cff` with **0.55.0**.

These changes are backward-compatible and introduce no new dependencies or runtime permission requirements.

## 0.54.0 — 2026-10-30

### Language improvements
- Add `0z`/`0Z` base-36 integers, underscore separators and signed 64-bit overflow checks ([#279](https://github.com/edujbarrios/semauri/pull/279)).
- Add two-digit `\xNN` character escapes to both quote styles, preserving located errors for malformed escapes ([#280](https://github.com/edujbarrios/semauri/pull/280)).
- Add hash-delimited raw strings with embedded quotes, literal backslashes and matching hash suffixes ([#281](https://github.com/edujbarrios/semauri/pull/281)).

### Editor support and reproducibility
- Highlight base-36 numbers, hex escapes and hash raw-string scopes in the VS Code grammar ([#282](https://github.com/edujbarrios/semauri/pull/282)).
- Update Rust crate, lockfile, Jupyter language version, README citation and `CITATION.cff` to **0.54.0**.

All additions are backward-compatible and need no new dependencies or runtime permissions.

## 0.53.0 — 2026-10-30

### Citable software and reproducible research
- Add canonical GitHub `CITATION.cff` and README citation instructions based on the author credit in `NOTICE`, with automated crate-version consistency checks ([#274](https://github.com/edujbarrios/semauri/pull/274)).

### More expressive text literals
- Accept `\\0`, `\\b`, and `\\f` escapes in both quote styles, preserving diagnostics and parsing semantics ([#275](https://github.com/edujbarrios/semauri/pull/275)).
- Accept literal-backslash raw strings using `r"..."`, `r'...'`, and uppercase `R` prefixes ([#276](https://github.com/edujbarrios/semauri/pull/276)).
- Highlight raw string scopes and expanded escapes in the VS Code TextMate grammar ([#277](https://github.com/edujbarrios/semauri/pull/277)).

Release 0.53.0 updates the Jupyter language identifier and CFF citation version. All changes are additive with no new dependencies or runtime permissions.

## 0.52.0 — 2026-10-30

### More expressive source literals
- Add case-insensitive binary (`0b`), octal (`0o`), and hexadecimal (`0x`) integer literals with underscore separators and checked signed 64-bit values ([#270](https://github.com/edujbarrios/semauri/pull/270)).
- Accept single-quoted UTF-8 strings with the same escapes as double-quoted strings; preserve the original quote style in formatting ([#271](https://github.com/edujbarrios/semauri/pull/271)).
- Update VS Code syntax highlighting and grammar tests for numeric bases and both quote styles ([#272](https://github.com/edujbarrios/semauri/pull/272)).
- Bump the Rust package to 0.52.0 and the experimental Jupyter language identifier to 0.52.

These are additive syntax changes. No new Rust dependencies or runtime capabilities are required.

## 0.51.1 — 2026-10-29

### Clearer positioning for humans and AI agents
- Lead the canonical README with **A natural-language-inspired programming language, built for humans and AI agents.** and align the crate description ([#267](https://github.com/edujbarrios/semauri/pull/267)).
- Preserve the precise contracts: statically typed programs, deterministic compiler behavior, and inspectable AI workflows.

### More reliable Jupyter diagnostics
- Propagate stdout-only compiler diagnostics (including failed `%%check --json` cells) as Jupyter errors instead of losing the diagnostic when stderr is empty ([#268](https://github.com/edujbarrios/semauri/pull/268)).
- Avoid duplicate stdout streams for stdout-only failures; retain prior stdout alongside stderr errors.
- Reject zero, negative, and non-finite notebook cell timeouts before launching the compiler, with regression tests.

This patch release adds no Rust dependencies, syntax changes, or runtime permissions. Existing `.suri` programs remain compatible.

## 0.51.0 — 2026-10-28

### Unicode text transformations
- Add `capitalize of STRING` ([#261](https://github.com/edujbarrios/semauri/pull/261)) to uppercase the first Unicode scalar while preserving the rest.
- Add `swapcase of STRING` ([#262](https://github.com/edujbarrios/semauri/pull/262)) to swap the case of cased Unicode scalars, preserving uncased characters.
- Add `uncapitalize of STRING` ([#264](https://github.com/edujbarrios/semauri/pull/264)) to lowercase only the first Unicode scalar, preserving all remaining text.
- Add `titlecase of STRING` ([#265](https://github.com/edujbarrios/semauri/pull/265)) to uppercase first scalars and lowercase remaining scalars in whitespace-delimited words, preserving whitespace.
- All four operations are contextual and pure, support typed HIR, constant folding, functions and list mapping, and reuse type diagnostic `S351`. Update the formatter, editor grammar, examples, docs and regression coverage.

### Jupyter notebooks and language tooling
- Add read-only `%%tokens`, `%%ast`, `%%hir`, `%%optimize`, `%%symbols`, `%%graph`, and `%%fmt` notebook directives, plus richer completions ([#263](https://github.com/edujbarrios/semauri/pull/263)).
- Keep notebook cells standalone and capability-restricted; inspections cannot write files, execute application effects or accept unsafe flags.
- Update the Jupyter language version to 0.51 and surface the new language operators in completions.
- Link the separate public grammar from the canonical guide and VS Code extension metadata ([#203](https://github.com/edujbarrios/semauri/pull/203)).

No new Rust dependencies or runtime permissions are introduced. Publication is gated on Rust compiler tests, Jupyter integration tests, six native distribution checks and the version-triggered release workflow.

## 0.50.0 — 2026-10-28

### Exact real cube roots
- Add pure contextual `cbrt of NUMBER` with signed real cube roots. Perfect integer cubes return exact i64 roots (including i64::MIN), while non-perfect integers and finite decimals return floats.
- Type diagnostic `S437` rejects non-numbers; `S438` rejects non-finite numeric inputs. Includes typed HIR, constant folding, evaluation, formatter, editor grammar, example and tests.

### Checked numeric reciprocals
- Add pure contextual `reciprocal of NUMBER` returning a floating-point inverse, with explicit rejection of zero (including negative zero), non-finite inputs, and non-finite results.
- Add located diagnostics `S439` for invalid operand types and `S440` for invalid numeric domains; extend parser, typed HIR, constant folding, evaluator, formatter, VS Code highlighting and regression coverage.

### Exact decimal digit counts
- Add pure contextual `digits of NUMBER` counting base-10 digits in integer magnitudes, including zero and i64::MIN, without rounding via f64 or counting the minus sign.
- Accept finite integral decimal values only within the signed i64 domain; report `S441` for non-number types and `S442` for fractional, non-finite or out-of-range numbers. Provide runnable example and regression coverage.

All three expressions compose with typed functions and list transformations, are supported by compiler inspection commands, preserve their words as normal identifiers unless followed by `of`, add no Rust dependencies or runtime capabilities, and retain source-located diagnostics.
## 0.49.0 — 2026-10-27

### Official experimental Jupyter kernel
- Add `integrations/jupyter/` as an installable Python/ipykernel bridge to the existing Rust `semauri` executable. Register the **Semauri** kernel in Jupyter Notebook or JupyterLab with `semauri-jupyter-install --user`.
- Execute isolated `.suri` notebook cells with source-located native Semauri diagnostics and a default permission restricted to `console_write`. Additional capabilities require an explicit `SEMAURI_ALLOW_EFFECTS` setting; temporary cell sources are removed after execution.
- Add `%%check`, `%%explain`, `%%explain --json`, `%%effects`, `%%plan`, keyword completion, a runnable `.ipynb` walkthrough and dedicated Jupyter protocol CI.
- Document notebook limitations: cells do not share bindings, because replay of earlier sources could duplicate side effects. True incremental sessions remain a future compiler-level feature.

### New typed numeric expressions
- Add contextual `cube of NUMBER` with exact, checked 64-bit integer multiplication, finite decimal results, static error `S433` and overflow/domain diagnostic `S434`.
- Add pure `prime of NUMBER` Boolean predicate using deterministic 64-bit Miller–Rabin arithmetic without dependencies. Non-number values report `S435`; invalid fractional, non-finite and out-of-range decimal values report `S436`.
- Provide typed-HIR, optimizer, execution, formatter, editor highlighting, examples and regression coverage including pseudoprimes and integer boundaries.

This release adds no new Rust dependencies and preserves the existing capability model and bounded collection limits.

## 0.48.0 — 2026-10-27

### Nearest-integer rounding
- Add pure contextual `round of NUMBER` with ties rounded away from zero, preserving exact i64 inputs and safely converting finite floating-point results without saturating overflow.
- Add statically located `S427` invalid-operand diagnostics and evaluation-time `S428` for non-finite values.
- Wire through parser, typed HIR, constant folder, evaluator, canonical formatter, VS Code grammar and regression tests.

### Truncation toward zero
- Add `truncate of NUMBER` to remove fractional parts toward zero (different from `floor of` for negatives), preserving integer precision and avoiding out-of-range integer casts.
- Introduce diagnostic `S429` for non-number operands and `S430` for non-finite values; add end-to-end tests and examples.

### Exact checked factorials
- Add `factorial of NUMBER` for non-negative integral inputs from 0 through 20; compute exact signed i64 results using checked arithmetic.
- Reject invalid operand types with `S431`, and negative/fractional/non-finite/overflow-range operands with `S432` rather than silently truncating or wrapping.
- Include typed function and list-transform support, static optimizer folding, source formatting, editor grammar support and runnable examples.

All new numeric operators are contextual: ordinary identifiers called `round`, `truncate`, or `factorial` remain compatible unless followed by `of`. No additional runtime permissions are granted.

## 0.47.0 — 2026-10-27

### Checked numeric squaring
- Add pure contextual `square of NUMBER`, preserving signed 64-bit integer precision and reporting `S422` on overflow or non-finite decimal results; static `S421` for invalid operands.
- Enable use in expressions, typed functions, list transformations, optimizer constant folding, CLI inspection, and VS Code highlighting, with tests and an executable example.

### Typed square roots
- Add `sqrt of NUMBER`, returning exact integer roots for perfect square integer operands (including large values near the 64-bit boundary), and floating-point results for non-perfect roots.
- Reject non-number operands with `S423` and negative/non-finite roots with `S424`, with shared semantics across runtime and optimizer, plus examples and tests.

### Boolean parity tests
- Add contextual `even of NUMBER` and `odd of NUMBER` returning `boolean`. Preserve exact parity for signed 64-bit integer operands, including negative and large integers.
- Accept integral finite decimals within the i64 range; reject fractional/out-of-range/non-finite operands with `S426` instead of performing lossy casts. Non-number inputs raise `S425`.
- Support functions, list transformations, optimizer folding, formatting, documentation, and editor highlighting.

All new operators are contextual (names stay valid variables when not followed by `of`) and require no additional execution capabilities.

## 0.46.0 — 2026-10-27

### Explain Semauri programs for people and coding agents
- Document the existing `semauri explain FILE` semantic trace prominently in the README and agent quick start; distinguish it from `semauri check FILE`, actual execution, effect inspection and runtime plans.
- Add a runnable `examples/explain_walkthrough.suri` with exact, numbered explanation output and integration coverage to catch stale documentation.
- Introduce `semauri explain FILE --json` with an ordered `explanations` array plus `ok` and `status`, letting IDEs, CI tools and agents consume semantic decisions without parsing numbered text.
- Return the established source-aware `diagnostics` JSON envelope on explanation failures; preserve the original human-readable mode and effect-free analysis.
- Add Rust integration tests for JSON/text parity, diagnostics, argument ordering and default output behavior.

## 0.45.0 — 2026-10-27

### Typed numeric sign classification
- Introduce pure, contextual `sign of NUMBER` returning exactly -1, 0 or 1 with integer precision (including the signed 64-bit minimum), finite decimal handling and signed-zero semantics.
- Add static `S417` operand checking, non-finite evaluation diagnostic `S418`, optimizer folding, formatter and VS Code highlighting support, examples, and regression tests.

### Contextual numeric floor and ceiling
- Introduce pure `floor of NUMBER` (toward negative infinity) and `ceiling of NUMBER` (toward positive infinity), with both operators statically returning `number`.
- Preserve exact integers across the full i64 range; round finite floats using IEEE-754 floor/ceiling and convert to integers only when safely in signed 64-bit bounds, without saturating out-of-range floats.
- Add static `S419` diagnostic for invalid operand types, `S420` for non-finite evaluation, and comprehensive compiler, optimizer, formatter, execution, example and VS Code coverage.
- Preserve identifier compatibility: `sign`, `floor` and `ceiling` become operators only when followed by `of`. No new capabilities or runtime effects are required.

## 0.44.0 — 2026-10-27

### Safer assertion-suite discovery
- Bound `semauri test PATH` recursion to 256 source files and 32 directory levels, rejecting excessive suites before executing tests.
- Exclude generated/cache directories and `.config.suri` from test discovery while preserving symlink isolation, deterministic file ordering and capability enforcement.
- Add end-to-end regression coverage for limits, cache exclusions and symlink boundaries.

### Machine-readable test reports
- Introduce `semauri test PATH --json` with `ok`, `status`, `passed`, `failed`, and per-file `tests` entries including error diagnostics on failure.
- Preserve text output by default and return JSON error envelopes for invalid paths and CLI usage when JSON output is requested.

### Focused test execution
- Add `semauri test PATH --filter TEXT` to select files by a case-sensitive substring of their relative path, normalizing separators across platforms.
- Support `--filter` together with `--json` and `--allow-console`, without introducing any new runtime effects or permissions.
- Add selection, JSON, empty-filter and single-file regression tests.

## 0.43.0 — 2026-10-26

### Project-wide verification and CI readiness
- Introduce `semauri verify PROJECT [--json] [--allow-console]`, an effect-aware local project quality gate for formatter conformance, static checking of all source files, entry-point effect inspection and assertion-test execution.
- Guarantee that project verification does not execute the application or filesystem/network effects; tests continue to use the restricted test runner and require explicit console permission for printing.
- Provide aggregated diagnostics and machine-readable reports with stable status codes and safety bounds, plus integration tests for invalid projects, formatting, static errors, unsafe test effects and generated project workflows.

### Safe recursive source formatting
- Extend `semauri fmt DIRECTORY --check|--write` to recursively check or normalize project sources in one command while preserving the existing single-file behavior.
- Preflight lexing and formatting of all discovered sources before any write, report all noncanonical file paths and summary counts, and refuse implicit directory output.
- Reuse bounded, symlink-safe project scanning that excludes build/cache directories; add integration tests for formatting, file preservation, and preflight failure behavior.

## 0.42.0 — 2026-10-26

### Practical project initialization
- Add `semauri init PROJECT` to create a non-overwriting multi-file project with typed library functions, an executable program, tests and English documentation.
- Introduce an explicit `.semauri-root` project marker so tests in subdirectories can safely import shared modules while all import targets remain confined to the marked root.
- Keep existing single-file programs and unmarked import restrictions backward compatible; add generated-project and safety regression tests.


### Secure local source imports
- Add `import "relative/file.suri".` for reusable typed multi-file programs, resolving dependencies before standard compilation and effect analysis.
- Reject import cycles, filesystem escapes including symlinks, imports after executable statements, invalid paths and oversized graphs (64 files, 32 levels, 4 MiB) using `S416`.
- Deduplicate canonical module paths, preserve compiler type/effect checking across files, and remap CLI diagnostics to original file locations.
- Support check, build, run, test and inspection commands; keep formatter handling contextual and update examples and regression coverage.

## 0.41.0 — 2026-10-26

### Unicode substring positions and rightmost search
- Add contextual `positions of NEEDLE in TEXT` returning a statically typed list of non-overlapping Unicode scalar match indices, bounded to 10,000 matches.
- Add `last position of NEEDLE in TEXT` returning the rightmost Unicode scalar match position, `-1` when absent, and the text length for an empty needle.
- Enforce two string operands with `S414` and reject empty or oversized all-position queries with `S415`, preserving deterministic optimizer/runtime behavior and contextual identifiers.
- Add editor grammar, formatter support, runnable examples, and regression tests.

## 0.40.0 — 2026-10-26

### Pure literal substring removal
- Add contextual `remove all of NEEDLE from TEXT` for non-overlapping, case-sensitive literal Unicode substring removal.
- Statically require string operands (`S412`) and reject empty search text (`S413`) without suppressing invalid values during optimization.
- Preserve pure deterministic behavior, formatting/editor support, runnable examples and integration tests.


### Direct Unicode character indexing
- Add `character at INDEX in TEXT` to return one Unicode scalar string using zero-based Unicode positions.
- Enforce static numeric/string operand typing (`S410`) and reject fractional, negative, and out-of-range indices (`S411`).
- Integrate evaluation and optimizer folding through one helper, contextual formatting, examples, and regression coverage.

## 0.39.0 — 2026-10-26

### Pure Unicode character and code point conversions
- Add contextual `characters of TEXT`, `codepoints of TEXT`, `codepoint of TEXT`, and `character from NUMBER` to inspect and construct Unicode scalar values.
- Enforce exact numeric scalar validity and a 10,000-item safety bound; report static `S408` and runtime `S409` diagnostics.
- Share one implementation across evaluator and optimizer, and update the formatter, editor grammar, docs, examples and tests.

## 0.38.0 — 2026-10-26

### RFC 6901 JSON pointer navigation
- Add `json at pointer POINTER in DOCUMENT` to select nested objects, arrays, and scalar values as serialized JSON text.
- Support RFC 6901 escaped keys, exact array indices, empty-root pointers, bounded size and depth, deterministic serialization, and strict `S406`/`S407` diagnostics.
- Compose with existing typed JSON decoding, add constant folding, formatter/editor support, executable examples, and regression coverage.


### Unicode-aware take and drop on strings
- Extend `take COUNT from COLLECTION` / `drop COUNT from COLLECTION` to immutable strings, counting Unicode scalar values rather than UTF-8 bytes.
- Preserve typed-list compatibility, source-aware errors for invalid counts, constant folding, and pure evaluation.
- Add runnable examples and regression tests for Unicode, boundaries, and nested selections.

## 0.37.0 — 2026-10-26

### Unicode-aware text searching
- Add `occurrences of NEEDLE in TEXT` for non-overlapping substring counts and `position of NEEDLE in TEXT` for the zero-based Unicode scalar position of the first match.
- Return `-1` for missing terms, `0` for the position of an empty term; reject empty occurrence terms with `S403` and incorrect types with `S402`.
- Provide constant folding, formatter/editor support, examples and integration tests.


### Type-directed JSON field extraction
- Add `json TYPE at KEY in DOCUMENT` for six checked scalar or homogeneous-list field types, including strings, numbers and booleans.
- Reject missing keys, JSON null, type mismatches, overflowing integer text and excessive input with located `S400`/`S401` diagnostics.
- Preserve contextual syntax, constant-fold valid scalar fields, and update examples, editor grammar and tests.

## 0.36.0 — 2026-10-26

### JSON object introspection
- Add pure `keys of json DOCUMENT` and `has json key KEY in DOCUMENT` expressions with sorted field names and exact membership semantics.
- Enforce strict string inputs, bounded JSON objects, source-aware diagnostics, constant folding, and editor/formatter support.
- Preserve JSON `null` as a present key and return an empty list for an empty object.


### Type-directed JSON decoding
- Add `decode json as TYPE from TEXT` for string, number, boolean, and homogeneous string/number/boolean lists.
- Validate JSON syntax, strict type correspondence, bounded input and list expansion, integer exactness, and effect-free evaluation, reporting diagnostics `S396`/`S397`.
- Add optimizer folding, formatter recognition, editor grammar, a runnable example, and integration tests.

## 0.35.0 — 2026-10-26

### Safe typed conversions from text
- Add contextual `parse number from TEXT` and `parse boolean from TEXT` expressions with verified types and deterministic parsing.
- Reject non-string operands (`S394`), malformed/non-finite values and oversized integers (`S395`), with source-aware diagnostics.
- Share text conversion between optimizer and evaluator and extend formatting, editor grammar, examples and regression coverage.


### Typed text splitting and joining
- Add contextual `split of TEXT by DELIMITER` and `join of LIST with DELIMITER`, with strict operand checking, Unicode support, and literal separators.
- Keep empty fields, reject empty split delimiters and excessive expansion with explicit source-aware diagnostics, and keep existing ML-domain `split` action compatibility while preserving `join` as an identifier.
- Add runtime and optimizer support, formatter and editor grammar updates, examples and regression coverage.

## 0.34.0 — 2026-10-26

### Natural-language string predicates
- Add contextual `starts with` and `ends with` to validate Unicode string prefixes and suffixes with checked operands and pure boolean results.
- Support constant folding, editor formatting, highlighted syntax, and end-to-end diagnostics and examples.

### Typed JSON serialization
- Add contextual `json of` with static type checks, deterministic JSON encoding, string escaping, and compile-time folding for literals.
- Document integration payloads and preserve `json` as an ordinary variable name.

### Natural-language text processing
- Add contextual `trim of`, `words of`, and `lines of` with Unicode-aware semantics, typed results, and constant folding.
- Preserve existing variable names and add regression tests, examples, formatter coverage, and VS Code highlighting.

## 0.33.0 — 2026-10-26

### Executable configuration and local AI runtime
- Add typed `string config`, `number config`, and `boolean config` runtime reads with an explicit `config_read` capability.
- Make `semauri run` discover the nearest `.config.suri`; add explicit `--config` selection and declared `--set` overrides. Bindings resolve individually so unused environment secrets are neither required nor loaded.
- Execute config, AI, and console operations as one typed dependency plan. Add a deterministic `local/*` adapter for offline and CI workflows.

### Safe AI execution controls
- Add typed request budgets, provider retry policy, and run-scoped response caching. Enforce non-negative integer limits and reject uncached inference after budget exhaustion.
- Make every checked-in AI example configuration-driven and executable, including a self-contained vision fixture.
- Document the supported runtime boundary and the prioritized capability roadmap without claiming hosted providers, streaming, structured extraction, or persistent agent state.

## 0.32.0 — 2026-10-26

### Agent-first project guide
- Remove the separate `docs/` tree and make the root README the canonical, compact language and operational guide for humans and coding agents.
- Repair the bilingual course and contribution links so the repository remains self-contained.

### Typed project configuration
- Add `.config.suri` with typed JSON parameters and lazy environment bindings.
- Add `semauri config [PATH]`, declared `--set NAME=JSON_VALUE` overrides, and `--check-env`. Configuration summaries never expose environment values.

### General AI primitives
- Add provider-neutral, typed primitives for language models, vision models, generation, image inspection, embeddings, tool schemas, bounded agents, and agent runs.
- Preserve distinct LLM/VLM nominal types and explicit effects for model loading, network access, inference, filesystem reads, and tool execution.
- Add two runnable `.suri` examples and focused compiler contracts. These primitives produce plans; provider execution remains the responsibility of an adapter.

## 0.31.0 — 2026-10-26

### Canonical source extension
- Replace the conflicting `.sema` suffix with `.suri`, a short Semauri-specific source extension not assigned in the IANA media-type registry or the current GitHub Linguist extension catalog. The IANA registry already contains the unrelated `application/vnd.sema` media type.
- Update the CLI test runner, diagnostics, all 43 runnable examples, all 22 bilingual course programs, test fixtures, package smoke tests, guides and documentation links.
- Treat this as a breaking filename migration: rename existing source files from `name.sema` to `name.suri`.

## 0.30.0 — 2026-10-26

### Language: exact contextual absolute values
- Add pure `absolute of NUMBER` with complete parser/AST/HIR/evaluator/formatter/optimizer support. Preserve exact signed i64 arithmetic above 2^53, avoiding f64 rounding; S386 type-checks operands and S388 diagnoses overflowing i64 minimum or non-finite results. The name `absolute` remains a valid identifier outside contextual `absolute of` syntax. Add six tests, runnable example and docs. [PR #188](https://github.com/edujbarrios/semauri/pull/188).

### Language: checked product of typed numeric lists
- Add pure `product of LIST` returning `number` for `list<number>`: exact checked integer multiplication with S387 on overflow and binary64 semantics for mixed/decimal lists, rejecting non-finite values. Typed empty numeric lists return one (multiplicative identity). Type errors reuse S353, and existing `product` variable identifiers remain compatible. Add eight integration tests, documentation and a runnable example. [PR #189](https://github.com/edujbarrios/semauri/pull/189).

### Boundaries
- No arbitrary runtime streaming datasets, tensor execution or neural network training are implemented; these expressions operate on concrete compiler-known values and grant no effects.

## 0.29.0 — 2026-10-25

### Language: typed pairwise zip
- Add pure `zip every LEFT in LIST1 with RIGHT in LIST2 to BODY`. Typed independent lexical iterators, static inferred result element type, shorter-length output, unchanged source inputs, and S383/S384 diagnostics. [PR #184](https://github.com/edujbarrios/semauri/pull/184).

### Language: stable predicate partition
- Add `partition every ITEM in LIST if PREDICATE` yielding `list<list<T>>` with true and false buckets in stable order, immutable inputs and typed empty results. Type checks reuse S365/S366. [PR #185](https://github.com/edujbarrios/semauri/pull/185).

### Language: typed one-level flattening
- Add contextual `flatten of`, turning `list<list<T>>` into `list<T>` by one-level concatenation, with preserved order and element types. Invalid operands receive located S385. Identifiers named `flatten` remain valid. [PR #186](https://github.com/edujbarrios/semauri/pull/186).

### Validation and boundaries
- Add 21 focused Rust integration tests across the three operators, runnable examples and semantics; native Windows/macOS/Linux builds for ARM64 and x86_64.
- Operations apply only to compiler-known core list values, not tensor streaming, model training or arbitrary general runtime control flow. No additional capabilities.

## 0.28.0 — 2026-10-25

### Language: typed seeded list reductions
- Add `reduce every ITEM in LIST starting ACC as SEED to BODY`: a pure, typed, lexically scoped single-pass fold over concrete lists. Supports numeric, string, Boolean and list accumulators, empty-list seed semantics and pure user-function calls. Type-check input and step (S379/S380), reject impure steps (S381), and emit S250 for malformed syntax. [PR #181](https://github.com/edujbarrios/semauri/pull/181).
- Add AST/HIR, optimizer, evaluator, effect-analysis and formatter integration; integration tests and a runnable example.

### Language: scoped short-circuit Boolean quantifiers
- Add `any every ITEM in LIST if PREDICATE` and `all every ITEM in LIST if PREDICATE`, reusing typed scoped predicate validation and early-exit evaluator paths. `any` over an empty typed list returns false; `all` returns true. Existing `any of`/`all of` remain supported. [PR #182](https://github.com/edujbarrios/semauri/pull/182).
- Add tests for short-circuit behavior, empty inputs, type errors, pure functions and compiler tools.

### Boundaries
Pure compiler-known list transformations only. No arbitrary runtime iteration, streaming ML dataset reductions or neural model training backend.

## 0.27.0 — 2026-10-25

### Language: statically typed lazy conditional values

- Add `if CONDITION then CONSEQUENCE otherwise ALTERNATIVE` as a pure, typed expression distinct from existing conditional statements, with boolean condition validation (`S375`), exact branch type matching (`S376`), and lazy evaluation of only the chosen branch. Static AST/HIR, optimizer, effects and symbol analysis support; compose with functions and typed map/filter. [PR #178](https://github.com/edujbarrios/semauri/pull/178).

### Language: bounded typed list repetition

- Add `repeat COUNT of VALUE` to create a fresh `list<T>` by evaluating one value and cloning it COUNT times. Counts must be integral and between 0–10,000 inclusive, with support for typed empty/nested lists, an `S377` static numeric-count error and an `S378` invalid count error. Contextual `repeat` avoids breaking existing identifiers. [PR #179](https://github.com/edujbarrios/semauri/pull/179).
- Cover both features with end-to-end Rust tests, runnable examples, documentation and compiler API checks. The formatter recognizes contextual delimiters without requiring new globally reserved words.

### Scope

Both transformations operate on compiler-known values, are deterministic and pure, and do not add general runtime-dependent branching, recursion, external tensor execution or live ML model training. No new effects or permissions.

## 0.26.0 — 2026-10-25

### Language: multi-parameter pure typed functions

- Expand user-defined pure functions to accept **1–16** explicitly typed parameters, retaining inferred return types, immutability, lexical scopes, and compatibility with v0.25.0 one-parameter invocations. Accept `define function weighted taking score as number, weight as number, bias as number to ((score times weight) plus bias).` and `invoke weighted to (3, 2, 5)`. [PR #175](https://github.com/edujbarrios/semauri/pull/175).
- Compile statically checked argument tuples to typed AST/HIR call nodes and evaluate arguments once left to right before binding parameter values in a fresh lexical environment. Add the source-located `S374` wrong-arity diagnostic, preserve `S370` for parameter type errors and `S311` for duplicate parameter names. Integrate formatter, optimizer, effects and regression tests.
- Supports nested/list types and use within `map every`, `filter every` and `count every`. No recursion, first-class function values or effectful user-function bodies.

### Correctness: precise minimum/maximum on mixed numeric lists

- Reuse the exact transitive mixed `i64`/`f64` comparator already shared by sorting and median in `minimum of` and `maximum of`. Prevent silent wrong selections for large integers around and above `2^53` and binary64 decimal neighbors. [PR #176](https://github.com/edujbarrios/semauri/pull/176).
- Add regression tests for mixed sign, integer boundaries and ordering independence, plus docs for numeric precision and existing finite-number requirements.

### Boundaries

All operations currently apply to compiler-known values. Semauri does not execute streaming ML tensors, model training or arbitrary runtime-dependent general-purpose functions. No new effect permissions.

## 0.25.0 — 2026-10-25

### Language: typed pure user-defined functions

- Add `define function NAME taking PARAM as TYPE to EXPRESSION.` and `invoke NAME to PRIMARY` with explicit parameter type, inferred return type, lexical shadowing, previously defined function invocation and composition with typed `map every`/`filter every` pipelines. New AST/HIR function call nodes, evaluator frames, formatter, optimizer and effect-analysis traversal. [PR #172](https://github.com/edujbarrios/semauri/pull/172).
- Types include primitive values and recursively nested `list of TYPE` parameters. Parameter counts are currently limited to one. Calls to unknown functions fail with `S369`, type mismatches with `S370`, duplicate declarations with `S371`, and invalid scope/effectful bodies with `S372`. Parser syntax errors use `S246`/`S247`.
- Function invocation uses contextual `invoke`, avoiding collision with the existing ML `apply lora` domain action. `define` and `function` remain contextual. Add regression tests, runnable programs, examples, and reference documentation.

### Language: typed predicate counting

- Add pure `count every ELEMENT in LIST if PREDICATE` to return an exact `number` match count without allocating a filtered list, with lexical iterator bindings, checked boolean predicate typing, scoped evaluation and source-located S373 overflow diagnostic. Contextual `count` preserves earlier variable names. [PR #173](https://github.com/edujbarrios/semauri/pull/173).
- Seven focused regression tests for empty lists, pure user-function integration, scope, formatter/HIR/optimizer/effect analysis and wrong-typed operands, plus a runnable example and semantic specification.

### Current boundaries

- Functions are pure, top-level, declared before use, and evaluated only on compiler-known values. Recursion, forward references, first-class functions, multi-argument definitions and general runtime ML execution are **not** supported. No new effect grants or ML training framework integrations.

## 0.24.0 — 2026-10-25

### Language: scoped typed map/filter pipelines
- Add contextual `map every x in LIST to EXPRESSION` and `filter every x in LIST if PREDICATE`, using lexical iterator symbols, source-located typed errors S365/S366, a new AST/HIR node, per-item pure evaluation and optimizer/effect analysis traversal. Preserve existing variable spellings `map` and `filter`. Mapped element type is inferred statically; filters retain original list type and return typed empties if nothing passes. [PR #169](https://github.com/edujbarrios/semauri/pull/169).
- Cover immutability, lexical shadowing, outer references, element-type conversion, nesting/composition, contextual formatting and diagnostics in regression tests.

### Language: bounded inclusive integer generators
- Add contextual `range from START to END` producing `list<number>`, ascending or descending, with inclusive endpoints and exact i64 values. Invalid operand types raise S367; non-integer bounds and ranges exceeding 10,000 values raise S368 without overflow or unbounded compile-time allocation. [PR #170](https://github.com/edujbarrios/semauri/pull/170).
- Add runnable `map_filter.suri` and `range_map_filter.suri` examples plus detailed reference documentation and toolchain packaging verification.

### Runtime and compatibility boundaries
- These operations evaluate concrete compiler-known typed lists; they do **not** implement arbitrary user-defined functions, runtime streaming transforms, external ML tensors, or neural-network training. No new effect permissions were introduced. The words `map`, `filter`, and `range` remain valid variable identifiers outside the contextual operator syntax.

## 0.23.0 — 2026-10-25

### Language: typed boundary access
- Implement pure `first of` and `last of` expressions with `list<T> -> T` static result types, preserving immutability. Source-located type errors `S361`; empty-list evaluation errors `S362`. Preserve legacy variable names `first` and `last` through contextual parsing and formatting, including user labels. [PR #166](https://github.com/edujbarrios/semauri/pull/166).

### Language: typed immutable list slicing
- Add `take COUNT from LIST` and `drop COUNT from LIST`, returning a new typed `list<T>`. Zero and over-length counts clamp naturally, enabling safe typed empty-list results, including nested lists. Wrong operand types receive `S363`; negative or fractional values receive `S364` at evaluation. [PR #167](https://github.com/edujbarrios/semauri/pull/167).
- Add runnable examples, integration tests for composition, preserved input order/type, nested lists, invalid counts, backward compatibility, formatting and effect analysis. `take`, `drop`, and `from` are newly reserved words. `first` and `last` are contextual and remain valid existing identifiers without `of`.

### Scope
- All new operators work with concrete compiler-known lists and do not grant effects, execute external dataset streams, or train ML models. User-defined typed functions, map/filter lambdas, and general runtime-dependent iteration are still future roadmap items.

## 0.22.0 — 2026-10-25

### Language: immutable typed sorting
- Add `sorted of` for concrete homogeneous number, string, or Boolean lists, producing a new list without mutation and retaining element type. Sorting is stable; strings use case-sensitive Unicode scalar order and booleans sort false-first. Non-sortable inputs or non-finite numeric items raise `S359`. [PR #163](https://github.com/edujbarrios/semauri/pull/163).
- Improve the shared numerical comparator for integers and decimals so mixed `i64`/`f64` sorting remains precise and transitive beyond `2^53`. The `median of` operator now uses the same comparator, fixing mixed-number median ordering.

### Language: stable typed deduplication
- Add `distinct of` on concrete homogeneous `list<T>` values. Keep first occurrences, preserve input order, and use existing list-membership equality (including distinguishing stored integer and decimal representations); non-list operands receive located `S360`. [PR #164](https://github.com/edujbarrios/semauri/pull/164).
- Add eleven focused integration tests across the two operations, runnable examples, language reference updates, formatter and effect-analysis checks. `sorted` and `distinct` are newly reserved words.

### Boundaries
- All new operators are pure transformations on compiler-known lists, not streams, external ML datasets or tensors; no new effect permissions or ML training execution.

## 0.21.0 — 2026-10-23

### Language: typed Boolean reductions
- Add `any of` and `all of` on `list<boolean>` with static type checking (`S356`), no new effects, integration tests, and a runnable example. `any` and `all` are newly reserved keywords. [PR #160](https://github.com/edujbarrios/semauri/pull/160).

### Language: typed numeric median
- Add `median of` for finite `list<number>` with checked type constraints (`S353`) and guarded empty/non-finite evaluation (`S357`). Exact integral medians and even integer midpoints preserve i64 precision using i128 intermediate sums; decimals follow f64 rounding. `median` is newly reserved. [PR #161](https://github.com/edujbarrios/semauri/pull/161).
- Fix existing unary `minus` integer precision loss in both execution and constant folding; preserve exact i64 values and diagnose signed negation overflow (`S358`).
- Include runnable examples, integration/regression tests, updated language reference, and release packaging documentation.

### Boundaries
- These operations are pure and work on concrete core-language lists, not tensors or runtime ML dataset handles. This release does not introduce model-training execution.
- Existing mixed/uppercase language syntax remains compatible, apart from the newly reserved words.

## 0.20.0 — 2026-10-23

### Typed numeric extrema
- Add `minimum of` and `maximum of` for `list<number>`, with static `S353` type errors, empty/non-finite `S355` evaluation diagnostics and no new effects. Exact integer-only comparisons preserve `i64` extrema beyond 2^53; mixed decimal/integer comparisons follow current `f64` semantics. [PR #157](https://github.com/edujbarrios/semauri/pull/157).
- Add integration tests for type checking, fractional and negative numbers, precision, formatting, composition, and effect analysis; include a runnable example and language specification.

### AI engineering reproducibility and portfolio evidence
- Add a documented ML compiler case study and a CI contract against the checked-in versioned ML example, covering deterministic typed dependency graphs, declaration audits and capability enforcement. [PR #158](https://github.com/edujbarrios/semauri/pull/158).
- Clarify that ML workflow planning, revision declarations, effect analysis and reproducibility preflight do not implement model training or prove external artifact integrity.

### Compatibility
- The new reserved words `minimum` and `maximum` cannot be used as bare variable identifiers. Other existing language constructs and case-insensitive command parsing remain supported.
- Native builds continue to target six Linux/macOS/Windows platform architectures and include SHA256 checksums on release.

## 0.19.0 — 2026-10-22

### Typed sequence and data processing

- Add `reverse of` for concrete strings and immutable homogeneous lists, maintaining list element types and leaving original inputs unchanged. Literal strings can be reversed during constant folding; string reversal operates on Unicode scalar values.
- Add `sum of` and `average of` for `list<number>`, with static type checking (`S353`), exact checked integer sums, decimal arithmetic for averages, and located numeric overflow/non-finite errors (`S354`).
- Introduce explicit reverse type diagnostic `S352`, integrate new keywords into the canonical lowercase formatter, and add integration tests and runnable examples for both features.
- Refresh the README, language grammar and documentation navigation to include new collection operations and their precision limits.

### Compatibility and limitations

- Existing uppercase/mixed-case Semauri programs remain accepted. `reverse`, `sum` and `average` are newly reserved language keywords; use different names for user identifiers that previously used those words.
- Numeric aggregation operates on concrete typed lists; the reference compiler does not compute over runtime-produced ML tensors or datasets.
- No new effects, external dependencies or framework execution capabilities are introduced.

## 0.18.0 — 2026-10-22

### Language

- Add typed `uppercase of` and `lowercase of` string expressions, with deterministic Unicode case mapping, compile-time type checking (`S351`), constant folding, runtime lowering and no additional effect capabilities.
- Support new string operations in the canonical lowercase source formatter, examples and regression tests.

### Getting started and documentation

- Present Semauri prominently as a statically typed programming language with an independent Rust compiler, native CLI and domain-specific build/execution backends.
- Refresh the primary project messages: **“A programming language for readable, typed and predictable software”** and **“Write clearly. Compile confidently. Build with intent.”**
- Add OS-specific installation and first-program instructions, including how to open PowerShell/Terminal, create and edit a `.suri` file, check and run it, resolve `PATH` issues and understand explicit capabilities.
- Expand the language reference and installation troubleshooting without changing the English-language Semauri syntax or the existing Spanish course.

### Compatibility

- Existing programs remain valid, including uppercase and mixed-case source commands. No new runtime permissions or dependencies are required.
- This release does not change the distinction between executable supported core operations and declarative ML workflow plans.

## 0.17.0 — 2026-10-22

### Added: lowercase language workflow

- The canonical spelling of all Semauri source commands and registered domain actions is lowercase; parsing remains case-insensitive for backward compatibility.
- `semauri fmt FILE` prints canonical lowercase source, `--write` rewrites it, and `--check` returns a nonzero exit status for formatting drift.
- Converted all 19 official examples and 22 beginner-course programs to lowercase language syntax, with automated conformance checks; documented code samples in both course languages now follow the style.

### Added: typed language features

- Pure Boolean `assert` statements, source-aware runtime assertion failures (`S502`), and the `semauri test PATH` command with strict effect controls.
- `length of` expressions for Unicode strings and typed lists, typed `contains` substring/list-membership comparisons, zero-based `list at index` access, and immutable same-type list concatenation with `plus`.
- Machine-readable `check --json` diagnostics while retaining human-readable defaults and existing inspector JSON outputs.

### Added: ML compiler and analysis infrastructure

- Validated typed SSA dataflow graphs via `semauri graph FILE` and `Compiler::runtime_graph`; graphs describe ML dependencies and reject invalid producer references.
- Version-qualified dataset/model input declarations: `source dataset ... revision ...` and `resolve model ... revision ...`.
- `semauri audit FILE [--strict]` for advisory ML provenance/seed checks; strict mode can reject incomplete declarations in CI.

### Fixed and improved

- File-only deletion semantics in native filesystem execution; recursive deletion of directories is refused.
- Preflight validation of filesystem operation arguments and names.
- Regression tests and CI gates for shipped examples, learning programs, language operations, ML graph integrity and reproducibility declarations.

### Compatibility and limitations

- Previously valid capitalized `.suri` programs still compile. The canonical lowercase convention is a source-style change, not a breaking grammar migration.
- Quoted strings, user-defined names and labels, comments, Rust APIs, generated output case and JSON keys are not blanket-lowercased.
- ML revisions are *declared*, not artifact-verified. Semauri still produces ML workflow plans rather than executing model training.
- User-defined functions and general dynamic runtime control flow are not part of this release.

# Changelog

All notable changes to Semauri are documented here.

## 0.16.1 — 2026-10-12

### Changed

- **Learn to Code is now bilingual:** opening `learn_to_code/` displays a language selector linking to complete `es/` and `en/` courses.
- Moved all 11 original Spanish lessons to `learn_to_code/es/` while keeping their programs and exercises.
- Added 11 equivalent English lessons under `learn_to_code/en/`, each with a translated guide, challenge and working Semauri program.
- Updated documentation, installation instructions, packaging smoke tests and learning-course regression tests for both languages.
- Release packages now include both language trees on Linux, macOS and Windows.

## 0.16.0 — 2026-10-12

### Added

- An 11-lesson beginner course in Spanish, with runnable `.suri` programs and challenges.
- `Unless`, `Otherwise if` / `Else if`, and Unicode scalar string escapes.
- Beginner course packaged with native compiler distributions.

## 0.15.0 — 2026-10-08

### Added

- `//` single-line comments and nestable `/* ... */` multi-line comments.
- Readable numeric literals with `_` digit separators and scientific `e` / `E` notation.

### Fixed

- Unterminated block comments produce a lexical diagnostic (`S105`).
- Invalid numeric separators, malformed exponents, integers outside the signed 64-bit range, and non-finite numeric literals produce `S106` rather than crashing the compiler.
- Comment delimiters inside strings remain ordinary text.

### Validation

- Language contract regressions added for both features.
- GitHub Actions checks Rust tests and builds/smoke-tests six native distribution packages.

## 0.14.0 — 2026-10-07

### Changed

- The reference compiler and CLI are implemented entirely in Rust.
- Semantic-domain registration and backend registration are public Rust extension APIs.
- External declarative domains can define artifacts, elements, typed properties, nominal types and deterministic operations without parser-specific cases.
- Runtime plans, effects/capabilities, optimization, filesystem execution and ML planning remain available through the native compiler.
- Official packaging builds one native executable for Linux, macOS and Windows on x86_64 and arm64.
- The source tree, tests, documentation and distribution pipeline are Rust-only.

### Validation

- Language, HIR, optimizer, runtime, schema, filesystem, ML and extension contracts are covered by Rust tests.
- CI checks the Rust-only repository invariant.
- Release artifacts are built and smoke-tested natively on all six supported target/architecture combinations.

## Earlier releases

Earlier milestones remain summarized in this changelog and in Git history.
