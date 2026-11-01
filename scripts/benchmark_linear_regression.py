#!/usr/bin/env python3
"""Reproducible wall-clock measurement of the Semauri OLS demo.

Measures a complete released-mode semauri CLI process (startup, parse,
type-check, execution, stdout, exit), not cargo compilation. Timing on
shared CI runners is illustrative, not a hardware-independent speed claim.
"""
import argparse
import os
from pathlib import Path
import platform
import statistics
import subprocess
import time

EXPECTED = (
    "Linear regression (ordinary least squares)\n"
    "Training observations:\n5\n"
    "Learned slope:\n2.0\n"
    "Learned intercept:\n1\n"
    "Prediction for x=6:\n13\n"
)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", default="target/release/semauri")
    parser.add_argument("--source", default="examples/linear_regression.suri")
    parser.add_argument("--samples", type=int, default=101)
    args = parser.parse_args()
    if args.samples < 3:
        parser.error("--samples must be at least 3")
    binary = str(Path(args.binary).resolve())
    source = str(Path(args.source).resolve())
    command = [binary, "run", "--allow", "console_write", source]

    def checked_run() -> tuple[str, int]:
        start = time.perf_counter_ns()
        result = subprocess.run(command, capture_output=True, text=True, check=False)
        elapsed = time.perf_counter_ns() - start
        if result.returncode:
            raise RuntimeError(
                f"Semauri exited {result.returncode}: {result.stderr.strip()}"
            )
        if result.stdout != EXPECTED:
            raise AssertionError(
                f"Output differs from documented result: {result.stdout!r}"
            )
        if result.stderr:
            raise AssertionError(f"Unexpected diagnostic: {result.stderr}")
        return result.stdout, elapsed

    # Separate cold-start verification and warmup from the reported sample.
    output, first_ns = checked_run()
    for _ in range(3):
        checked_run()

    timings_ms = [checked_run()[1] / 1_000_000 for _ in range(args.samples)]
    median_ms = statistics.median(timings_ms)
    print("$ semauri run --allow console_write examples/linear_regression.suri")
    print(output, end="")
    print(f"First execution (wall): {first_ns / 1_000_000:.3f} ms")
    print(f"Median CLI execution (wall): {median_ms:.3f} ms")
    print(f"Measurement: {args.samples} runs, 3 warmups, release build, build excluded")
    print(f"Machine: {platform.system()} {platform.machine()}")

    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with open(summary, "a", encoding="utf-8") as result:
            result.write("## Semauri: linear regression from scratch\n\n")
            result.write("**Actual program output**\n\n```text\n")
            result.write(output)
            result.write("```\n\n")
            result.write(
                f"**Measured median CLI wall time:** {median_ms:.3f} ms "
                f"({args.samples} runs, 3 warmups; release build, build excluded; "
                f"{platform.system()} {platform.machine()}).\n\n"
            )
            result.write(
                "**Scope:** process startup + parse/type-check + program execution; "
                "shared runner, not a universal speed guarantee.\n"
            )


if __name__ == "__main__":
    main()
