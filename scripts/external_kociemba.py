#!/usr/bin/env python3
"""Run external kociemba on a committed cube file and write raw benchmark JSON.

This script is intentionally outside the Rust workspace. It needs an isolated Python
environment with `kociemba` installed; the repository itself has no such dependency.
Every returned solution is written to a dump for independent verification by
`benchkit --verify-file`. The internal check here only catches transcription errors.
"""

import argparse
import hashlib
import json
import os
import time
from importlib.metadata import version
from pathlib import Path

import kociemba
from kociemba.pykociemba.cubiecube import CubieCube, moveCube
from kociemba.pykociemba.facecube import FaceCube

from bench_meta import metadata

FACES = "URFDLB"


def _parse_own_solution(text):
    out = []
    for token in text.split():
        face = FACES.index(token[0])
        power = 2 if token.endswith("2") else 3 if token.endswith("'") else 1
        out.append((face, power))
    return out


def _internally_valid(facelets, solution):
    cube = FaceCube(facelets).toCubieCube()
    for face, power in _parse_own_solution(solution):
        for _ in range(power):
            cube.multiply(moveCube[face])
    solved = CubieCube()
    return (
        cube.cp == solved.cp
        and cube.co == solved.co
        and cube.ep == solved.ep
        and cube.eo == solved.eo
    )


def _pct(sorted_values, fraction):
    return sorted_values[int((len(sorted_values) - 1) * fraction)]


def _noop(_value):
    return None


def overhead_floor(strings, repeats=3):
    totals = []
    for _ in range(repeats):
        start = time.perf_counter_ns()
        for item in strings:
            _noop(item)
        totals.append(time.perf_counter_ns() - start)
    totals.sort()
    return totals[len(totals) // 2]


def sha256_file(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--cubes", required=True, help="facelet file, one 54-char state per line"
    )
    parser.add_argument(
        "--max-depth", required=True, type=int, help="kociemba max_depth"
    )
    parser.add_argument(
        "--solutions", required=True, help="output dump, one solution per line"
    )
    parser.add_argument("--json", required=True, help="raw JSON report")
    parser.add_argument(
        "--command", required=True, help="exact command that produced this run"
    )
    parser.add_argument("--warmup", type=int, default=1, help="untimed warm-up solves")
    args = parser.parse_args()

    cubes = [
        line.strip()
        for line in Path(args.cubes).read_text().splitlines()
        if line.strip()
    ]
    if any(len(cube) != 54 for cube in cubes):
        raise SystemExit("cube file contains a line that is not 54 facelets")
    if not cubes:
        raise SystemExit("cube file is empty")
    if not (0 <= args.warmup <= len(cubes)):
        raise SystemExit("invalid warmup count")

    for cube in cubes[: args.warmup]:
        kociemba.solve(cube, max_depth=args.max_depth)

    times = []
    lengths = []
    failed = 0
    invalid = 0
    over_depth = 0
    solutions = []
    for cube in cubes:
        start = time.perf_counter_ns()
        try:
            solution = kociemba.solve(cube, max_depth=args.max_depth)
        except Exception:
            failed += 1
            solutions.append("")
            continue
        elapsed = time.perf_counter_ns() - start
        move_count = len(solution.split())
        if not _internally_valid(cube, solution):
            invalid += 1
        if move_count > args.max_depth:
            over_depth += 1
        times.append(elapsed)
        lengths.append(move_count)
        solutions.append(solution)

    Path(args.solutions).write_text("\n".join(solutions) + "\n")
    sorted_times = sorted(times)
    total = sum(times)
    floor_total = overhead_floor(cubes)
    report = {
        "solver": "kociemba",
        "kociemba_version": version("kociemba"),
        "max_depth": args.max_depth,
        "n": len(cubes),
        "warmup": args.warmup,
        "solved": len(lengths),
        "failed": failed,
        "invalid": invalid,
        "over_depth": over_depth,
        "avg_length": round(sum(lengths) / len(lengths), 2) if lengths else 0.0,
        "max_length": max(lengths) if lengths else 0,
        "avg_ns": round(total / len(times), 1) if times else 0.0,
        "p50_ns": _pct(sorted_times, 0.5) if times else 0,
        "p99_ns": _pct(sorted_times, 0.99) if times else 0,
        "total_solve_ns": total,
        "worker_overhead_floor_total_ns": floor_total,
        "worker_overhead_floor_per_item_ns": round(floor_total / len(cubes), 2),
        "throughput": round(len(times) * 1e9 / total, 1) if total else 0.0,
        "solution_file": args.solutions,
        "solution_file_sha256": sha256_file(args.solutions),
        "lengths": lengths,
        "times_ns": times,
    }
    report["meta"] = metadata(args.cubes, args.command)
    Path(args.json).write_text(json.dumps(report, indent=2) + "\n")
    print(
        f"max_depth={args.max_depth} n={len(cubes)} solved={len(lengths)} "
        f"failed={failed} invalid={invalid} avg_length={report['avg_length']} "
        f"p50_ns={report['p50_ns']}"
    )


if __name__ == "__main__":
    main()
