#!/usr/bin/env python3
"""Render the five committed benchmark charts from docs/data JSON reports.

Standard library only. The optional docs/data/summary.json is deliberately not
used: every label is recomputed from the raw run reports. Use --check to detect
stale or missing committed SVGs.
"""

import argparse
import difflib
import json
import math
import statistics
import xml.etree.ElementTree as ET
from pathlib import Path
from xml.sax.saxutils import escape

WIDTH, HEIGHT = 1080, 560
PX, PY, PW, PH = 88, 72, 950, 370
FOOT_Y, LIMIT = 478, 40_000
BARS = {
    "light": ["#0969da", "#8250df", "#1a7f37", "#cf222e", "#9a6700", "#0550ae"],
    "dark": ["#58a6ff", "#bc8cff", "#3fb950", "#f85149", "#d29922", "#1f6feb"],
}
LIGHT = dict(
    bg="#ffffff", text="#1f2328", muted="#57606a", grid="#d0d7de", line="#0969da"
)
DARK = dict(
    bg="#0d1117", text="#e6edf3", muted="#8b949e", grid="#30363d", line="#58a6ff"
)
THEMES = {"light": LIGHT, "dark": DARK}
NAMES = "enso-latency enso-vs-kociemba solution-length".split()
NAMES += "exa-depth-curve overhead-floor".split()


def run_files(stem):
    return [stem.format(i=i) for i in range(1, 6)]


FILES = {
    "cubie": run_files("enso-cubie-100k-seed1-run{i}.json"),
    "facelets": run_files("enso-facelets-100k-seed1-run{i}.json"),
    "batch": run_files("enso-facelets-batch-100k-seed1-run{i}.json"),
    "ocubie": run_files("overhead-cubie-100k-seed1-run{i}.json"),
    "ofacelets": run_files("overhead-facelets-100k-seed1-run{i}.json"),
    "k21": ["kociemba-d21-100k-seed1.json"],
    "k24": ["kociemba-d24-100k-seed1.json"],
}
FILES.update({f"e{d}": [f"exa-depth-{d}.json"] for d in range(8, 15)})
EXA = [f"e{d}" for d in range(8, 15)]


def load(repo, names):
    rows = []
    for name in names:
        path = repo / "docs" / "data" / name
        if not path.is_file():
            raise SystemExit(f"missing data file: {path}")
        try:
            rows.append((name, json.loads(path.read_text(encoding="utf-8"))))
        except (OSError, ValueError) as exc:
            raise SystemExit(f"bad JSON in {path}: {exc}") from exc
    return rows


def values(rows, key):
    out = []
    for name, report in rows:
        if report.get(key) is None:
            raise SystemExit(f"{name}: missing {key}")
        out.append(float(report[key]))
    return out


def per_solve_ns(report, name):
    if report.get("batch_ns") is not None and report.get("n"):
        return float(report["batch_ns"]) / float(report["n"])
    for key in ("p50_ns", "avg_ns"):
        if report.get(key) is not None:
            return float(report[key])
    raise SystemExit(f"{name}: no timing field found")


def mean_length(report, name):
    if report.get("avg_length") is None:
        raise SystemExit(f"{name}: missing avg_length")
    return float(report["avg_length"])


def stat(rows, fn):
    samples = [fn(report, name) for name, report in rows]
    return statistics.median(samples), min(samples), max(samples)


def fmt_time(value):
    value = float(value)
    for unit, divisor in (("s", 1e9), ("ms", 1e6), ("us", 1e3)):
        if value >= divisor:
            return f"{value / divisor:.2f} {unit}"
    return f"{value:.0f} ns"


def fmt_length(value):
    return f"{float(value):.2f} moves"


def unique(items):
    return list(dict.fromkeys(items))


def footnote(rows):
    commits, cpus, dates = [], [], []
    for name, report in rows:
        meta = report.get("meta") or {}
        commit = (meta.get("git") or {}).get("commit")
        cpu = (meta.get("machine") or {}).get("cpu_model")
        date = meta.get("annotated_at_utc")
        if not commit or not cpu or not date:
            raise SystemExit(f"{name}: incomplete meta")
        commits.append(str(commit)[:12])
        cpus.append(str(cpu))
        dates.append(str(date)[:10])
    files = ", ".join(name for name, _ in rows)
    source = f"commit: {', '.join(unique(commits))} | CPU: {', '.join(unique(cpus))}"
    return f"Data: {files} | {source} | UTC: {', '.join(unique(dates))}"


def wrap(text, width=168):
    lines, line = [], ""
    for word in str(text).split(" "):
        if line and len(line) + 1 + len(word) > width:
            lines.append(line)
            line = word
        else:
            line = f"{line} {word}" if line else word
    return lines + ([line] if line else [])


