# Semauri compiler performance baseline

The optimization series starts from **v0.48.0**, commit on `main` before
the performance branch. Five matching Semauri and Python programs live in
`benches/`. They exercise real process startup, loading, parsing, typing,
compile-time evaluation and execution, not a synthetic inner loop.

Run `cargo build --release` and then `bash scripts/bench.sh --runs 50
--warmups 3 --json benches/latest.json`. The runner verifies **byte-identical
stdout** before timing; a mismatch aborts measurement. Each benchmark has
50 measured independent processes per language after 3 warmups, and reports
median/min wall-clock milliseconds. CPU, host and Python version are recorded.

## User-supplied reference measurements (different machine; NOT comparable)

| Benchmark | Semauri reference median | Python reference median |
| --- | ---: | ---: |
| hello | 1.8 ms | 15 ms |
| map_reduce | 36.5 ms | 18.5 ms |
| filter_count | 15.4 ms | not supplied |
| sort | 8.0 ms | not supplied |
| text | 21.3 ms | not supplied |

These numbers were supplied in the performance brief. They have **not**
been remeasured on this execution environment; they must not be used
as controlled before/after speedups. For a credible comparison, keep the
same CPU, runner, release build and benchmark corpus before and after.

## CI measurement

The performance workflow records the runner CPU and prints the full
50-run table for each push to `perf/**`. Reports are also uploaded as
artifacts. Hosted runner CPU types can differ between jobs, so the results
must disclose hardware and should be treated as indicative rather than
strict A/B comparisons unless the CPU model matches.

`cargo test --all-targets` is the behavioral regression gate. The
benchmarks must maintain exact Python stdout, and the explain text and JSON
regressions are tested separately.
