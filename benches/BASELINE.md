# Performance baseline — Semauri 0.48.0

This folder provides five paired end-to-end workloads and deterministic
expected stdout. Before optimizing, build `cargo build --release` and run
`bash scripts/bench.sh 50`. The harness performs 3 warmups and records
50 wall-clock timings per workload and language (median and minimum).
It fails if Semauri and Python do not print the exact expected output.
No Semauri syntax, error message, security limit, or execution behavior
is changed by this harness.

## Prior independent measurements (provided by the project owner)

These are **not reproduced on the CI runner**, and are not directly
comparable with CI timings until measured there:

| benchmark | Semauri prior median | Python prior median |
|---|---:|---:|
| hello | 1.8 ms | 15 ms |
| map_reduce | 36.5 ms | 18.5 ms |
| filter_count | 15.4 ms | not provided |
| sort | 8.0 ms | not provided |
| text | 21.3 ms | not provided |

## Same-machine baseline and comparison

`scripts/bench_driver.py` emits machine/CPU, case-by-case medians and
minimums, and can save JSON with `BENCH_RESULTS=path.json`.
The performance CI job compiles the pinned `v0.48.0` revision and
the candidate revision **on the same hosted runner**, runs the harness
separately with both binaries, and prints a before/after/speedup table.
This avoids reporting speedups between different CPUs. Always record the
GitHub Actions run URL and CPU when copying these numbers into a PR.

```sh
cargo build --release
bash scripts/bench.sh 50
```

The harness intentionally includes process startup and parsing, not just
the compiler's compute path. For microarchitectural attribution use
`perf` or `valgrind --tool=callgrind` separately. Do not label hot
functions as measured without a matching profiling run.
