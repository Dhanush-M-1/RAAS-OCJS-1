#!/usr/bin/env python3
"""Measure peak memory of submitted programs, to replace heuristic labels.

Why this exists: the Light/Heavy labels in `codecontests_subset/` are derived
from source length and problem difficulty (see `extract_codecontests.py`), not
from anything the program actually does at runtime. That makes the label a proxy
for "long source", so a model trained on it is scored against a proxy rather than
against memory behaviour. This harness produces the real thing: peak RSS actually
observed while running each program on its own test inputs.

Measurement method: each program is run once per test case in a fresh child
process, and peak RSS is read from `os.wait4` rusage (`ru_maxrss`, KiB on Linux).
The child is capped with RLIMIT_AS and RLIMIT_CPU so a runaway submission cannot
take the host down - this machine has little headroom.

Usage:
    ./.venv/bin/python measure_memory.py --limit 20 --workers 4
    ./.venv/bin/python measure_memory.py --out measured_memory.csv

Java is skipped by default: `javac` is not installed on this host, so Java must
be measured through the judge's container runtime instead (`--include-java` will
attempt it and report if the toolchain is missing).
"""
from __future__ import annotations

import argparse
import csv
import json
import os
import resource
import shutil
import subprocess
import sys
import tempfile
import time
from concurrent.futures import ProcessPoolExecutor, as_completed

HERE = os.path.dirname(os.path.abspath(__file__))
DEFAULT_MANIFEST = os.path.join(HERE, "..", "benchmarks", "dataset", "manifest.json")

ADDR_CAP_BYTES = 512 * 1024 * 1024   # safety net; real programs here use far less
CPU_CAP_SECONDS = 20
SRC_EXT = {"cpp": ".cpp", "c": ".c", "python": ".py", "java": ".java"}


def _limits():
    """Cap the child so a runaway submission cannot exhaust the host."""
    resource.setrlimit(resource.RLIMIT_AS, (ADDR_CAP_BYTES, ADDR_CAP_BYTES))
    resource.setrlimit(resource.RLIMIT_CPU, (CPU_CAP_SECONDS, CPU_CAP_SECONDS))
    os.setsid()


def build(language: str, src: str, workdir: str, timeout: int = 120):
    """Compile if needed. Returns (argv, error). Python runs the source directly."""
    if language == "python":
        return [sys.executable, src], None
    if language in ("cpp", "c"):
        exe = os.path.join(workdir, "prog.bin")
        compiler = "g++" if language == "cpp" else "gcc"
        std = "-std=c++17" if language == "cpp" else "-std=c11"
        p = subprocess.run([compiler, "-O2", std, "-o", exe, src],
                           capture_output=True, text=True, timeout=timeout)
        if p.returncode != 0:
            return None, f"compile failed: {p.stderr.strip()[:200]}"
        return [exe], None
    if language == "java":
        if shutil.which("javac") is None:
            return None, "javac not installed on this host"
        p = subprocess.run(["javac", "-d", workdir, src],
                           capture_output=True, text=True, timeout=timeout)
        if p.returncode != 0:
            return None, f"javac failed: {p.stderr.strip()[:200]}"
        cls = os.path.splitext(os.path.basename(src))[0]
        return ["java", "-cp", workdir, cls], None
    return None, f"unsupported language: {language}"


def run_one(argv, stdin_data: str, timeout_s: float):
    """Run one test case; return (peak_kb, wall_s, note)."""
    started = time.time()
    try:
        p = subprocess.Popen(argv, stdin=subprocess.PIPE,
                             stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                             preexec_fn=_limits)
    except OSError as e:
        return 0, 0.0, f"spawn failed: {e}"

    try:
        p.stdin.write(stdin_data.encode())
        p.stdin.close()
    except (BrokenPipeError, OSError):
        pass

    peak_kb, note = 0, ""
    deadline = started + timeout_s
    while True:
        pid, _status, ru = os.wait4(p.pid, os.WNOHANG)
        if pid != 0:
            peak_kb = ru.ru_maxrss
            break
        if time.time() > deadline:
            try:
                os.killpg(os.getpgid(p.pid), 9)
            except OSError:
                p.kill()
            _pid, _st, ru = os.wait4(p.pid, 0)
            peak_kb = ru.ru_maxrss
            note = "timeout"
            break
        time.sleep(0.004)

    p.returncode = 0   # reaped manually above; keep Popen quiet
    return peak_kb, time.time() - started, note


