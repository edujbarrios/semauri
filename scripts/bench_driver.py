#!/usr/bin/env python3
"""End-to-end Semauri/Python timing harness; same stdout is required."""
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent.parent
CASES = ("hello", "map_reduce", "filter_count", "sort", "text")
EXPECTED = {
    "hello": "Hello, Semauri!\n",
    "map_reduce": "333383335000\n",
    "filter_count": "4666\n",
    "sort": "5000.5\n",
    "text": "10000\n",
}
N = int(sys.argv[1]) if len(sys.argv) > 1 else 50
WARMUPS = int(os.environ.get("BENCH_WARMUPS", "3"))
if N < 1 or WARMUPS < 0:
    sys.exit("Measurements must be positive; warmups cannot be negative")
BINARY = os.environ.get("SEMAURI_BIN", str(ROOT / "target/release/semauri"))
LABEL = os.environ.get("BENCH_LABEL", "candidate")
results = {}
for case in CASES:
    commands = {
        "Semauri": [BINARY, "run", "--allow", "console_write", str(ROOT / "benches" / (case + ".suri"))],
        "Python": ["python3", str(ROOT / "benches" / (case + ".py"))],
    }
    expected = EXPECTED[case]
    rows = {}
    for tool, cmd in commands.items():
        durations = []
        for iteration in range(WARMUPS + N):
            begin = time.perf_counter_ns()
            proc = subprocess.run(cmd, capture_output=True, text=True, check=False)
            elapsed = (time.perf_counter_ns() - begin) / 1_000_000
            if proc.returncode != 0 or proc.stdout != expected or proc.stderr:
                sys.exit(
                    f"{case} ({tool}) mismatch at iteration {iteration}: "
                    f"exit={proc.returncode}, stdout={proc.stdout!r}, stderr={proc.stderr!r}, "
                    f"expected={expected!r}"
                )
            if iteration >= WARMUPS:
                durations.append(elapsed)
        rows[tool] = {"median_ms": round(statistics.median(durations), 3),
                      "min_ms": round(min(durations), 3)}
    results[case] = rows

cpu = platform.processor()
if os.path.exists("/proc/cpuinfo"):
    for line in Path("/proc/cpuinfo").read_text().splitlines():
        if line.startswith("model name"):
            cpu = line.split(":", 1)[1].strip()
            break
metadata = {"label": LABEL, "os": platform.platform(), "cpu": cpu,
            "n": N, "warmups": WARMUPS, "results": results}
print(f"Machine: {metadata['os']}, CPU: {cpu}. {N} timed runs after {WARMUPS} warmups.")
print("| benchmark | Semauri median (ms) | Semauri min (ms) | Python median (ms) | Python min (ms) |")
print("|---|---:|---:|---:|---:|")
for case, values in results.items():
    s, p = values["Semauri"], values["Python"]
    print(f"| {case} | {s['median_ms']} | {s['min_ms']} | {p['median_ms']} | {p['min_ms']} |")
output = os.environ.get("BENCH_RESULTS")
if output:
    Path(output).write_text(json.dumps(metadata, indent=2) + "\n")
