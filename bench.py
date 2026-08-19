#!/usr/bin/env python3
"""Benchmark the head-less circular-crop pipeline across language ports.

All implementations link the same shared C codec (common/libcodec.a), so
decode and PNG encode cost are identical. The timed work is: decode image ->
build the anti-aliased circular crop -> encode PNG. Any difference is the
language runtime plus the crop loop.

Usage:
    python3 bench.py                          # 3 runs, repeat=1 (end to end)
    python3 bench.py --repeat 200             # amplify the crop loop
    python3 bench.py --runs 5 --timing        # show decode/crop/save breakdown
"""

from __future__ import annotations

import argparse
import os
import re
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent

PHASE_RE = re.compile(
    r"\[(?P<lang>\w+)\] decode (?P<decode>[\d.]+) ms \| crop (?P<crop>[\d.]+) ms"
)


def discover_implementations() -> list[tuple[str, list[str]]]:
    impls: list[tuple[str, list[str]]] = []

    # Rust: prefer the shared-codec benchmark binary.
    rust = ROOT / "target" / "release" / "circular-bench"
    if rust.exists():
        impls.append(("Rust", [str(rust)]))

    c = ROOT / "c" / "circular-bench-c"
    if not c.exists():
        c = ROOT / "c" / "circular-img-c"
    if c.exists():
        impls.append(("C", [str(c)]))

    nim = ROOT / "nim" / "circular-img"
    if nim.exists():
        impls.append(("Nim", [str(nim)]))

    return impls


def run_once(
    command: list[str],
    input_path: Path,
    cx: float,
    cy: float,
    radius: float,
    env: dict[str, str],
) -> tuple[float, str]:
    """Copy the input to a scratch dir, run, return (wall seconds, stderr)."""
    with tempfile.TemporaryDirectory() as tmp:
        local_input = Path(tmp) / input_path.name
        shutil.copyfile(input_path, local_input)

        argv = command + [str(local_input), str(cx), str(cy), str(radius)]

        full_env = dict(os.environ)
        full_env.update(env)

        start = time.perf_counter()
        result = subprocess.run(
            argv, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, env=full_env
        )
        elapsed = time.perf_counter() - start

        if result.returncode != 0:
            sys.stderr.write(result.stderr.decode(errors="replace"))
            raise SystemExit(
                f"Command failed ({result.returncode}): {' '.join(argv)}"
            )

        return elapsed, result.stderr.decode(errors="replace")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--runs", type=int, default=3)
    parser.add_argument("--warmup", type=int, default=2,
                        help="untimed warm-up runs per implementation")
    parser.add_argument("--input", default="/Users/han/Desktop/wallp-1.jpg")
    parser.add_argument("--cx", type=float, default=640)
    parser.add_argument("--cy", type=float, default=280)
    parser.add_argument("--radius", type=float, default=220)
    parser.add_argument("--repeat", type=int, default=1,
                        help="crop iterations per process (amplifies crop cost)")
    parser.add_argument("--timing", action="store_true",
                        help="show decode/crop/save breakdown per run")
    args = parser.parse_args()

    input_path = Path(args.input)
    if not input_path.exists():
        raise SystemExit(f"Input not found: {input_path}")

    impls = discover_implementations()
    if not impls:
        raise SystemExit("No built implementations found. Build them first.")

    label = "end-to-end" if args.repeat == 1 else f"crop x{args.repeat}"
    print(f"Input : {input_path}")
    print(f"Params: cx={args.cx} cy={args.cy} radius={args.radius}")
    print(f"Runs  : {args.runs} per implementation  ({label})")
    print("Codec : shared common/libcodec.a (stb) for all languages\n")

    run_env = {"CIRCULAR_REPEAT": str(args.repeat)}
    if args.timing:
        run_env["CIRCULAR_TIMING"] = "1"

    results: dict[str, list[float]] = {}
    phases: dict[str, list[tuple[float, float]]] = {}

    for name, command in impls:
        for _ in range(args.warmup):
            run_once(command, input_path, args.cx, args.cy, args.radius, run_env)

        timings = []
        phase_rows = []
        for run in range(1, args.runs + 1):
            elapsed, stderr = run_once(
                command, input_path, args.cx, args.cy, args.radius, run_env
            )
            timings.append(elapsed)
            print(f"{name:>6} run {run}: {elapsed * 1000:8.2f} ms")
            match = PHASE_RE.search(stderr)
            if match:
                phase_rows.append(
                    (float(match.group("decode")), float(match.group("crop")))
                )
        results[name] = timings
        if phase_rows:
            phases[name] = phase_rows
            if args.timing:
                for decode, crop in phase_rows:
                    print(f"             decode {decode:7.2f} ms | crop {crop:7.2f} ms")
        print()

    ranked = sorted(results.items(), key=lambda item: statistics.median(item[1]))

    print("=" * 58)
    print(f"{'rank':<6}{'lang':<8}{'median':>11}{'best':>11}{'all (ms)':>22}")
    print("-" * 58)
    for rank, (name, timings) in enumerate(ranked, start=1):
        median = statistics.median(timings) * 1000
        best = min(timings) * 1000
        all_ms = " ".join(f"{t * 1000:.1f}" for t in timings)
        print(f"{rank:<6}{name:<8}{median:>10.2f}ms{best:>10.2f}ms{all_ms:>22}")
    print("=" * 58)

    if phases:
        print(f"\nPhase medians ({label}):")
        print(f"{'lang':<8}{'decode':>12}{'crop':>12}{'total':>12}")
        for name, rows in phases.items():
            d = statistics.median(r[0] for r in rows)
            c = statistics.median(r[1] for r in rows)
            print(f"{name:<8}{d:>10.2f}ms{c:>10.2f}ms{d + c:>10.2f}ms")

    fastest = ranked[0]
    if len(ranked) > 1:
        slowest = statistics.median(ranked[-1][1])
        speedup = slowest / statistics.median(fastest[1])
        print(f"\nWinner: {fastest[0]}  ({speedup:.3f}x vs slowest)")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
