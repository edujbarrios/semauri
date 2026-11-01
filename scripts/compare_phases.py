#!/usr/bin/env python3
"""Compare five equal-output benchmark suites across compiler phases on one CPU."""
import json
import sys
from pathlib import Path

labels = ("phase0", "phase1", "phase2", "phase3")
data = [json.loads(Path(p).read_text()) for p in sys.argv[1:]]
if len(data) != len(labels):
    sys.exit("Expected phase0, phase1, phase2, phase3 benchmark JSON")
if len({(d["cpu"],d["os"],d["n"],d["warmups"]) for d in data}) != 1:
    sys.exit("Phase measurements cannot be compared across different runner specifications")
print("Compiler phase measurements on:",data[0]["cpu"]," / ",data[0]["os"])
print("Samples:",data[0]["n"],"warmups:",data[0]["warmups"])
print("| benchmark | phase0 (ms) | phase1 (ms) | phase2 (ms) | phase3 (ms) | total speedup | Python (ms) |")
print("|---|---:|---:|---:|---:|---:|---:|")
for case in data[0]["results"]:
    med = [d["results"][case]["Semauri"]["median_ms"] for d in data]
    python = data[-1]["results"][case]["Python"]["median_ms"]
    print(f"| {case} | {med[0]:.3f} | {med[1]:.3f} | {med[2]:.3f} | {med[3]:.3f} | {med[0]/med[3]:.2f}x | {python:.3f} |")
for previous, following in zip(data, data[1:]):
    print("Phase transition:",previous["label"],"->",following["label"])
    for case in previous["results"]:
        old=previous["results"][case]["Semauri"]["median_ms"]
        new=following["results"][case]["Semauri"]["median_ms"]
        print(f"  {case}: {old/new:.2f}x")
