#!/usr/bin/env python3
"""Time the head-less circular-crop pipeline for each language port."""

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def discover():
    impls = []
    rust = ROOT / "target" / "release" / "circular-bench"
    if rust.exists():
        impls.append(("Rust", [str(rust)]))
    c = ROOT / "c" / "circular-bench-c"
    if c.exists():
        impls.append(("C", [str(c)]))
    nim = ROOT / "nim" / "circular-img"
    if nim.exists():
        impls.append(("Nim", [str(nim)]))
    return impls


def run_once(command, input_path, cx, cy, radius):
    with tempfile.TemporaryDirectory() as tmp:
        local = Path(tmp) / input_path.name
        shutil.copyfile(input_path, local)
        argv = command + [str(local), str(cx), str(cy), str(radius)]
        start = time.perf_counter()
        subprocess.run(argv, stdout=subprocess.DEVNULL, check=True)
        return time.perf_counter() - start


def main():
    input_path = Path("/Users/han/Desktop/wallp-1.jpg")
    cx, cy, radius = 640, 280, 220

    results = {}
    for name, command in discover():
        times = [run_once(command, input_path, cx, cy, radius) for _ in range(3)]
        results[name] = times
        print(f"{name:>6} " + " ".join(f"{t * 1000:7.2f}ms" for t in times))

    print()
    ranked = sorted(results.items(), key=lambda item: min(item[1]))
    for rank, (name, times) in enumerate(ranked, start=1):
        print(f"{rank}. {name:<6} best {min(times) * 1000:.2f} ms")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
