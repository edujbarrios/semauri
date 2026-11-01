#!/usr/bin/env python3
"""Compare benchmark medians from the *same* runner; report speedup."""
import json
import sys
from pathlib import Path

before, after = [json.loads(Path(p).read_text()) for p in sys.argv[1:3]]
if (before["cpu"], before["os"]) != (after["cpu"], after["os"]):
    sys.exit("Cannot compare performance on different CPUs/OSes")
print("CPU:", before["cpu"])
print("OS:", before["os"])
print("Runs:", before["n"], "warmups:", before["warmups"])
print("| benchmark | before (ms) | after (ms) | speedup | Python (ms) |")
print("|---|---:|---:|---:|---:|")
for name in before["results"]:
    old = before["results"][name]["Semauri"]["median_ms"]
    new = after["results"][name]["Semauri"]["median_ms"]
    python = after["results"][name]["Python"]["median_ms"]
    print(f"| {name} | {old:.3f} | {new:.3f} | {old/new:.2f}x | {python:.3f} |")
