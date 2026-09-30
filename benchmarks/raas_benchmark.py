#!/usr/bin/env python3
"""
RAAS-OCJS unified benchmark and test harness.

This single file replaces the Python-level *benchmarking* script that used to
live at benchmarks/run_codenet_benchmarks.py (corpus build, live runs, 10k macro
simulation, burst stress, CSV export).

It also overlaps with model-training/extract_codecontests.py, but does NOT
replace it: that script emits the stratified Light/Heavy *training* subset that
feeds XGBoost training, which is a different artifact from the benchmark corpus
built here (sources + real test cases + validation, used to drive the judge).
Keep both. `fetch` is this file's equivalent for benchmarking.

WHAT IT IS NOT
--------------
Two parts of the old testing surface are not Python and cannot be folded in:

  * feature-extraction-pipeline/src/bin/probe.rs  -> build with
        cargo build --bin probe        (cgroup/AST probe, Rust)
  * the unit + integration suites                 -> run with
        cargo test                     (in server/ and feature-extraction-pipeline/)

The `preflight` subcommand below checks both of those are runnable and reports
what it finds, so this file is still the single place you start from.

SUBCOMMANDS
-----------
  preflight   Check judge reachability, Docker runtime images, cgroup mode.
  fetch       Download CodeContests, select N submissions, validate each
              candidate solution against its own test cases, and save the
              corpus into the repo (benchmarks/dataset/).
  run         Submit the saved corpus to the live judge across all strategies
              and write per-run + per-strategy result CSVs.
  simulate    Re-run the 10,000-submission macro simulation and the
              500-submission freeze burst from the measured profiles.
  all         fetch (if needed) -> run -> simulate.
  status      Show what corpus and results are currently on disk.

METROLOGY RULES (do not break these)
------------------------------------
  * Memory is charged as RESERVED LIMIT, never peak RSS. A submission that
    started in the low tier is charged the tier size; one that started
    uncapped, or that was promoted mid-run, is charged the 2048 MiB worst-case
    ceiling, because `Tier::High` is `--memory 0` and carries no finite
    reservation. See alloc_for().
  * Allocation is taken from the judge's own `tier_started` / `tier_promoted`
    for each run, never from a corpus label. A corpus label would model an
    oracle classifier instead of the one under test.
  * Output files are always suffixed with the tier that produced them
    (`..._tier256.csv`). The old harness wrote unsuffixed names that held
    whichever tier ran last, which is how 128 MiB figures ended up being cited
    as the shipped configuration.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import heapq
import json
import os
import random
import re
import statistics
import subprocess
import sys
import time
from pathlib import Path

try:
    import requests
except ImportError:  # pragma: no cover - reported cleanly in preflight
    requests = None


# --------------------------------------------------------------------------- #
# Paths
# --------------------------------------------------------------------------- #

ROOT = Path(__file__).resolve().parent.parent
BENCH_DIR = ROOT / "benchmarks"
DATASET_DIR = BENCH_DIR / "dataset"
SOURCES_DIR = DATASET_DIR / "sources"
RESULTS_DIR = BENCH_DIR / "results"
MANIFEST_PATH = DATASET_DIR / "manifest.json"


# --------------------------------------------------------------------------- #
# Configuration knobs (environment-overridable, matching the judge's own
# constants where they must agree)
# --------------------------------------------------------------------------- #

# The judge binds 0.0.0.0:3000 with no auth. It is normally reached over the
# LAN from the machine that drives the benchmark; override for a local daemon.
JUDGE_URL = os.environ.get("JUDGE_URL", "http://192.168.0.111:3000")

# MUST match server/src/docker.rs LOW_TIER_MB (default 256) and
# HIGH_WATERMARK_PCT (70), or every derived figure is fiction.
LIGHT_TIER_MB = int(os.environ.get("LIGHT_TIER_MB", "256"))
HIGH_WATERMARK_PCT = 70

# The static baseline convention the paper compares against. This is a
# comparison convention, not a judge setting.
BASELINE_TIER_MB = 2048

# A promoted container is lifted to uncapped, not to a fixed tier. Charging it
# the baseline ceiling is deliberately conservative: it credits RAAS-OCJS with
# a ceiling the system does not actually impose.
PROMOTED_TIER_MB = BASELINE_TIER_MB

# Simulations are stochastic; seed them so a reported figure is reproducible.
SEED = int(os.environ.get("BENCH_SEED", "42"))

STRATEGIES = ["baseline", "predictive", "reactive", "hybrid"]

# Physical hardware calibration profile of the judge host (the laptop that runs
# the daemon). Used only for the simulated host-capacity figures.
HOST_SPECS = {
    "cpu_model": "13th Gen Intel Core i5-13420H",
    "cpu_cores": 8,
    "cpu_threads": 12,
    "p_cores": 4,
    "e_cores": 4,
    "cpu_max_mhz": 4600.0,
    "l3_cache_mb": 12.0,
    "total_ram_mb": 15360.0,   # 15 GiB usable physical RAM
    "os_reserved_mb": 1024.0,  # held back for the OS
    "os": "Linux (Fedora, cgroup v2)",
    "storage": "NVMe SSD",
}

# Cloud provisioning target for the theoretical projection.
CLOUD_INSTANCE = {
    "name": "AWS EC2 c6i.4xlarge",
    "vcpus": 16,
    "ram_gb": 32,
    "hourly_cost_usd": 0.68,
    "packing_fraction": 0.875,  # the ceiling the baseline row itself achieves
}

# CodeContests language enum -> judge language.
# The dataset has NO C and no language 5; C++ is 2, Java is 4, and Python
# appears twice (PYTHON is largely Python 2 sources, PYTHON3 is Python 3).
CC_LANG = {1: "python", 2: "cpp", 3: "python", 4: "java"}
CC_LANG_PREF = {3: 0, 1: 1, 2: 0, 4: 0}  # lower is preferred for ties

LANG_EXT = {"python": "py", "cpp": "cpp", "java": "java", "c": "c"}

# Default corpus shape: 100 submissions, weighted the way contest traffic is.
DEFAULT_MIX = {"cpp": 40, "python": 35, "java": 25}

# Test-case intake limits. The judge runs every case sequentially with a 10 s
# per-case wall-clock guard, so an unbounded case list makes a run unbounded.
MAX_CASES_PER_SUB = 5
MAX_CASE_BYTES = 64 * 1024
MAX_TOTAL_BYTES = 256 * 1024


# --------------------------------------------------------------------------- #
# Small helpers
# --------------------------------------------------------------------------- #

def log(msg: str) -> None:
    print(msg, flush=True)


def tier_suffix() -> str:
    """Output files are always tagged with the tier that produced them."""
    return f"tier{LIGHT_TIER_MB}"


def sanitize(name: str, limit: int = 64) -> str:
    """Filesystem- and Docker-safe identifier fragment."""
    s = re.sub(r"[^A-Za-z0-9]+", "_", str(name)).strip("_")
    return (s or "x")[:limit]


def ensure_dirs() -> None:
    for d in (DATASET_DIR, SOURCES_DIR, RESULTS_DIR):
        d.mkdir(parents=True, exist_ok=True)


def write_csv(path: Path, rows: list[dict]) -> None:
    if not rows:
        log(f"[WARN] nothing to write for {path.name}")
        return
    with open(path, "w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        w.writeheader()
        w.writerows(rows)
    log(f"[OUTPUT] {path.relative_to(ROOT)}  ({len(rows)} rows)")


def sha256_text(s: str) -> str:
    return hashlib.sha256(s.encode("utf-8", "ignore")).hexdigest()[:16]


# --------------------------------------------------------------------------- #
# Judge client
# --------------------------------------------------------------------------- #

def judge_url() -> str:
    return JUDGE_URL.rstrip("/")


def submit(payload: dict, timeout: float = 180.0) -> dict | None:
    """POST one submission. Returns the parsed result, or None on transport error."""
    if requests is None:
        raise RuntimeError("the `requests` package is required (pip install requests)")
    try:
        r = requests.post(f"{judge_url()}/submit", json=payload, timeout=timeout)
    except Exception as e:
        log(f"    [ERROR] submit failed: {e}")
        return None
    if r.status_code != 200:
        log(f"    [ERROR] HTTP {r.status_code}: {r.text[:200]}")
        return None
    try:
        return r.json()
    except Exception:
        log(f"    [ERROR] non-JSON response: {r.text[:200]}")
        return None


def check_judge(quiet: bool = False) -> bool:
    if requests is None:
        if not quiet:
            log("[FAIL] `requests` is not installed in this interpreter")
        return False
    try:
        r = requests.get(f"{judge_url()}/health", timeout=5)
        ok = r.status_code == 200 and r.json().get("status") == "OK"
    except Exception as e:
        if not quiet:
            log(f"[FAIL] judge unreachable at {judge_url()}: {e}")
        return False
    if ok and not quiet:
        log(f"[OK]   judge healthy at {judge_url()}")
    return ok


# --------------------------------------------------------------------------- #
# Source normalisation
# --------------------------------------------------------------------------- #

def normalize_java(code: str) -> str:
    """The runtime image compiles `/app/Main.java` and runs class `Main`.

    A source whose public class has any other name produces an opaque SE, so
    rewrite it. Every submission that reaches the judge has passed through here.
    """
    if "class Main" in code:
        return code
    c = re.sub(r"public\s+class\s+\w+", "public class Main", code, count=1)
    if "class Main" not in c:
        c = re.sub(r"class\s+\w+", "class Main", c, count=1)
    return c


# --------------------------------------------------------------------------- #
# Program-type classification (for corpus diversity reporting)
#
# A lightweight marker scan, not the Tree-sitter extractor: the point is to
# label the corpus so a reader can see what kinds of programs it contains, not
# to feed the classifier (the judge does that itself from the real AST).
# --------------------------------------------------------------------------- #

TYPE_RULES: list[tuple[str, tuple[str, ...]]] = [
    ("Graph / Trees", (
        r"\badjacency", r"\badj\b", r"\bgraph\b", r"\bbfs\b", r"\bdfs\b",
        r"dijkstra", r"bellman", r"\bqueue<int>", r"\bdeque", r"topolog",
        r"\bedge[s]?\b", r"\bvisited\b", r"floyd", r"kruskal", r"prim\b",
    )),
    ("DP / State Table", (
        r"\bdp\b", r"memo", r"\bknapsack\b", r"\bLCS\b", r"lis\b",
        r"\bcache\b", r"recursion_memo",
    )),
    ("Data Structures", (
        r"priority_queue", r"multiset", r"\bunordered_map", r"\bmap<", r"\bset<",
        r"segment tree", r"fenwick", r"\bbit\b", r"heapq", r"defaultdict",
        r"Counter", r"TreeMap", r"TreeSet", r"PriorityQueue", r"HashMap",
    )),
    ("Number Theory / Math", (
        r"\bgcd\b", r"\bsieve\b", r"\bprime", r"\bmodulo\b", r"factorial",
        r"\bpow\(|powm|binpow|fast_pow", r"phi\b", r"combinat",
    )),
    ("Strings", (
        r"\bstring\b", r"substr", r"\bchar\b", r"palindrom", r"KMP",
        r"z_function", r"\bhash\b",
    )),
    ("Greedy / Sorting", (
        r"\bsort\(|sorted\(|Arrays\.sort|Collections\.sort", r"cmp\b|comparator",
        r"greedy",
    )),
    ("Recursion / Backtracking", (
        r"\bdef\s+(\w+)\(", r"backtrack", r"permut", r"\bdfs\(",
    )),
    ("Bitmask / Bits", (
        r"1\s*<<", r">>\s*1", r"bitset", r"bit_count|popcount",
    )),
]

TAG_TO_TYPE = {
    "graphs": "Graph / Trees",
    "trees": "Graph / Trees",
    "dfs and similar": "Graph / Trees",
    "shortest paths": "Graph / Trees",
    "dp": "DP / State Table",
    "data structures": "Data Structures",
    "math": "Number Theory / Math",
    "number theory": "Number Theory / Math",
    "strings": "Strings",
    "greedy": "Greedy / Sorting",
    "sortings": "Greedy / Sorting",
    "binary search": "Greedy / Sorting",
    "brute force": "Brute Force / Simulation",
    "implementation": "Brute Force / Simulation",
    "bitmasks": "Bitmask / Bits",
}


def classify_program(source: str, tags: list[str]) -> str:
    for tag in tags or []:
        t = (tag or "").strip().lower()
        if t in TAG_TO_TYPE:
            return TAG_TO_TYPE[t]
    low = source.lower()
    for label, pats in TYPE_RULES:
        for p in pats:
            if re.search(p, source if p.isupper() else low):
                return label
    return "Brute Force / Simulation"


def structural_flags(source: str) -> dict:
    """A few structural markers, recorded for corpus diversity reporting.

    `has_recursion` is an approximation: a declared function name that also
    appears as a call site at least twice. Cheap and good enough for labelling;
    the judge's own Tree-sitter extractor is the authority on the real AST.
    """
    declared = re.findall(r"\bdef\s+(\w+)\s*\(", source)
    declared += re.findall(
        r"\b(?:int|void|long|double|bool|float|char|string|String)\s+(\w+)\s*\(", source)
    recursive = False
    for name in set(declared):
        if len(re.findall(r"\b" + re.escape(name) + r"\s*\(", source)) >= 2:
            recursive = True
            break
    return {
        "has_fast_io": bool(re.search(
            r"sync_with_stdio|cin\.tie|BufferedReader|StringTokenizer|sys\.stdin|scanf|getline\(cin",
            source)),
        "has_recursion": recursive,
        "has_loop": bool(re.search(r"\bfor\b|\bwhile\b", source)),
        "has_2d_indexing": bool(re.search(r"\[\s*\w+\s*\]\s*\[", source)),
        "source_lines": source.count("\n") + 1,
    }


# --------------------------------------------------------------------------- #
# Corpus: fetch from CodeContests and persist into the repo
# --------------------------------------------------------------------------- #

def pick_test_cases(problem: dict) -> list[dict]:
    """Choose a bounded, diverse set of test cases for one problem.

    Preference order: public tests first (they are the problem's own examples),
    then a sample of generated tests, which is where multi-case and larger
    inputs come from. Everything is capped so a single submission cannot make
    the suite run unboundedly long.
    """
    cases: list[dict] = []
    total = 0

    def add(inputs, outputs):
        nonlocal total
        for i, o in zip(inputs, outputs):
            if len(cases) >= MAX_CASES_PER_SUB:
                return
            if i is None or o is None:
                continue
            ib = len(i.encode("utf-8", "ignore"))
            ob = len(o.encode("utf-8", "ignore"))
            if ib == 0 or ob == 0:
                continue
            if ib > MAX_CASE_BYTES or ob > MAX_CASE_BYTES:
                continue
            if total + ib > MAX_TOTAL_BYTES:
                return
            cases.append({"input": i, "expected": o})
            total += ib

    for key in ("public_tests", "generated_tests", "private_tests"):
        block = problem.get(key) or {}
        add(block.get("input") or [], block.get("output") or [])
        if len(cases) >= MAX_CASES_PER_SUB:
            break
    return cases


def collect_candidates(want: dict[str, int], args) -> list[dict]:
    """Stream CodeContests and gather candidate submissions per language.

    Each problem contributes at most `--langs-per-problem` submissions (default
    3), so the same problem can appear once per language, but never twice. That
    is deliberate: the judge routes with a *per-language* model, so comparing
    languages on an identical problem is the only way to separate the language
    effect from the problem mix. It also means `--count 100` draws on fewer
    distinct problems than 100 - see the summary printed at the end of fetch.
    """
    from datasets import load_dataset

    quota = dict(want)
    picked: list[dict] = []
    seen_problems: set[str] = set()
    scanned = 0
    t0 = time.time()
    per_problem_limit = max(1, args.langs_per_problem)

    splits = args.splits.split(",")
    for split in splits:
        if not any(quota.values()):
            break
        log(f"\n[DATASET] Streaming deepmind/code_contests split={split!r} ...")
        ds = load_dataset("deepmind/code_contests", split=split, streaming=True)
        src_names = getattr(ds.features.get("source"), "names", None)
        diff_names = getattr(ds.features.get("difficulty"), "names", None)

        def label(value, names):
            """CodeContests encodes source/ difficulty as ClassLabel indices."""
            if value is None:
                return "UNKNOWN"
            if names and isinstance(value, int) and 0 <= value < len(names):
                return names[value]
            return str(value)

        for problem in ds:
            scanned += 1
            if args.max_problems and scanned > args.max_problems:
                log(f"[DATASET] hit --max-problems ({args.max_problems}); stopping scan")
                return picked
            if not any(quota.values()):
                return picked

            pname = problem.get("name") or f"prob_{split}_{scanned}"
            if pname in seen_problems:
                continue

            cases = pick_test_cases(problem)
            if not cases:
                continue

            sols = problem.get("solutions") or {}
            langs = sols.get("language") or []
            codes = sols.get("solution") or []

            # Group every candidate per language, best-language-variant first.
            by_lang: dict[str, list[str]] = {}
            order: dict[str, int] = {}
            for l_id, code in zip(langs, codes):
                lang = CC_LANG.get(l_id)
                if not lang or not code or len(code.strip()) <= 20:
                    continue
                if lang == "java":
                    code = normalize_java(code)
                pref = CC_LANG_PREF.get(l_id, 9)
                by_lang.setdefault(lang, []).append(code)
                order[lang] = min(order.get(lang, 99), pref)

            # Take up to `per_problem_limit` languages for this problem, always
            # draining the language with the most unfilled quota first so the
            # corpus stays balanced instead of finishing C++ before Java.
            taken_here = 0
            for lang in sorted(by_lang, key=lambda L: (order.get(L, 9), -quota.get(L, 0))):
                if taken_here >= per_problem_limit:
                    break
                if quota.get(lang, 0) <= 0:
                    continue
                tags = list(problem.get("cf_tags") or [])
                diff = problem.get("difficulty")
                ptype = classify_program(by_lang[lang][0], tags)
                picked.append({
                    "problem_name": pname,
                    "split": split,
                    "language": lang,
                    "candidates": by_lang[lang],
                    "test_cases": cases,
                    "source_platform": label(problem.get("source"), src_names),
                    "difficulty": int(diff) if diff is not None else -1,
                    "difficulty_label": label(problem.get("difficulty"), diff_names),
                    "cf_rating": int(problem.get("cf_rating") or 0),
                    "cf_tags": tags,
                    "program_type": ptype,
                    "declared_time_limit_s": (
                        (problem.get("time_limit") or {}).get("seconds")
                        if problem.get("time_limit") else None),
                    "declared_memory_limit_bytes": int(problem.get("memory_limit_bytes") or 0),
                })
                quota[lang] -= 1
                taken_here += 1
                if not any(quota.values()):
                    break
            if taken_here:
                seen_problems.add(pname)

            if scanned % 250 == 0:
                log(f"  scanned {scanned} problems, selected {len(picked)} "
                    f"({time.time()-t0:.0f}s)  remaining {quota}")

    return picked


def validate_candidate(problem_name: str, lang: str, candidates: list[str],
                       cases: list[dict], max_try: int) -> tuple[str | None, int]:
    """Return the first candidate that the judge accepts under the baseline tier.

    CodeContests is crowd-sourced: a non-trivial fraction of its solutions do
    not compile or compute the wrong answer. Left in, those surface as RE/SE
    verdicts that have nothing to do with the scheduling policy under test.
    """
    # Validate on EVERY case the run will use, not a subset. Validating on only
    # the smallest public cases lets solutions through that fail CodeContests'
    # generated tests, which then shows up as WA in the results - noise about
    # solution correctness rather than about the scheduling policy. Pass
    # --validate-cases N to weaken this deliberately.
    n = args_global["validate_cases"]
    probe_cases = (sorted(cases, key=lambda c: len(c["input"]))[:n] if n > 0 else cases)

    for idx, cand in enumerate(candidates[:max_try]):
        payload = {
            "id": f"validate_{sanitize(problem_name, 32)}_{lang}_{idx}",
            "language": lang,
            "approach": "baseline",           # uncapped: isolate correctness
            "source": cand,
            "test_cases": probe_cases,
        }
        t0 = time.time()
        res = submit(payload, timeout=args_global["validate_timeout"])
        if res is None:
            continue
        verdict = res.get("verdict")
        log(f"    [VALIDATE] {problem_name}/{lang} cand {idx}: {verdict} "
            f"({time.time()-t0:.1f}s)")
        if verdict == "AC":
            return cand, idx + 1
    return None, min(len(candidates), max_try)


args_global: dict = {"validate_cases": 0, "validate_timeout": 120.0}


def cmd_fetch(args) -> int:
    ensure_dirs()
    # The sources dir is generated: clear it so a re-fetch cannot leave orphan
    # files behind that are not in the manifest (harmless but confusing, and
    # they make "N sources == N submissions" stop being true).
    stale = [p for p in SOURCES_DIR.iterdir()
             if p.is_file() and p.suffix.lstrip(".") in set(LANG_EXT.values())]
    for p in stale:
        p.unlink()
    if stale:
        log(f"[CLEAN] removed {len(stale)} stale source file(s) from "
            f"{SOURCES_DIR.relative_to(ROOT)}")
    if args.with_synthetic:
        log("[NOTE] --with-synthetic appends hand-written programs to the corpus. "
            "They are labelled origin='synthetic' and are NOT from CodeContests.")

    if args.languages:
        want = {}
        for part in args.languages.split(","):
            part = part.strip().lower()
            if part == "c":
                log("[WARN] CodeContests contains no C submissions (its language enum "
                    "is PYTHON / CPP / PYTHON3 / JAVA). Use --with-synthetic for C.")
                continue
            if part in DEFAULT_MIX:
                want[part] = DEFAULT_MIX[part]
        # Rescale to the requested total.
        scale = args.count / max(1, sum(want.values()))
        want = {k: max(1, round(v * scale)) for k, v in want.items()}
    else:
        scale = args.count / 100.0
        want = {k: max(1, round(v * scale)) for k, v in DEFAULT_MIX.items()}
    log(f"[CORPUS] target mix: {want} (total {sum(want.values())})")

    # Validation rejects a real fraction of candidates (CodeContests is
    # crowd-sourced), so collect more than the target and stop once enough
    # have been validated. Without this the corpus silently comes up short.
    collect_want = {k: max(1, round(v * args.overshoot)) for k, v in want.items()}
    log(f"[CORPUS] collecting {sum(collect_want.values())} candidates "
        f"(overshoot x{args.overshoot}) to yield {args.count} validated")
    candidates = collect_candidates(collect_want, args)
    log(f"\n[DATASET] {len(candidates)} candidate submissions across "
        f"{len(set(c['problem_name'] for c in candidates))} problems")

    if args.with_synthetic:
        candidates.extend(synthetic_programs())

    validate = not args.no_validate
    if validate and not check_judge():
        log("[WARN] judge unreachable - saving every candidate unvalidated. "
            "Expect RE/SE noise in the results. Re-run `fetch` once the judge is up.")
        validate = False

    args_global["validate_cases"] = args.validate_cases
    args_global["validate_timeout"] = args.validate_timeout

    submissions: list[dict] = []
    rejected = 0
    for i, cand in enumerate(candidates, 1):
        if len(submissions) >= args.count:
            log(f"\n[CORPUS] reached {args.count} validated submissions; "
                f"stopping with {len(candidates) - i + 1} candidates unused")
            break
        lang = cand["language"]
        source = cand["candidates"][0]
        tries = 0
        if validate:
            log(f"[{i}/{len(candidates)}] {lang:6s} {cand['problem_name'][:52]}")
            source, tries = validate_candidate(
                cand["problem_name"], lang, cand["candidates"],
                cand["test_cases"], args.max_candidates)
            if source is None:
                rejected += 1
                log(f"    [SKIP] no candidate passed for {lang} {cand['problem_name']}")
                continue

        sub_id = f"cc_{sanitize(cand['problem_name'], 40)}_{lang}"
        if any(s["submission_id"] == sub_id for s in submissions):
            sub_id = f"{sub_id}_{i}"
        filename = f"{sub_id}.{LANG_EXT[lang]}"
        (SOURCES_DIR / filename).write_text(source, encoding="utf-8")

        flags = structural_flags(source)
        submissions.append({
            "submission_id": sub_id,
            "problem_name": cand["problem_name"],
            "problem_id": sanitize(cand["problem_name"], 40),
            "split": cand["split"],
            "language": lang,
            "file": f"sources/{filename}",
            "origin": cand.get("origin", "code_contests"),
            "source_platform": cand["source_platform"],
            "difficulty": cand["difficulty"],
            "difficulty_label": cand.get("difficulty_label", "UNKNOWN"),
            "cf_rating": cand["cf_rating"],
            "cf_tags": cand["cf_tags"],
            "program_type": cand["program_type"],
            "declared_time_limit_s": cand["declared_time_limit_s"],
            "declared_memory_limit_bytes": cand["declared_memory_limit_bytes"],
            "candidate_tried": tries,
            "source_sha256_16": sha256_text(source),
            "source_chars": len(source),
            **flags,
            "num_cases": len(cand["test_cases"]),
            "input_bytes": sum(len(c["input"].encode()) for c in cand["test_cases"]),
            "test_cases": cand["test_cases"],
        })

    manifest = {
        "meta": {
            "created_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "dataset": "deepmind/code_contests",
            "splits": args.splits,
            "requested_count": args.count,
            "saved_count": len(submissions),
            "rejected_by_validation": rejected,
            "validated": validate,
            "seed": SEED,
            "max_cases_per_submission": MAX_CASES_PER_SUB,
            "max_candidates_tried": args.max_candidates,
            "with_synthetic": bool(args.with_synthetic),
            "judge_url_used_for_validation": judge_url() if validate else None,
        },
        "submissions": submissions,
    }
    MANIFEST_PATH.write_text(json.dumps(manifest, indent=1), encoding="utf-8")

    log(f"\n[DATASET] saved {len(submissions)} submissions "
        f"({rejected} rejected by validation) -> {MANIFEST_PATH.relative_to(ROOT)}")
    summarise_corpus(manifest)
    return 0 if submissions else 1


def summarise_corpus(manifest: dict) -> None:
    subs = manifest["submissions"]
    if not subs:
        return
    def tally(key):
        c: dict[str, int] = {}
        for s in subs:
            c[str(s[key])] = c.get(str(s[key]), 0) + 1
        return dict(sorted(c.items(), key=lambda kv: -kv[1]))
    log("\n  by language      : " + ", ".join(f"{k}={v}" for k, v in tally("language").items()))
    log("  by program type  : " + ", ".join(f"{k}={v}" for k, v in tally("program_type").items()))
    log("  by platform      : " + ", ".join(f"{k}={v}" for k, v in tally("source_platform").items()))
    log("  by difficulty    : " + ", ".join(f"{k}={v}" for k, v in tally("difficulty_label").items()))
    log("  by origin        : " + ", ".join(f"{k}={v}" for k, v in tally("origin").items()))
    cases = [s["num_cases"] for s in subs]
    log(f"  cases/submission : min={min(cases)} max={max(cases)} mean={statistics.mean(cases):.1f}")
    probs = [s["problem_name"] for s in subs]
    uniq = sorted(set(probs))
    per_prob: dict[str, int] = {}
    for p in probs:
        per_prob[p] = per_prob.get(p, 0) + 1
    log(f"  problems         : {len(uniq)} unique across {len(subs)} submissions "
        f"(max {max(per_prob.values())} languages on one problem)")
    ib = [s["input_bytes"] for s in subs]
    log(f"  input bytes      : min={min(ib)} max={max(ib)} mean={statistics.mean(ib):.0f} "
        f"total={sum(ib)/1024:.0f} KB")
    flags = [k for k in ("has_fast_io", "has_recursion", "has_loop", "has_2d_indexing")
             if k in subs[0]]
    if flags:
        log("  structure        : " + ", ".join(
            f"{k}={sum(1 for s in subs if s.get(k))}" for k in flags))


# --------------------------------------------------------------------------- #
# Optional synthetic supplement
#
# CodeContests solutions are all small algorithmic programs: none of them
# allocate enough to cross the 179.2 MiB soft watermark, so a corpus made only
# from it never exercises live promotion. These hand-written programs exist to
# cover that path (and the C language, which CodeContests lacks). They are
# labelled origin='synthetic' and must never be counted as dataset results.
# --------------------------------------------------------------------------- #

def synthetic_programs() -> list[dict]:
    def mk(pid, name, ptype, lang, src, exp):
        return {
            "problem_name": pid,
            "problem_id": pid,
            "split": "synthetic",
            "language": lang,
            "program_type": ptype,
            "source_platform": "SYNTHETIC",
            "difficulty": -1,
            "difficulty_label": "SYNTHETIC",
            "cf_rating": 0,
            "cf_tags": [],
            "origin": "synthetic",
            "declared_time_limit_s": None,
            "declared_memory_limit_bytes": 0,
            "candidates": [src.strip()],
            "test_cases": [{"input": "", "expected": exp}],
        }

    heavy_cpp = """
#include <iostream>
#include <vector>
using namespace std;
const int N = 2000;
const int W = 26500;
int dp[N][W];
int main() {
    ios_base::sync_with_stdio(false);
    cin.tie(NULL);
    for (int i = 1; i < N; i++) {
        int weight = (i * 13) % 50 + 1;
        int val = (i * 17) % 100 + 1;
        for (int w = 0; w < W; w++) {
            dp[i][w] = dp[i - 1][w];
            if (w >= weight) {
                int cand = dp[i - 1][w - weight] + val;
                if (cand > dp[i][w]) dp[i][w] = cand;
            }
        }
    }
    cout << "DP Optimal: " << dp[N - 1][W - 1] << "\\n";
    return 0;
}
"""
    heavy_py = """
import sys
def solve():
    N = 2000
    W = 26500
    byte_table = bytearray(215 * 1024 * 1024)
    for i in range(0, len(byte_table), 4096):
        byte_table[i] = (i % 251)
    total = sum(byte_table[::100000])
    print("DP Optimal Python:", total % 10000)
if __name__ == "__main__":
    solve()
"""
    heavy_java = """
public class Main {
    public static void main(String[] args) {
        int chunks = 210;
        byte[][] matrix = new byte[chunks][1024 * 1024];
        for (int i = 0; i < chunks; i++) {
            for (int j = 0; j < 1024 * 1024; j += 4096) {
                matrix[i][j] = (byte)((i + j) % 127);
            }
        }
        long sum = 0;
        for (int i = 0; i < chunks; i++) {
            sum += matrix[i][0];
        }
        System.out.println("DP Optimal Java: " + sum);
    }
}
"""
    heavy_c = """
#include <stdio.h>
#include <stdlib.h>
#define N 2000
#define W 26500
int main() {
    size_t sz = (size_t)N * W * sizeof(int);
    int *table = (int *)malloc(sz);
    if (!table) return 1;
    for (size_t i = 0; i < (size_t)N * W; i += 1024) {
        table[i] = (int)(i % 10007);
    }
    long long sum = 0;
    for (size_t i = 0; i < (size_t)N * W; i += 50000) {
        sum += table[i];
    }
    printf("DP Optimal C: %lld\\n", sum % 1000000007LL);
    free(table);
    return 0;
}
"""
    light_c = """
#include <stdio.h>
#include <stdlib.h>
int main() {
    int n = 1000000;
    long long *arr = (long long *)malloc(sizeof(long long) * (n + 1));
    long long *pref = (long long *)malloc(sizeof(long long) * (n + 1));
    if (!arr || !pref) return 1;
    pref[0] = 0;
    for (int i = 1; i <= n; i++) {
        arr[i] = (i * 37LL) % 10007;
        pref[i] = pref[i - 1] + arr[i];
    }
    long long sum = 0;
    for (int i = 1; i <= 1000; i++) {
        int l = (i * 17) % n + 1;
        int r = (i * 31) % n + 1;
        if (l > r) { int t = l; l = r; r = t; }
        sum += (pref[r] - pref[l - 1]);
    }
    printf("%lld\\n", sum % 1000000007LL);
    free(arr);
    free(pref);
    return 0;
}
"""
    cpu_c = """
#include <stdio.h>
#define N 300
#define INF 1000000000
int dist[N][N];
int main() {
    for (int i = 0; i < N; i++)
        for (int j = 0; j < N; j++)
            dist[i][j] = (i == j) ? 0 : ((i * 31 + j * 17) % 1000 + 1);
    for (int k = 0; k < N; k++)
        for (int i = 0; i < N; i++)
            for (int j = 0; j < N; j++)
                if (dist[i][k] + dist[k][j] < dist[i][j])
                    dist[i][j] = dist[i][k] + dist[k][j];
    printf("Done: %d\\n", dist[0][N-1]);
    return 0;
}
"""
    return [
        mk("S1_knapsack_heavy", "Synthetic 0-1 Knapsack 2D DP", "DP / State Table", "cpp", heavy_cpp, "DP Optimal: 82619\n"),
        mk("S1_knapsack_heavy", "Synthetic 0-1 Knapsack 2D DP", "DP / State Table", "python", heavy_py, "DP Optimal Python: 612\n"),
        mk("S1_knapsack_heavy", "Synthetic 0-1 Knapsack 2D DP", "DP / State Table", "java", heavy_java, "DP Optimal Java: 11404\n"),
        mk("S1_knapsack_heavy", "Synthetic 0-1 Knapsack 2D DP", "DP / State Table", "c", heavy_c, "DP Optimal C: 85633\n"),
        mk("S2_prefix_light", "Synthetic Prefix Sums (Light)", "Data Structures", "c", light_c, "61444859\n"),
        mk("S3_floyd_cpu", "Synthetic Floyd-Warshall (CPU-bound)", "Graph / Trees", "c", cpu_c, "Done: 23\n"),
    ]


# --------------------------------------------------------------------------- #
# Corpus: load from disk
# --------------------------------------------------------------------------- #

def load_corpus() -> list[dict]:
    if not MANIFEST_PATH.exists():
        log(f"[FAIL] no corpus at {MANIFEST_PATH.relative_to(ROOT)} - run `fetch` first")
        return []
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    corpus = []
    for s in manifest["submissions"]:
        src_path = DATASET_DIR / s["file"]
        if not src_path.exists():
            log(f"[WARN] missing source file for {s['submission_id']}: {s['file']}")
            continue
        s = dict(s)
        s["source"] = src_path.read_text(encoding="utf-8")
        corpus.append(s)
    return corpus


# --------------------------------------------------------------------------- #
# Live runs against the judge
# --------------------------------------------------------------------------- #

def alloc_for(tier_started: str, tier_promoted: bool) -> tuple[float, float, int]:
    """Memory (MiB), CPU cores and CFS shares charged for one submission.

    Derived from the judge's OWN decision, never from a corpus label; see the
    metrology note at the top of this file.
    """
    if tier_promoted:
        return float(PROMOTED_TIER_MB), 2.0, 2048
    if tier_started == "low":
        return float(LIGHT_TIER_MB), 1.0, 1024
    return float(BASELINE_TIER_MB), 2.0, 2048


def cmd_run(args) -> int:
    ensure_dirs()
    if not check_judge():
        return 1
    corpus = load_corpus()
    if not corpus:
        return 1
    if args.limit:
        corpus = corpus[: args.limit]
    strategies = [s.strip() for s in args.strategies.split(",") if s.strip()]

    total = len(corpus) * len(strategies)
    log(f"\n=== LIVE RUN: {len(corpus)} submissions x {len(strategies)} strategies "
        f"= {total} submissions ===")
    log(f"    judge={judge_url()}  tier={LIGHT_TIER_MB} MiB  "
        f"watermark={LIGHT_TIER_MB * HIGH_WATERMARK_PCT // 100} MiB")

    rows: list[dict] = []
    current = 0
    failures = 0
    t_start = time.time()

    for prog in corpus:
        for strat in strategies:
            current += 1
            sub_id = f"{prog['submission_id']}_{strat}"
            payload = {
                "id": sub_id,
                "language": prog["language"],
                "source": prog["source"],
                "test_cases": prog["test_cases"],
                "approach": strat,
            }
            t0 = time.perf_counter()
            res = submit(payload, timeout=args.timeout)
            e2e_ms = (time.perf_counter() - t0) * 1000.0
            if res is None:
                failures += 1
                log(f"[{current:3d}/{total}] {sub_id}: no response")
                continue

            verdict = res.get("verdict", "UNKNOWN")
            tier_started = res.get("tier_started", "low")
            tier_promoted = bool(res.get("tier_promoted", False))
            peak_bytes = res.get("peak_memory_bytes", 0) or 0
            used_mb = peak_bytes / (1024.0 * 1024.0)
            alloc_mb, cores, shares = alloc_for(tier_started, tier_promoted)
            wasted_mb = max(0.0, alloc_mb - used_mb)

            rows.append({
                "submission_id": sub_id,
                "problem_id": prog["problem_id"],
                "problem_name": prog["problem_name"],
                "origin": prog["origin"],
                "program_type": prog["program_type"],
                "source_platform": prog["source_platform"],
                "difficulty": prog["difficulty"],
                "language": prog["language"].upper(),
                "strategy": strat.capitalize(),
                "verdict": verdict,
                "num_cases": prog["num_cases"],
                "input_bytes": prog["input_bytes"],
                "tier_started": tier_started,
                "tier_promoted": tier_promoted,
                "promotion_time_ms": res.get("promotion_time_ms", 0),
                "allocated_mb": round(alloc_mb, 2),
                "used_mb": round(used_mb, 2),
                "wasted_mb": round(wasted_mb, 2),
                "wasted_pct": round((wasted_mb / alloc_mb) * 100.0, 2),
                "allocated_cpu_cores": cores,
                "cpu_shares": shares,
                "cpu_time_ms": res.get("cpu_time_ms", 0),
                "container_wall_ms": res.get("wall_time_ms", 0),
                "e2e_request_to_verdict_ms": round(e2e_ms, 2),
            })

            if current % 10 == 0 or verdict != "AC":
                log(f"[{current:3d}/{total}] {prog['language'].upper():6s} "
                    f"{strat:10s} {prog['problem_id'][:26]:26s} {verdict:4s} "
                    f"used={used_mb:7.1f}MB start={tier_started:4s} "
                    f"prom={str(tier_promoted):5s} e2e={e2e_ms:7.1f}ms")

            time.sleep(args.spacing)

    if not rows:
        log("[FAIL] no results collected")
        return 1

    elapsed = time.time() - t_start
    raw = RESULTS_DIR / f"real_dataset_empirical_runs_{tier_suffix()}.csv"
    write_csv(raw, rows)

    # Verdict tally is the first thing anyone should read.
    tally: dict[str, int] = {}
    for r in rows:
        tally[r["verdict"]] = tally.get(r["verdict"], 0) + 1
    log("\n  verdicts: " + ", ".join(f"{k}={v}" for k, v in sorted(tally.items())))
    log(f"  promotions: {sum(1 for r in rows if r['tier_promoted'])}/{len(rows)}")
    log(f"  transport failures: {failures}")
    log(f"  wall clock: {elapsed:.0f}s for {len(rows)} submissions")
    return 0


# --------------------------------------------------------------------------- #
# Macro contest simulation (10,000 submissions from measured profiles)
# --------------------------------------------------------------------------- #

def load_empirical() -> list[dict]:
    path = RESULTS_DIR / f"real_dataset_empirical_runs_{tier_suffix()}.csv"
    if not path.exists():
        log(f"[FAIL] no empirical runs at {path.relative_to(ROOT)} - run `run` first")
        return []
    with open(path, newline="", encoding="utf-8") as f:
        rows = list(csv.DictReader(f))
    for r in rows:
        for k in ("cpu_time_ms", "container_wall_ms", "e2e_request_to_verdict_ms",
                  "used_mb", "allocated_mb", "wasted_mb", "wasted_pct",
                  "allocated_cpu_cores", "promotion_time_ms", "difficulty",
                  "num_cases", "input_bytes"):
            try:
                r[k] = float(r[k])
            except (KeyError, TypeError, ValueError):
                r[k] = 0.0
        r["tier_promoted"] = str(r.get("tier_promoted", "")).strip().lower() in ("true", "1", "yes")
    return rows


def dispatch_sim(sub, start_time, wait_ms, strat, prof_dict, events, completed):
    key = (sub["problem_id"], sub["language"], strat)
    prof = prof_dict.get(key)
    if not prof:
        # Fall back to any profile for the same problem, then to anything.
        prof = next((p for (pid, _l, _s), p in prof_dict.items()
                     if pid == sub["problem_id"]), None)
    if not prof:
        prof = next(iter(prof_dict.values()))

    jitter = random.uniform(0.97, 1.03)
    wall_ms = prof["container_wall_ms"] * jitter
    e2e_ms = prof["e2e_request_to_verdict_ms"] * jitter
    cpu_ms = prof["cpu_time_ms"] * jitter
    used_mb = prof["used_mb"] * jitter
    alloc_mb, cores, _shares = alloc_for(
        prof.get("tier_started", "low"), prof.get("tier_promoted", False))
    wasted_mb = max(0.0, alloc_mb - used_mb)

    completed.append({
        "submission_id": sub["sub_id"],
        "problem_id": sub["problem_id"],
        "language": sub["language"].upper(),
        "strategy": strat.capitalize(),
        "arrival_s": round(sub["arrival_s"], 2),
        "wait_ms": round(wait_ms, 2),
        "cpu_time_ms": round(cpu_ms, 2),
        "container_wall_ms": round(wall_ms, 2),
        "e2e_request_to_verdict_ms": round(e2e_ms, 2),
        "turnaround_ms": round(wait_ms + e2e_ms, 2),
        "allocated_mb": round(alloc_mb, 2),
        "used_mb": round(used_mb, 2),
        "wasted_mb": round(wasted_mb, 2),
        "wasted_pct": round((wasted_mb / alloc_mb) * 100.0, 2),
        "allocated_cpu_cores": cores,
        "tier_promoted": prof.get("tier_promoted", False),
    })
    heapq.heappush(events, (start_time + wall_ms / 1000.0, "departure", None))


def simulate_queue(subs, slots, strat, prof_dict):
    """Discrete-event queue: bounded concurrency, FIFO wait, measured service."""
    completed: list[dict] = []
    available = slots
    waiting: list[dict] = []
    events: list[tuple] = []
    for s in subs:
        heapq.heappush(events, (s["arrival_s"], "arrival", s))
    while events:
        t, kind, data = heapq.heappop(events)
        if kind == "departure":
            available += 1
            if waiting:
                nxt = waiting.pop(0)
                available -= 1
                dispatch_sim(nxt, t, (t - nxt["arrival_s"]) * 1000.0,
                             strat, prof_dict, events, completed)
        else:
            if available > 0:
                available -= 1
                dispatch_sim(data, t, 0.0, strat, prof_dict, events, completed)
            else:
                waiting.append(data)
    return completed


def macro_simulation(rows: list[dict]) -> dict[str, list[dict]]:
    random.seed(SEED)
    N = 10000
    CONTEST_SECS = 7200.0
    log(f"\n=== MACRO CONTEST SIMULATION (N={N:,}, seeded {SEED}) ===")

    prof_dict = {(r["problem_id"], r["language"].lower(), r["strategy"].lower()): r
                 for r in rows}
    problems = sorted(set(r["problem_id"] for r in rows if r["origin"] == "code_contests"))
    heavy = sorted(set(r["problem_id"] for r in rows if r["origin"] == "synthetic"))
    if not problems:
        problems = sorted(set(r["problem_id"] for r in rows))
    log(f"  sampled from {len(problems)} real problems"
        + (f" + {len(heavy)} synthetic" if heavy else ""))

    # Arrivals: opening rush, mid-contest lull, scoreboard-freeze rush.
    arrivals = []
    t = 0.0
    for i in range(N):
        progress = i / N
        rate = 2.5 if progress < 0.125 else (3.0 if progress > 0.85 else 1.05)
        t += random.expovariate(rate)
        arrivals.append(t)
    scale = CONTEST_SECS / arrivals[-1]
    arrivals = [a * scale for a in arrivals]

    lang_weights = [("cpp", 0.50), ("python", 0.30), ("java", 0.15), ("c", 0.05)]
    langs, weights = zip(*lang_weights)
    subs = []
    for i in range(N):
        if heavy and random.random() < 0.20:
            pid = random.choice(heavy)
        else:
            pid = random.choice(problems)
        lang = random.choices(langs, weights=weights)[0]
        # Only submit a language that actually has a profile for this problem.
        if (pid, lang, "baseline") not in prof_dict:
            lang = next((L for L in langs if (pid, L, "baseline") in prof_dict), "cpp")
        subs.append({"sub_id": f"sub_{i+1:05d}", "arrival_s": arrivals[i],
                     "problem_id": pid, "language": lang})

    all_completed: dict[str, list[dict]] = {}
    # Admission counts are derived from the host and the tier, never hardcoded:
    # this is what makes the 7 -> 56 slot claim track the real tier size.
    usable = HOST_SPECS["total_ram_mb"] - HOST_SPECS["os_reserved_mb"]
    adaptive_slots = max(1, int(usable // LIGHT_TIER_MB))
    baseline_slots = max(1, int(usable // BASELINE_TIER_MB))

    for strat in STRATEGIES:
        slots = baseline_slots if strat == "baseline" else adaptive_slots
        done = simulate_queue(subs, slots, strat, prof_dict)
        all_completed[strat] = done
        alloc_gb = sum(c["allocated_mb"] for c in done) / 1024.0
        used_gb = sum(c["used_mb"] for c in done) / 1024.0
        wasted = (sum(c["wasted_mb"] for c in done)
                  / max(1e-9, sum(c["allocated_mb"] for c in done))) * 100.0
        proms = sum(1 for c in done if c["tier_promoted"])
        log(f"  {strat.upper():10s} slots={slots:3d} | reserved={alloc_gb:9.1f} GB "
            f"| used={used_gb:7.1f} GB | wasted={wasted:5.1f}% | promos={proms}")
    return all_completed


def export_summaries(all_completed: dict[str, list[dict]]) -> None:
    suffix = tier_suffix()
    strategies = STRATEGIES
    base = all_completed["baseline"]
    N = len(base)
    base_alloc_gb = sum(c["allocated_mb"] for c in base) / 1024.0

    # 1. Global strategy summary
    strat_rows = []
    for strat in strategies:
        data = all_completed[strat]
        alloc_gb = sum(c["allocated_mb"] for c in data) / 1024.0
        used_gb = sum(c["used_mb"] for c in data) / 1024.0
        wasted_gb = sum(c["wasted_mb"] for c in data) / 1024.0
        saved_gb = base_alloc_gb - alloc_gb if strat != "baseline" else 0.0
        waits = [c["wait_ms"] for c in data]
        turns = [c["turnaround_ms"] for c in data]
        e2e = [c["e2e_request_to_verdict_ms"] for c in data]
        proms = sum(1 for c in data if c["tier_promoted"])
        core_hrs = sum(c["allocated_cpu_cores"] * (c["container_wall_ms"] / 1000.0) / 3600.0
                       for c in data)
        strat_rows.append({
            "Strategy": strat.capitalize(),
            "Total_Submissions": N,
            "Total_Allocated_GB": round(alloc_gb, 2),
            "Total_Used_GB": round(used_gb, 2),
            "Total_Wasted_GB": round(wasted_gb, 2),
            "Wasted_Percentage": round((wasted_gb / alloc_gb) * 100.0, 2),
            "Memory_Saved_vs_Baseline_GB": round(saved_gb, 2),
            "Memory_Savings_Pct": round((saved_gb / base_alloc_gb) * 100.0, 2) if strat != "baseline" else 0.0,
            "Total_CPU_Core_Hours": round(core_hrs, 3),
            "Avg_Queue_Wait_ms": round(statistics.mean(waits), 2),
            "Avg_E2E_Latency_ms": round(statistics.mean(e2e), 2),
            "Avg_Turnaround_ms": round(statistics.mean(turns), 2),
            "P95_Turnaround_ms": round(statistics.quantiles(turns, n=100)[94], 2),
            "Live_Promotions": proms,
            "Promotion_Rate_Pct": round(proms / N * 100.0, 2),
            "Tier_MiB": LIGHT_TIER_MB,
        })
    write_csv(RESULTS_DIR / f"real_dataset_strategy_summary_{suffix}.csv", strat_rows)

    # 2. Per-language breakdown
    lang_rows = []
    for lang in sorted(set(c["language"] for c in base)):
        base_lang = [c for c in base if c["language"] == lang]
        base_lang_alloc = sum(c["allocated_mb"] for c in base_lang) / 1024.0
        for strat in strategies:
            d = [c for c in all_completed[strat] if c["language"] == lang]
            if not d:
                continue
            alloc_gb = sum(c["allocated_mb"] for c in d) / 1024.0
            used_gb = sum(c["used_mb"] for c in d) / 1024.0
            wasted_gb = sum(c["wasted_mb"] for c in d) / 1024.0
            saved_gb = base_lang_alloc - alloc_gb if strat != "baseline" else 0.0
            turns = [c["turnaround_ms"] for c in d]
            proms = sum(1 for c in d if c["tier_promoted"])
            lang_rows.append({
                "Language": lang,
                "Strategy": strat.capitalize(),
                "Submissions": len(d),
                "Share_Pct": round(len(d) / N * 100.0, 1),
                "Total_Allocated_GB": round(alloc_gb, 2),
                "Total_Used_GB": round(used_gb, 2),
                "Total_Wasted_GB": round(wasted_gb, 2),
                "Wasted_Percentage": round((wasted_gb / alloc_gb) * 100.0, 2),
                "Memory_Saved_vs_Baseline_GB": round(saved_gb, 2),
                "Memory_Savings_Pct": round((saved_gb / base_lang_alloc) * 100.0, 2) if strat != "baseline" else 0.0,
                "Avg_Allocated_Cores": round(statistics.mean([c["allocated_cpu_cores"] for c in d]), 2),
                "Total_Core_Hours": round(sum(c["allocated_cpu_cores"] * (c["container_wall_ms"] / 1000.0) / 3600.0 for c in d), 3),
                "Avg_CPU_Time_ms": round(statistics.mean([c["cpu_time_ms"] for c in d]), 1),
                "Avg_Container_Wall_ms": round(statistics.mean([c["container_wall_ms"] for c in d]), 1),
                "Avg_E2E_Request_To_Verdict_ms": round(statistics.mean([c["e2e_request_to_verdict_ms"] for c in d]), 1),
                "P95_Turnaround_ms": round(statistics.quantiles(turns, n=100)[94], 1),
                "Live_Promotions": proms,
                "Promotion_Rate_Pct": round(proms / len(d) * 100.0, 1),
                "Tier_MiB": LIGHT_TIER_MB,
            })
    write_csv(RESULTS_DIR / f"real_dataset_language_metrics_{suffix}.csv", lang_rows)

    # 3. Cloud provisioning projection. Every figure is DERIVED from the tier
    #    size, the instance shape and its price - nothing is transcribed.
    inst_ram_mb = CLOUD_INSTANCE["ram_gb"] * 1024
    usable = int(inst_ram_mb * CLOUD_INSTANCE["packing_fraction"])
    price = CLOUD_INSTANCE["hourly_cost_usd"]
    base_pods = usable // BASELINE_TIER_MB
    adapt_pods = usable // LIGHT_TIER_MB
    burst = 500
    base_vms = -(-burst // base_pods)
    adapt_vms = -(-burst // adapt_pods)
    base_cost = base_vms * price
    adapt_cost = adapt_vms * price
    reactive_row = next((r for r in strat_rows if r["Strategy"] == "Reactive"), strat_rows[-1])
    cloud_rows = [
        {"Provisioning_Dimension": "Default Per-Pod Memory Reservation",
         "Static_Baseline_Cloud": f"{BASELINE_TIER_MB} MiB",
         "RAAS_OCJS_Adaptive_Cloud": f"{LIGHT_TIER_MB} MiB",
         "Cloud_Efficiency_Gain": f"{BASELINE_TIER_MB/LIGHT_TIER_MB:.1f}x reduction in baseline pod memory"},
        {"Provisioning_Dimension": "Default Per-Pod CPU Reservation",
         "Static_Baseline_Cloud": "2.0 vCPUs", "RAAS_OCJS_Adaptive_Cloud": "1.0 vCPU",
         "Cloud_Efficiency_Gain": "2.0x reduction in baseline CPU reservation"},
        {"Provisioning_Dimension": f"Max Pod Packing Density ({CLOUD_INSTANCE['name']}, {CLOUD_INSTANCE['ram_gb']} GB)",
         "Static_Baseline_Cloud": f"{base_pods} concurrent pods",
         "RAAS_OCJS_Adaptive_Cloud": f"{adapt_pods} concurrent pods",
         "Cloud_Efficiency_Gain": f"{adapt_pods/base_pods:.1f}x higher container density per VM"},
        {"Provisioning_Dimension": f"VM Fleet Size for {burst}-Sub Burst",
         "Static_Baseline_Cloud": f"{base_vms} VMs ({base_pods} slots each)",
         "RAAS_OCJS_Adaptive_Cloud": f"{adapt_vms} VMs ({adapt_pods}+ slots each)",
         "Cloud_Efficiency_Gain": f"{(base_vms-adapt_vms)/base_vms*100.0:.1f}% reduction in active cloud VMs"},
        {"Provisioning_Dimension": f"Cluster Hourly Cost (AWS @ USD {price:.2f}/hr)",
         "Static_Baseline_Cloud": f"USD {base_cost:.2f} / hour",
         "RAAS_OCJS_Adaptive_Cloud": f"USD {adapt_cost:.2f} / hour",
         "Cloud_Efficiency_Gain": f"USD {base_cost-adapt_cost:.2f} / hour savings ({(base_cost-adapt_cost)/base_cost*100.0:.1f}% cost cut)"},
        {"Provisioning_Dimension": "Total Contest RAM Reserved (10,000 Subs, Reactive)",
         "Static_Baseline_Cloud": f"{strat_rows[0]['Total_Allocated_GB']} GB",
         "RAAS_OCJS_Adaptive_Cloud": f"{reactive_row['Total_Allocated_GB']} GB",
         "Cloud_Efficiency_Gain": f"{reactive_row['Memory_Saved_vs_Baseline_GB']} GB reclaimed ({reactive_row['Memory_Savings_Pct']}% savings)"},
    ]
    write_csv(RESULTS_DIR / f"real_dataset_cloud_projection_{suffix}.csv", cloud_rows)

    log(f"\n  NOTE: baseline uses {PROMOTED_TIER_MB} MiB worst-case per pod and the "
        f"adaptive column uses the {LIGHT_TIER_MB} MiB tier. The "
        f"{LIGHT_TIER_MB} MiB column is only deployable if the tier experiment "
        f"holds at that size.")


def burst_stress(rows: list[dict]) -> None:
    """500-submission scoreboard-freeze burst on the calibration host."""
    random.seed(SEED)
    N, WINDOW = 500, 30.0
    log(f"\n=== FREEZE BURST STRESS (N={N}, {WINDOW:.0f}s window, host "
        f"{HOST_SPECS['total_ram_mb']/1024:.0f} GiB) ===")
    prof_dict = {(r["problem_id"], r["language"].lower(), r["strategy"].lower()): r
                 for r in rows}
    problems = sorted(set(r["problem_id"] for r in rows if r["origin"] == "code_contests"))
    heavy = sorted(set(r["problem_id"] for r in rows if r["origin"] == "synthetic"))
    if not problems:
        problems = sorted(set(r["problem_id"] for r in rows))

    subs = []
    for i in range(N):
        pid = random.choice(heavy) if (heavy and random.random() < 0.20) else random.choice(problems)
        lang = random.choices(["cpp", "python", "java", "c"], weights=[0.50, 0.30, 0.15, 0.05])[0]
        if (pid, lang, "baseline") not in prof_dict:
            lang = next((L for L in ("cpp", "python", "java", "c")
                         if (pid, L, "baseline") in prof_dict), "cpp")
        subs.append({"sub_id": f"burst_{i+1:04d}",
                     "arrival_s": random.uniform(0.0, WINDOW),
                     "problem_id": pid, "language": lang})
    subs.sort(key=lambda s: s["arrival_s"])

    usable = HOST_SPECS["total_ram_mb"] - HOST_SPECS["os_reserved_mb"]
    adaptive = max(1, int(usable // LIGHT_TIER_MB))
    safe_baseline = max(1, int(usable // BASELINE_TIER_MB))
    log(f"  slots: baseline safe={safe_baseline}, baseline 2x overcommit={safe_baseline*2}, "
        f"adaptive=floor({usable:.0f}/{LIGHT_TIER_MB})={adaptive}")

    scenarios = [("Baseline (Safe)", "baseline", safe_baseline),
                 ("Baseline (Overcommitted)", "baseline", safe_baseline * 2)]
    for strat in ("predictive", "reactive", "hybrid"):
        scenarios.append((f"{strat.capitalize()} (Adaptive {adaptive})", strat, adaptive))

    out = []
    for label, strat, slots in scenarios:
        done = simulate_queue(subs, slots, strat, prof_dict)
        waits = [c["wait_ms"] for c in done]
        turns = [c["turnaround_ms"] for c in done]
        peak = slots * (BASELINE_TIER_MB if strat == "baseline" else LIGHT_TIER_MB)
        drain = max(c["arrival_s"] + c["turnaround_ms"] / 1000.0 for c in done)
        row = {
            "Scenario": label, "Strategy": strat.capitalize(), "Concurrent_Slots": slots,
            "Peak_Allocated_RAM_MB": peak,
            "Host_RAM_Utilization_Pct": round(peak / HOST_SPECS["total_ram_mb"] * 100.0, 1),
            "Avg_Queue_Wait_ms": round(statistics.mean(waits), 1),
            "P95_Queue_Wait_ms": round(statistics.quantiles(waits, n=100)[94], 1),
            "Avg_E2E_Latency_ms": round(statistics.mean([c["e2e_request_to_verdict_ms"] for c in done]), 1),
            "Avg_Turnaround_ms": round(statistics.mean(turns), 1),
            "P95_Turnaround_ms": round(statistics.quantiles(turns, n=100)[94], 1),
            "Burst_Drain_Time_s": round(drain, 1),
            "Tier_MiB": LIGHT_TIER_MB,
        }
        out.append(row)
        log(f"  {label:32s} slots={slots:3d} | wait={row['Avg_Queue_Wait_ms']:9.1f} ms "
            f"| P95 turn={row['P95_Turnaround_ms']:9.1f} ms | drain={row['Burst_Drain_Time_s']:5.1f}s")
    write_csv(RESULTS_DIR / f"real_dataset_burst_stress_{tier_suffix()}.csv", out)


def cmd_simulate(args) -> int:
    rows = load_empirical()
    if not rows:
        return 1
    ensure_dirs()
    completed = macro_simulation(rows)
    export_summaries(completed)
    burst_stress(rows)
    return 0


# --------------------------------------------------------------------------- #
# Preflight / status
# --------------------------------------------------------------------------- #

def cmd_preflight(args) -> int:
    log("=== RAAS-OCJS PREFLIGHT ===")
    rc = 0

    log(f"\n[1] Python deps")
    try:
        import datasets  # noqa: F401
        log("    OK   datasets importable")
    except ImportError as e:
        log(f"    FAIL datasets: {e}  -> python3 -m venv .venv && .venv/bin/pip install -r model-training/requirements.txt")
        rc = 1
    if requests is None:
        log("    FAIL requests not installed")
        rc = 1
    else:
        log("    OK   requests importable")

    log(f"\n[2] Judge at {judge_url()}")
    if not check_judge():
        log("    (start it with: cd server && sudo ./target/debug/server)")
        rc = 1

    log(f"\n[3] Docker runtime images")
    for img in ("python-judge-runtime", "cpp-judge-runtime", "java-judge-runtime"):
        try:
            out = subprocess.run(["docker", "images", "-q", img],
                                 capture_output=True, text=True, timeout=30)
            if out.stdout.strip():
                log(f"    OK   {img}")
            else:
                log(f"    FAIL {img} missing -> docker build -t {img} server/runtimes/<lang>")
                rc = 1
        except Exception as e:
            log(f"    FAIL docker not usable: {e}")
            rc = 1
            break

    log(f"\n[4] Host cgroup mode (judge needs cgroup v2 + a reachable host dir)")
    try:
        fs = subprocess.run(["stat", "-fc", "%T", "/sys/fs/cgroup"],
                            capture_output=True, text=True, timeout=10).stdout.strip()
        log(f"    /sys/fs/cgroup is {fs}" + ("  (cgroup v2)" if fs == "cgroup2fs" else "  (NOT unified)"))
        if fs != "cgroup2fs":
            rc = 1
    except Exception as e:
        log(f"    could not stat cgroup fs: {e}")

    log(f"\n[5] Rust test suites (cannot be folded into this Python file)")
    for d in ("server", "feature-extraction-pipeline"):
        log(f"    cargo test --manifest-path {d}/Cargo.toml")
    log("    cargo run --bin probe --manifest-path feature-extraction-pipeline/Cargo.toml")

    log(f"\n[6] Corpus / results on disk")
    if MANIFEST_PATH.exists():
        m = json.loads(MANIFEST_PATH.read_text())
        log(f"    corpus: {m['meta']['saved_count']} submissions "
            f"(created {m['meta']['created_utc']}, validated={m['meta']['validated']})")
    else:
        log("    no corpus yet -> run `fetch`")
    have = sorted(p.name for p in RESULTS_DIR.glob("*.csv")) if RESULTS_DIR.exists() else []
    log(f"    results: {len(have)} CSV(s)" + (f" {have}" if have else ""))

    log(f"\n[{'OK' if rc == 0 else 'ISSUES FOUND'}] preflight complete")
    return rc


def cmd_status(args) -> int:
    log("=== RAAS-OCJS BENCHMARK STATUS ===")
    log(f"judge           : {judge_url()}  ({'healthy' if check_judge(quiet=True) else 'UNREACHABLE'})")
    log(f"tier            : {LIGHT_TIER_MB} MiB "
        f"(watermark {LIGHT_TIER_MB * HIGH_WATERMARK_PCT // 100} MiB)   seed={SEED}")
    if MANIFEST_PATH.exists():
        m = json.loads(MANIFEST_PATH.read_text())
        log(f"corpus          : {m['meta']['saved_count']} submissions  "
            f"validated={m['meta']['validated']}  splits={m['meta']['splits']}")
        summarise_corpus(m)
    else:
        log("corpus          : none (run `fetch`)")
    if RESULTS_DIR.exists():
        for p in sorted(RESULTS_DIR.glob("*.csv")):
            with open(p, newline="", encoding="utf-8") as f:
                n = sum(1 for _ in f) - 1
            log(f"  {p.name:52s} {n:>6} rows")
    return 0


# --------------------------------------------------------------------------- #
# CLI
# --------------------------------------------------------------------------- #

def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="raas_benchmark.py",
        description="RAAS-OCJS unified benchmark + test harness "
                    "(supersedes run_codenet_benchmarks.py and extract_codecontests.py)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    sub = p.add_subparsers(dest="cmd", required=True)

    f = sub.add_parser("fetch", help="build the corpus from CodeContests into the repo")
    f.add_argument("--count", type=int, default=100, help="target submissions (default 100)")
    f.add_argument("--splits", default="train", help="comma list: train,valid,test")
    f.add_argument("--languages", default="", help="subset e.g. cpp,python,java (default: weighted all)")
    f.add_argument("--max-problems", type=int, default=0, help="stop scanning after N problems (0 = until quota met)")
    f.add_argument("--max-candidates", type=int, default=4, help="candidate solutions tried per submission")
    f.add_argument("--overshoot", type=float, default=1.4,
                   help="collect this multiple of the target, since validation rejects some")
    f.add_argument("--langs-per-problem", type=int, default=3,
                   help="max languages taken from one problem (default 3)")
    f.add_argument("--validate-cases", type=int, default=0,
                   help="cases used to validate a candidate (0 = all, the default)")
    f.add_argument("--validate-timeout", type=float, default=120.0)
    f.add_argument("--no-validate", action="store_true", help="skip judge validation (fast, noisy)")
    f.add_argument("--with-synthetic", action="store_true",
                   help="append hand-written heavy/C programs (needed to exercise promotion)")
    f.set_defaults(func=cmd_fetch)

    r = sub.add_parser("run", help="submit the saved corpus to the judge")
    r.add_argument("--strategies", default=",".join(STRATEGIES))
    r.add_argument("--limit", type=int, default=0, help="only the first N submissions (smoke test)")
    r.add_argument("--timeout", type=float, default=180.0, help="per-request timeout (s)")
    r.add_argument("--spacing", type=float, default=0.05, help="pause between submissions (s)")
    r.set_defaults(func=cmd_run)

    s = sub.add_parser("simulate", help="macro contest + burst simulation from measured profiles")
    s.set_defaults(func=cmd_simulate)

    a = sub.add_parser("all", help="fetch (if needed) -> run -> simulate")
    a.add_argument("--count", type=int, default=100)
    a.add_argument("--splits", default="train")
    a.add_argument("--languages", default="")
    a.add_argument("--max-problems", type=int, default=0)
    a.add_argument("--max-candidates", type=int, default=4)
    a.add_argument("--overshoot", type=float, default=1.4)
    a.add_argument("--langs-per-problem", type=int, default=3)
    a.add_argument("--validate-cases", type=int, default=0)
    a.add_argument("--validate-timeout", type=float, default=120.0)
    a.add_argument("--with-synthetic", action="store_true")
    a.add_argument("--no-validate", action="store_true")
    a.add_argument("--limit", type=int, default=0)
    a.add_argument("--strategies", default=",".join(STRATEGIES))
    a.add_argument("--timeout", type=float, default=180.0)
    a.add_argument("--spacing", type=float, default=0.05)
    a.set_defaults(func=None)  # handled in main

    sub.add_parser("preflight", help="check judge, images, cgroup mode, deps").set_defaults(func=cmd_preflight)
    sub.add_parser("status", help="show corpus and results on disk").set_defaults(func=cmd_status)
    return p


def main(argv=None) -> int:
    args = build_parser().parse_args(argv)
    if args.cmd == "all":
        if not MANIFEST_PATH.exists():
            args_global["validate_cases"] = 0
            args_global["validate_timeout"] = 120.0
            if cmd_fetch(args) != 0:
                return 1
        if cmd_run(args) != 0:
            return 1
        return cmd_simulate(args)
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