def measure_submission(sub) -> dict:
    """Compile once, run every test case, keep the peak across cases."""
    language = sub.get("language", "")
    sid = sub.get("submission_id", "?")
    result = {"submission_id": sid, "language": language,
              "cases_run": 0, "peak_rss_kb": 0, "total_wall_s": 0.0,
              "status": "ok", "note": ""}

    # manifest "file" is relative to the dataset directory, e.g. sources/<id>.cpp
    src_path = os.path.join(HERE, "..", "benchmarks", "dataset", sub.get("file", ""))
    if not os.path.isfile(src_path):
        result.update(status="missing", note=f"no source at {src_path}")
        return result

    cases = sub.get("test_cases") or []
    if not cases:
        result.update(status="no_cases")
        return result

    # Generous: the declared limit is a judge limit, not a measurement limit.
    timeout_s = max(5.0, float(sub.get("declared_time_limit_s") or 2) * 3)

    workdir = tempfile.mkdtemp(prefix="measure_")
    try:
        if language == "java":
            # Run from the source's directory so the class name resolves.
            argv, err = build(language, src_path, workdir)
        else:
            argv, err = build(language, src_path, workdir)
        if err:
            result.update(status="build_failed", note=err)
            return result

        peak = 0
        for case in cases:
            kb, wall, note = run_one(argv, case.get("input", ""), timeout_s)
            result["cases_run"] += 1
            result["total_wall_s"] += wall
            peak = max(peak, kb)
            if note:
                result["note"] = note
        result["peak_rss_kb"] = peak
        if peak >= ADDR_CAP_BYTES // 1024 - 4096:
            result.update(status="capped", note="hit RLIMIT_AS ceiling")
    except Exception as e:                                    # noqa: BLE001
        result.update(status="error", note=f"{type(e).__name__}: {e}"[:200])
    finally:
        shutil.rmtree(workdir, ignore_errors=True)
    return result


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--manifest", default=DEFAULT_MANIFEST)
    ap.add_argument("--out", default=os.path.join(HERE, "measured_memory.csv"))
    ap.add_argument("--limit", type=int, default=0, help="0 = all submissions")
    ap.add_argument("--workers", type=int, default=4)
    ap.add_argument("--include-java", action="store_true")
    args = ap.parse_args()

    with open(args.manifest, encoding="utf-8") as fh:
        manifest = json.load(fh)
    subs = manifest.get("submissions", [])
    if not args.include_java:
        subs = [s for s in subs if s.get("language") != "java"]
    if args.limit:
        subs = subs[:args.limit]

    print(f"measuring {len(subs)} submissions with {args.workers} workers")
    print(f"  address-space cap {ADDR_CAP_BYTES // (1024*1024)} MiB, "
          f"CPU cap {CPU_CAP_SECONDS}s per case")

    rows, started = [], time.time()
    with ProcessPoolExecutor(max_workers=args.workers) as pool:
        futures = {pool.submit(measure_submission, s): s for s in subs}
        for i, fut in enumerate(as_completed(futures), 1):
            r = fut.result()
            rows.append(r)
            if i % 5 == 0 or i == len(subs):
                print(f"  [{i}/{len(subs)}] {r['submission_id'][:40]:40s} "
                      f"{r['status']:12s} peak={r['peak_rss_kb']/1024:8.1f} MiB")

    fields = ["submission_id", "language", "peak_rss_kb", "cases_run",
              "total_wall_s", "status", "note"]
    with open(args.out, "w", newline="", encoding="utf-8") as fh:
        w = csv.DictWriter(fh, fieldnames=fields)
        w.writeheader()
        w.writerows(rows)

    ok = [r for r in rows if r["status"] == "ok"]
    print(f"\nwrote {args.out}")
    print(f"  measured {len(ok)}/{len(rows)} in {time.time()-started:.0f}s")
    if ok:
        peaks = sorted(r["peak_rss_kb"] / 1024 for r in ok)
        print(f"  peak RSS MiB: min {peaks[0]:.1f}  median {peaks[len(peaks)//2]:.1f}  "
              f"p95 {peaks[int(len(peaks)*0.95)]:.1f}  max {peaks[-1]:.1f}")
        for lang in sorted({r["language"] for r in ok}):
            lp = sorted(r["peak_rss_kb"]/1024 for r in ok if r["language"] == lang)
            print(f"    {lang:7s} n={len(lp):4d} median {lp[len(lp)//2]:7.1f} MiB  max {lp[-1]:7.1f} MiB")
    weird = [r for r in rows if r["status"] != "ok"]
    if weird:
        from collections import Counter
        print(f"  non-ok: {dict(Counter(r['status'] for r in weird))}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
