#!/usr/bin/env python3
"""Build a training set from CodeNet with labels based on MEASURED memory.

Why this exists: the labels in `codecontests_subset/` come from source length and
problem difficulty (`extract_codecontests.py`), and measurement shows those labels
are essentially unrelated to memory behaviour - corr(log code_size, log memory) is
about 0.15 across CodeNet, i.e. source length explains roughly 2% of memory
variance. This script replaces that proxy with the real thing: CodeNet annotates
each submission with the memory footprint (KiB) and CPU time observed when it ran.

Sources of error it guards against (CodeNet metadata is known to be dirty):
  - `memory` / `cpu_time` are empty or negative for many non-accepted submissions,
    so rows without a positive memory reading are dropped
  - only `Accepted` rows are used by default: a Wrong Answer or Runtime Error run
    may never have exercised the program's real allocation path, so its memory
    reading is not a fair target. `--include-mle` adds Memory-Limit-Exceeded rows,
    which are by definition heavy and are otherwise rare.
  - the middle band between the two thresholds is DROPPED rather than forced into
    a class: submissions near the boundary are exactly where a label is ambiguous,
    and guessing there is what produced the original problem.

Output layout mirrors what `feature-extraction-pipeline` already expects, so the
existing Rust extractor can consume it unchanged:

    <out>/<C|C++|Java|Python>/<Light|Heavy>/<submission_id>.<ext>

Usage:
    ./.venv/bin/python extract_codenet.py --scan                 # stats only
    ./.venv/bin/python extract_codenet.py --per-class 20000
"""
from __future__ import annotations

import argparse
import csv
import glob
import os
import random
import sys

import pyarrow.parquet as pq

HERE = os.path.dirname(os.path.abspath(__file__))
SRC = {                       # CodeNet config -> (output dir, file extension)
    "Python": ("Python", ".py"),
    "Java": ("Java", ".java"),
    "C": ("C", ".c"),
    "C++": ("C++", ".cpp"),
}
MIB = 1024.0                  # CodeNet memory is in KiB


def shards(lang: str, root: str):
    return sorted(glob.glob(os.path.join(root, lang, "*.parquet")))


def scan(lang: str, root: str) -> dict:
    """Read only the columns needed to count how much usable data exists."""
    total = accepted = valid = 0
    buckets = {t: 0 for t in (25, 50, 100, 200, 500)}
    for f in shards(lang, root):
        t = pq.read_table(f, columns=["status", "memory"])
        for status, memory in zip(t.column("status").to_pylist(),
                                  t.column("memory").to_pylist()):
            total += 1
            if status == "Accepted":
                accepted += 1
            try:
                kb = int(memory)
            except (TypeError, ValueError):
                continue
            if kb <= 0:
                continue
            valid += 1
            mib = kb / MIB
            for thresh in buckets:
                if mib >= thresh:
                    buckets[thresh] += 1
    return {"total": total, "accepted": accepted, "valid": valid, "buckets": buckets}


