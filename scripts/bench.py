#!/usr/bin/env python3
"""Compare Semauri CLI and CPython, including process startup, with exact stdout.

No third-party Python dependencies. Use the SAME runner/CPU, cargo --release
build and CLI invocation for each phase. CI VMs can vary; do not compare
results from different CPU models as if they were controlled speedups.
"""
import argparse
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parent.parent
NAMES = ("hello", "map_reduce", "filter_count", "sort", "text")


def machine():
    model = "unknown"
    try:
        text = Path("/proc/cpuinfo").read_text()
        for line in text.splitlines():
            if line.startswith("model name"):
                model = line.partition(":")[2].strip()
                break
    except OSError:
        pass
    return {"platform": platform.platform(), "machine": platform.machine(),
            "cpu": model, "python": platform.python_version(),
            "hostname": platform.node()}


def invocation(name, language, semauri):
    file = ROOT / "benches" / (name + (".suri" if language == "semauri" else ".py"))
    if language == "semauri":
        return [str(semauri), "run", "--allow", "console_write", str(file)]
    return ["python3", str(file)]


def execute(argv):
    start = time.perf_counter_ns()
    result = subprocess.run(argv, cwd=ROOT, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, check=False)
    elapsed_ms = (time.perf_counter_ns() - start) / 1_000_000
    if result.returncode != 0:
        raise RuntimeError(f"{argv!r}: status {result.returncode}: {result.stderr.decode(errors='replace')}")
    return elapsed_ms, result.stdout


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--runs", type=int, default=50)
    parser.add_argument("--warmups", type=int, default=3)
    parser.add_argument("--semauri", type=Path, default=ROOT / "target/release/semauri")
    parser.add_argument("--json", type=Path)
    args = parser.parse_args()
    if args.runs < 1 or args.warmups < 0:
        parser.error("runs must be positive and warmups nonnegative")
    semauri = args.semauri.resolve()
    if not semauri.exists():
        parser.error(f"release binary not found: {semauri}; run cargo build --release")
    report = {"machine": machine(), "runs": args.runs, "warmups": args.warmups,
              "results": []}
    print("CPU:", report["machine"]["cpu"])
    print("Platform:", report["machine"]["platform"])
    print("Runs:", args.runs, "warmups:", args.warmups)
    print(f'{"benchmark":<15} {"semauri median":>15} {"semauri min":>13} {"Python median":>15} {"Python min":>11} {"speedup":>10}')
    for name in NAMES:
        commands = {language: invocation(name, language, semauri)
                    for language in ("semauri", "python")}
        # Check outputs before timing and fail fast on semantic divergence.
        actual = {language: execute(command)[1] for language, command in commands.items()}
        if actual["semauri"] != actual["python"]:
            raise RuntimeError(f"{name}: stdout mismatch: Semauri={actual['semauri']!r}; Python={actual['python']!r}")
        result = {"name": name, "stdout": actual["semauri"].decode(errors="replace")}
        for language, command in commands.items():
            for _ in range(args.warmups):
                elapsed, output = execute(command)
                if output != actual[language]:
                    raise RuntimeError(f"{name} {language}: non-deterministic warmup output")
            samples = []
            for _ in range(args.runs):
                elapsed, output = execute(command)
                if output != actual[language]:
                    raise RuntimeError(f"{name} {language}: non-deterministic output")
                samples.append(elapsed)
            result[language] = {"median_ms": statistics.median(samples),
                                "min_ms": min(samples)}
        sem = result["semauri"]
        py = result["python"]
        ratio = py["median_ms"] / sem["median_ms"]
        print(f'{name:<15} {sem["median_ms"]:>15.3f} {sem["min_ms"]:>13.3f} {py["median_ms"]:>15.3f} {py["min_ms"]:>11.3f} {ratio:>10.2f}x')
        report["results"].append(result)
    if args.json:
        args.json.parent.mkdir(parents=True, exist_ok=True)
        args.json.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")


if __name__ == "__main__":
    main()