class Svg:
    def __init__(self, mode, title, subtitle, foot):
        self.theme = THEMES[mode]
        self.title = title
        bg = self.theme["bg"]
        self.parts = [f'<rect width="{WIDTH}" height="{HEIGHT}" fill="{bg}"/>']
        self.text(24, 32, title, 21, weight="700")
        self.text(24, 54, subtitle, 12, color="muted")
        for index, line in enumerate(wrap(foot)):
            self.text(24, FOOT_Y + 14 * index, line, 9, color="muted")

    def text(self, x, y, value, size=12, anchor="start", color="text", weight="400"):
        attrs = (
            f'x="{x:.1f}" y="{y:.1f}" font-family="sans-serif" '
            f'font-size="{size}" fill="{self.theme[color]}" '
            f'text-anchor="{anchor}" font-weight="{weight}"'
        )
        self.parts.append(f"<text {attrs}>{escape(str(value))}</text>")

    def rect(self, x, y, w, h, fill):
        geom = f'x="{x:.1f}" y="{y:.1f}" width="{w:.1f}" height="{max(h, 0.5):.1f}"'
        self.parts.append(f'<rect {geom} fill="{fill}"/>')

    def line(self, x1, y1, x2, y2, color="grid", width=1):
        geom = f'x1="{x1:.1f}" y1="{y1:.1f}" x2="{x2:.1f}" y2="{y2:.1f}"'
        self.parts.append(
            f'<line {geom} stroke="{self.theme[color]}" stroke-width="{width}"/>'
        )

    def render(self):
        head = (
            '<?xml version="1.0" encoding="UTF-8"?>\n'
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" '
            f'height="{HEIGHT}" viewBox="0 0 {WIDTH} {HEIGHT}" role="img">'
            f"<title>{escape(self.title)}</title>"
        )
        return head + "\n" + "\n".join(self.parts) + "\n</svg>\n"


def axis(svg, samples, formatter, force_log=False):
    positive = [value for value in samples if value > 0]
    if not positive:
        raise SystemExit("chart has no positive values")
    low_sample, high_sample = min(positive), max(positive)
    use_log = force_log or high_sample / max(low_sample, 1e-30) >= 20
    if use_log:
        low, high = low_sample / 2, high_sample * 2
        ticks = [low_sample, math.sqrt(low_sample * high_sample), high_sample]
    else:
        low, high = 0.0, high_sample * 1.15
        ticks = [0.0, high_sample / 2, high_sample]

    def y(value):
        if use_log:
            fraction = math.log(max(value, low) / low) / math.log(high / low)
        else:
            fraction = (value - low) / (high - low) if high > low else 0.0
        return PY + PH - fraction * PH

    for tick in ticks:
        tick_y = y(tick)
        svg.line(PX, tick_y, PX + PW, tick_y)
        if tick > 0 or not use_log:
            svg.text(PX - 8, tick_y + 4, formatter(tick), 10, "end", "muted")
    if use_log:
        svg.text(PX + PW, PY - 12, "log scale", 10, "end", "muted")
    return y


def bar_chart(mode, title, subtitle, foot, items, formatter=fmt_time, force_log=False):
    svg = Svg(mode, title, subtitle, foot)
    samples = [value for _, value, low, high in items for value in (value, low, high)]
    y = axis(svg, samples, formatter, force_log)
    slot = PW / len(items)
    width = min(88, slot * 0.58)
    colors = BARS[mode]
    for index, (label, value, low, high) in enumerate(items):
        center, top, bottom = PX + slot * (index + 0.5), y(value), PY + PH
        svg.rect(
            center - width / 2, top, width, bottom - top, colors[index % len(colors)]
        )
        if high > low:
            svg.line(center, y(high), center, y(low), "text", 2)
            for edge in (high, low):
                svg.line(center - 5, y(edge), center + 5, y(edge), "text", 2)
        svg.text(center, top - 8, formatter(value), 11, "middle", "text", "700")
        svg.text(center, bottom + 20, label, 11, "middle")
    return svg.render()


def line_chart(mode, title, subtitle, foot, points):
    svg = Svg(mode, title, subtitle, foot)
    y = axis(svg, [value for _, value in points], fmt_time)
    xs = [PX + PW * (index + 0.5) / len(points) for index in range(len(points))]
    stroke = THEMES[mode]["line"]
    coords = " ".join(f"{x:.1f},{y(value):.1f}" for x, (_, value) in zip(xs, points))
    polyline = (
        f'<polyline points="{coords}" fill="none" stroke="{stroke}" stroke-width="3"/>'
    )
    svg.parts.append(polyline)
    for x, (label, value) in zip(xs, points):
        cy = y(value)
        svg.parts.append(f'<circle cx="{x:.1f}" cy="{cy:.1f}" r="4" fill="{stroke}"/>')
        svg.text(x, cy - 12, fmt_time(value), 11, "middle", "text", "700")
        svg.text(x, PY + PH + 20, label, 11, "middle")
    return svg.render()