def collect(lang: str, root: str, light_max: float, heavy_min: float,
            statuses: set[str], per_class: int, seed: int):
    """Return (light_rows, heavy_rows), each a list of dicts, capped per class."""
    rng = random.Random(seed)
    ext = SRC[lang][1]
    light, heavy = [], []
    seen = 0
    for f in shards(lang, root):
        t = pq.read_table(f, columns=["s_id", "p_id", "status", "memory",
                                      "cpu_time", "code_size", "code"])
        cols = {name: t.column(name).to_pylist()
                for name in ("s_id", "p_id", "status", "memory", "cpu_time",
                             "code_size", "code")}
        for i in range(t.num_rows):
            seen += 1
            status = cols["status"][i]
            if status not in statuses:
                continue
            try:
                kb = int(cols["memory"][i])
            except (TypeError, ValueError):
                continue
            if kb <= 0:
                continue
            code = cols["code"][i]
            if not code or len(code) < 40:      # skip empty/stub files
                continue
            mib = kb / MIB
            row = {"submission_id": cols["s_id"][i], "language": SRC[lang][0],
                   "problem_id": cols["p_id"][i], "status": status,
                   "memory_kb": kb, "memory_mib": round(mib, 3),
                   "cpu_time_ms": cols["cpu_time"][i],
                   "code_size": cols["code_size"][i],
                   "code": code, "ext": ext, "source_parquet": os.path.basename(f)}
            if mib < light_max:
                if len(light) < per_class:
                    light.append(row)
            elif mib >= heavy_min:
                if len(heavy) < per_class:
                    heavy.append(row)
            # else: boundary band, deliberately dropped
        if len(light) >= per_class and len(heavy) >= per_class:
            break
    return light, heavy, seen


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--root", default=os.path.join(HERE, "codenet"))
    ap.add_argument("--out", default=os.path.join(HERE, "codenet_subset"))
    ap.add_argument("--langs", nargs="+", default=["Python", "Java", "C", "C++"])
    ap.add_argument("--light-max-mib", type=float, default=25.0)
    ap.add_argument("--heavy-min-mib", type=float, default=100.0)
    ap.add_argument("--per-class", type=int, default=20000)
    ap.add_argument("--seed", type=int, default=42)
    ap.add_argument("--include-mle", action="store_true")
    ap.add_argument("--manifest-only", action="store_true",
                    help="rebuild the manifest without rewriting source files")
    ap.add_argument("--scan", action="store_true",
                    help="report available data and exit without writing")
    args = ap.parse_args()

    statuses = {"Accepted"} | ({"Memory Limit Exceeded"} if args.include_mle else set())

    if args.scan:
        print(f"{'language':9s} {'rows':>12s} {'accepted':>12s} {'valid mem':>12s}"
              + "".join(f"{f'>={t}MiB':>12s}" for t in (25, 100)))
        for lang in args.langs:
            if not shards(lang, args.root):
                print(f"{lang:9s} (no shards found)")
                continue
            s = scan(lang, args.root)
            print(f"{lang:9s} {s['total']:>12,} {s['accepted']:>12,} {s['valid']:>12,}"
                  f"{s['buckets'][25]:>12,}{s['buckets'][100]:>12,}")
        return 0

    print(f"labels: Light < {args.light_max_mib} MiB, Heavy >= {args.heavy_min_mib} MiB, "
          f"band [{args.light_max_mib}, {args.heavy_min_mib}) dropped")
    print(f"statuses: {sorted(statuses)}   cap per class per language: {args.per_class:,}")

    written, manifest = [], []
    for lang in args.langs:
        if not shards(lang, args.root):
            print(f"  {lang}: no shards, skipped")
            continue
        light, heavy, seen = collect(lang, args.root, args.light_max_mib,
                                     args.heavy_min_mib, statuses,
                                     args.per_class, args.seed)
        out_dir = SRC[lang][0]
        for label, rows in (("Light", light), ("Heavy", heavy)):
            d = os.path.join(args.out, out_dir, label)
            if not args.manifest_only:
                os.makedirs(d, exist_ok=True)
            for r in rows:
                path = os.path.join(d, r["submission_id"] + r["ext"])
                if not args.manifest_only:
                    with open(path, "w", encoding="utf-8", errors="replace") as fh:
                        fh.write(r["code"])
                m = {k: v for k, v in r.items() if k not in ("code", "ext")}
                m["label"] = label
                m["tier"] = label
                # Aliases the trainer reads by name (train_advanced_xgboost.py
                # selects cpu_time/memory/difficulty from the manifest).
                m["cpu_time"] = r["cpu_time_ms"]
                m["memory"] = r["memory_kb"]
                m["difficulty"] = -1
                m["relative_path"] = os.path.relpath(path, args.out)
                manifest.append(m)
            written.append((out_dir, label, len(rows)))
        print(f"  {lang:7s} scanned {seen:>10,} rows -> Light {len(light):>6,}  Heavy {len(heavy):>6,}")

    print("\nwritten:")
    for lang, label, n in written:
        print(f"  {lang:6s} {label:5s} {n:>7,}")

    if manifest:
        fields = ["submission_id", "language", "label", "tier", "problem_id",
                  "status", "memory_kb", "memory_mib", "cpu_time", "memory",
                  "difficulty", "code_size", "relative_path", "source_parquet"]
        out_csv = os.path.join(HERE, "codenet_manifest.csv")
        with open(out_csv, "w", newline="", encoding="utf-8") as fh:
            w = csv.DictWriter(fh, fieldnames=fields, extrasaction="ignore")
            w.writeheader()
            w.writerows(manifest)
        n_light = sum(1 for m in manifest if m["label"] == "Light")
        print(f"\nwrote {out_csv}: {len(manifest):,} rows "
              f"(Light {n_light:,} / Heavy {len(manifest)-n_light:,})")
        print(f"wrote sources under {args.out}/")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
