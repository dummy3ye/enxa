#!/usr/bin/env python3
"""Annotate a benchmark JSON file with reproducible machine/repo metadata.

Standard library only. This does not measure anything; it records the environment in
which an existing measurement was made.
"""

import argparse
import datetime
import hashlib
import json
import os
import platform
import subprocess
from pathlib import Path


def _read_first(path):
    try:
        return Path(path).read_text().splitlines()[0].strip()
    except (OSError, IndexError):
        return None


def _git(*args):
    try:
        return subprocess.check_output(["git", *args], text=True).strip()
    except (OSError, subprocess.CalledProcessError):
        return None


def sha256_file(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def metadata(cube_file, command):
    cpuinfo = (
        Path("/proc/cpuinfo").read_text().splitlines()
        if Path("/proc/cpuinfo").exists()
        else []
    )
    model_line = next((line for line in cpuinfo if line.startswith("model name")), None)
    model = model_line.split(":", 1)[1].strip() if model_line else None
    flags_line = next((line for line in cpuinfo if line.startswith("flags")), None)
    freq_dir = Path("/sys/devices/system/cpu/cpu0/cpufreq")
    cpu_freq = {}
    for field in ("scaling_governor", "scaling_min_freq", "scaling_max_freq"):
        cpu_freq[field] = _read_first(freq_dir / field)
    try:
        current = sorted(
            {
                line.strip()
                for line in Path("/sys/devices/system/cpu").glob(
                    "cpu*/cpufreq/scaling_cur_freq"
                )
                for line in [line.read_text()]
            }
        )
    except OSError:
        current = []
    cpu_freq["scaling_cur_freq"] = current
    try:
        rustc = subprocess.check_output(["rustc", "--version"], text=True).strip()
    except (OSError, subprocess.CalledProcessError):
        rustc = None
    return {
        "annotated_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "command": command,
        "git": {
            "commit": _git("rev-parse", "HEAD"),
            "dirty": bool(_git("status", "--porcelain")),
        },
        "cube_file": str(cube_file) if cube_file else None,
        "cube_sha256": sha256_file(cube_file) if cube_file else None,
        "rustc": rustc,
        "rustflags": os.environ.get("RUSTFLAGS"),
        "machine": {
            "cpu_model": model,
            "cpu_flags": flags_line.split(":", 1)[1].strip() if flags_line else None,
            "os": platform.platform(),
            "python": platform.python_version(),
            "logical_cpus": os.cpu_count(),
        },
        "cpu_freq": cpu_freq,
    }


def annotate(path, cube_file, command):
    data = json.loads(Path(path).read_text())
    data["meta"] = metadata(cube_file, command)
    Path(path).write_text(json.dumps(data, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--json", required=True, help="existing benchmark JSON to annotate"
    )
    parser.add_argument(
        "--cube-file",
        help="cube file whose SHA-256 is recorded; omit for generated inputs",
    )
    parser.add_argument(
        "--command", required=True, help="exact command that produced the JSON"
    )
    args = parser.parse_args()
    annotate(args.json, args.cube_file, args.command)


if __name__ == "__main__":
    main()
