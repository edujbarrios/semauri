#!/usr/bin/env python3
"""Compare old and new Semauri executables on identical workloads and hardware.

Each measured invocation performs startup, parse, type checking and evaluation.
This is a paired end-to-end benchmark, not an isolated instruction-level
profile. Alternate execution order to reduce warming and scheduling bias.
"""
import argparse
from pathlib import Path
import statistics
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parent.parent
CASES = {
    "distinct_unique": (
        "let values be range from 1 to 10000.\n"
        "print cardinality of values.\n",
        "10000\n",
    ),
    "distinct_repeated": (
        "let values be range from 1 to 10000.\n"
        "let residues be map every n in values to (n modulo 100).\n"
        "print count distinct of residues.\n",
        "100\n",
    ),
    "map_reduce": (
        (ROOT / "benches" / "map_reduce.suri").read_text(),
        "333383335000\n",
    ),
    "text": (
        (ROOT / "benches" / "text.suri").read_text(),
        "10000\n",
    ),
}


def sample(binary: str, program: Path, expected: str) -> float:
    begin = time.perf_counter_ns()
    result = subprocess.run(
        [binary, "run", "--allow", "console_write", str(program)],
        capture_output=True, text=True, check=False,
    )
    duration_ms = (time.perf_counter_ns() - begin) / 1_000_000
    if result.returncode or result.stdout != expected or result.stderr:
        raise RuntimeError(
            f"{binary}: exit={result.returncode}, "
            f"stdout={result.stdout!r}, stderr={result.stderr!r}, "
            f"expected={expected!r}"
        )
    return duration_ms


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("before")
    parser.add_argument("after")
    parser.add_argument("--runs", type=int, default=30)
    parser.add_argument("--warmups", type=int, default=3)
    args = parser.parse_args()
    if args.runs < 1 or args.warmups < 0:
        parser.error("runs must be positive and warmups nonnegative")

    print(f"Paired CLI timings: {args.runs} samples, {args.warmups} warmups; "
          "process startup + compilation + execution included.", flush=True)
    print("| Workload | Base median (ms) | Candidate median (ms) | Speedup |")
    print("|---|---:|---:|---:|")
    with tempfile.TemporaryDirectory(prefix="semauri-perf-") as directory:
        for name, (program, expected) in CASES.items():
            path = Path(directory) / f"{name}.suri"
            path.write_text(program, encoding="utf-8")
            values = [[], []]
            binaries = (args.before, args.after)
            for iteration in range(args.warmups + args.runs):
                order = (0, 1) if iteration % 2 == 0 else (1, 0)
                for side in order:
                    duration = sample(binaries[side], path, expected)
                    if iteration >= args.warmups:
                        values[side].append(duration)
            old, new = (statistics.median(row) for row in values)
            print(f"| {name} | {old:.3f} | {new:.3f} | {old/new:.2f}x |", flush=True)


if __name__ == "__main__":
    main()