def build(repo):
    sets = {key: load(repo, names) for key, names in FILES.items()}
    t = {key: stat(rows, per_solve_ns) for key, rows in sets.items()}
    l = {key: stat(rows, mean_length) for key, rows in sets.items()}
    r = lambda *keys: [row for key in keys for row in sets[key]]
    it = lambda label, triple: (label,) + triple
    tk = lambda label, key: it(label, t[key])
    lk = lambda label, key: it(label, l[key])
    n_enso = int(statistics.median(values(sets["cubie"], "n")))
    seed = int(statistics.median(values(sets["cubie"], "seed")))
    n_exa = int(statistics.median(values(sets["e8"], "n")))
    seed_exa = int(statistics.median(values(sets["e8"], "seed")))
    runs = len(sets["cubie"])
    depths = [sets[key][0][1]["depth"] for key in EXA]
    ko_depths = [sets[key][0][1]["max_depth"] for key in ("k21", "k24")]
    enso_latency = [
        tk("cubie", "cubie"),
        tk("facelets", "facelets"),
        tk("batch", "batch"),
    ]
    ko_compare = [
        tk("enso cubie", "cubie"),
        tk("enso facelets", "facelets"),
        tk("kociemba d21", "k21"),
        tk("kociemba d24", "k24"),
    ]
    len_items = [
        lk("enso cubie", "cubie"),
        lk("enso facelets", "facelets"),
        lk("kociemba d21", "k21"),
        lk("kociemba d24", "k24"),
    ] + [it(f"exa {d}", l[key]) for key, d in zip(EXA, depths)]
    exa_points = [(str(d), t[key][0]) for key, d in zip(EXA, depths)]
    floor_items = [
        tk("overhead cubie", "ocubie"),
        tk("enso cubie", "cubie"),
        tk("overhead facelets", "ofacelets"),
        tk("enso facelets", "facelets"),
    ]
    title1 = f"enso latency ({n_enso:,} cubes, seed {seed})"
    sub1 = f"p50; batch=batch_ns/n; whiskers={runs}-run min..max"
    foot1 = footnote(r("cubie", "facelets", "batch"))
    title2 = "enso vs external kociemba (p50 solve time)"
    sub2 = f"kociemba max_depth {ko_depths[0]}/{ko_depths[1]}"
    foot2 = footnote(r("cubie", "facelets", "k21", "k24"))
    title3 = "mean solution length (HTM)"
    sub3 = f"n={n_enso:,}; whiskers=min..max run means; exa labels=depth"
    foot3 = footnote(r("cubie", "facelets", "k21", "k24", *EXA))
    title4 = "exa solve time vs scramble depth"
    sub4 = f"n={n_exa:,}, seed {seed_exa}; benchkit per-solve timing"
    foot4 = footnote(r(*EXA))
    title5 = f"harness overhead floor ({n_enso:,} inputs)"
    sub5 = "overhead-loop = input loop without solver; whiskers=min..max"
    foot5 = footnote(r("ocubie", "ofacelets", "cubie", "facelets"))
    out = {}
    for mode in THEMES:
        charts = {
            "enso-latency": bar_chart(mode, title1, sub1, foot1, enso_latency),
            "enso-vs-kociemba": bar_chart(
                mode, title2, sub2, foot2, ko_compare, force_log=True
            ),
            "solution-length": bar_chart(
                mode, title3, sub3, foot3, len_items, fmt_length
            ),
            "exa-depth-curve": line_chart(mode, title4, sub4, foot4, exa_points),
            "overhead-floor": bar_chart(
                mode, title5, sub5, foot5, floor_items, force_log=True
            ),
        }
        if list(charts) != NAMES:
            raise SystemExit(f"expected charts {NAMES}, got {list(charts)}")
        out.update({f"{name}-{mode}.svg": svg for name, svg in charts.items()})
    return out


def main():
    root = Path(__file__).resolve().parent.parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=root)
    parser.add_argument("--check", action="store_true", help="check committed SVGs")
    args = parser.parse_args()
    repo = args.repo.resolve()
    charts = build(repo)
    for name, svg in charts.items():
        ET.fromstring(svg)
        size = len(svg.encode("utf-8"))
        if size >= LIMIT:
            raise SystemExit(f"{name}: {size} bytes exceeds {LIMIT} byte limit")
    if args.check:
        for name, svg in sorted(charts.items()):
            path = repo / "docs" / "assets" / name
            if not path.is_file():
                raise SystemExit(f"missing SVG: {path} (regenerate without --check)")
            old = path.read_text(encoding="utf-8")
            if old != svg:
                delta = difflib.unified_diff(
                    old.splitlines(True),
                    svg.splitlines(True),
                    fromfile=str(path),
                    tofile=str(path),
                )
                raise SystemExit(f"stale SVG: {path}\n" + "".join(list(delta)[:50]))
        print("OK")
    else:
        assets = repo / "docs" / "assets"
        assets.mkdir(parents=True, exist_ok=True)
        for name, svg in sorted(charts.items()):
            (assets / name).write_text(svg, encoding="utf-8")
        print(f"wrote {len(charts)} SVGs to {assets}")


if __name__ == "__main__":
    main()
