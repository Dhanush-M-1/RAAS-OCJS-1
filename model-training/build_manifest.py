#!/usr/bin/env python3
"""Build a training manifest from an extracted source tree.

Why this exists: the directories themselves are the label. `codecontests_subset/
<Language>/<Light|Heavy>/` is produced by the extractor, but the manifest it
writes alongside is gitignored, so a clean checkout could not retrain anything.
This rebuilds the manifest from what is on disk, with no network access.

It also reconciles a naming mismatch that silently broke the pipeline: both
extractors write a `tier` column, while `train_advanced_xgboost.py` reads
`label`. Emitting both keeps the trainer working and keeps the extractor's
vocabulary.

Usage:
    ./.venv/bin/python build_manifest.py [--subset codecontests_subset]
                                         [--out sample_manifest.csv]
"""
from __future__ import annotations

import argparse
import csv
import os
import re
import sys

TIERS = ("Light", "Heavy")
EXT_TO_LANG = {".cpp": "C++", ".cc": "C++", ".py": "Python", ".java": "Java", ".c": "C"}


def problem_id_from_submission(submission_id: str) -> str:
    """Recover the problem grouping key from a submission id.

    Extracted ids look like `cc_<split>_<sanitised problem name>_<solution idx>`.
    `problem_id` is what the trainer groups on for its leakage-safe split, so it
    must collapse every solution of one problem to the same value.
    """
    sid = submission_id
    sid = re.sub(r"^cc_(train|valid|test)_", "", sid)
    sid = re.sub(r"_\d+$", "", sid)   # trailing solution index
    return sid


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--subset", default="codecontests_subset",
                    help="directory laid out as <Language>/<Light|Heavy>/<file>")
    ap.add_argument("--out", default="sample_manifest.csv")
    args = ap.parse_args()

    if not os.path.isdir(args.subset):
        print(f"error: no such directory: {args.subset}", file=sys.stderr)
        return 2

    rows = []
    for lang in sorted(os.listdir(args.subset)):
        lang_dir = os.path.join(args.subset, lang)
        if not os.path.isdir(lang_dir):
            continue
        for tier in TIERS:
            tier_dir = os.path.join(lang_dir, tier)
            if not os.path.isdir(tier_dir):
                continue
            for fname in sorted(os.listdir(tier_dir)):
                stem, ext = os.path.splitext(fname)
                lang_name = EXT_TO_LANG.get(ext.lower(), lang)
                rows.append({
                    "submission_id": stem,
                    "language": lang_name,
                    "label": tier,          # what train_advanced_xgboost.py reads
                    "tier": tier,           # what the extractors write
                    "problem_id": problem_id_from_submission(stem),
                    "problem_name": problem_id_from_submission(stem),
                    "difficulty": -1,
                    "source_platform": "CODECONTESTS",
                    "file_path": os.path.join(tier_dir, fname),
                })

    fields = ["submission_id", "language", "label", "tier", "problem_id",
              "problem_name", "difficulty", "source_platform", "file_path"]
    with open(args.out, "w", newline="", encoding="utf-8") as fh:
        w = csv.DictWriter(fh, fieldnames=fields)
        w.writeheader()
        w.writerows(rows)

    from collections import Counter
    per_lang = Counter(r["language"] for r in rows)
    per_class = Counter(r["label"] for r in rows)
    print(f"wrote {args.out}: {len(rows)} rows")
    print(f"  by language: {dict(per_lang)}")
    print(f"  by label   : {dict(per_class)}")
    print(f"  unique problems: {len({r['problem_id'] for r in rows})}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
