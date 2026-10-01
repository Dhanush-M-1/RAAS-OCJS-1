#!/usr/bin/env python3
"""
Promotion stress suite for RAAS-OCJS.

The other benchmark subcommands measure the corpus we happen to have. This one
does the opposite: it *authors* submissions to probe the promotion mechanism
itself, sweeping allocation size across the 179.2 MiB soft watermark so that
every band (light / medium / borderline / heavy / extreme) is exercised by a
program whose demand we chose rather than inherited.

Two allocation styles are used, because they test different things:

  gradual  allocates and touches in 8 MiB chunks with a short pause between
           them. The watermark is crossed while the program is still running, so
           the promotion path gets the time it needs to complete. This measures
           whether promotion WORKS.

  instant  allocates and touches everything in one pass. The watermark is
           crossed as fast as the kernel can fault pages, so promotion has to
           win a race against the allocation itself. This measures the promotion
           WINDOW, which is the failure mode the paper attributes to the
           128 MiB tier.

Usage:
    RAAS_AUTH_TOKEN=... JUDGE_URL=http://localhost:3000 \
        python3 benchmarks/promotion_suite.py --timeout 180
"""

from __future__ import annotations

import argparse
import csv
import json
import os
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

JUDGE_URL = os.environ.get("JUDGE_URL", "http://localhost:3000")
AUTH_TOKEN = os.environ.get("RAAS_AUTH_TOKEN", "")
HEADERS = {"x-raas-token": AUTH_TOKEN} if AUTH_TOKEN else {}

RESULTS = Path(__file__).resolve().parent / "results"
OUT_CSV = RESULTS / "promotion_suite.csv"

WATERMARK_MIB = 179.2          # 70% of the 256 MiB tier
TIER_MIB = 256

# --------------------------------------------------------------------------- #
# Source generators
# --------------------------------------------------------------------------- #

def py_source(target_mib: int, style: str) -> str:
    """Allocate exact MiB and touch every page, via stdin-supplied size."""
    if style == "gradual":
        return (
            "import sys, time\n"
            "n = int(sys.stdin.readline())\n"
            "chunk = 8 * 1024 * 1024\n"
            "buf = []\n"
            "for _ in range(n * 1024 * 1024 // chunk):\n"
            "    b = bytearray(chunk)\n"
            "    for i in range(0, chunk, 4096):\n"
            "        b[i] = 1\n"
            "    buf.append(b)\n"
            "    time.sleep(0.12)\n"
            "print('ok')\n"
        )
    return (
        "import sys\n"
        "n = int(sys.stdin.readline())\n"
        "b = bytearray(n * 1024 * 1024)\n"
        "for i in range(0, len(b), 4096):\n"
        "    b[i] = 1\n"
        "print('ok')\n"
    )


def cpp_source(target_mib: int, style: str) -> str:
    if style == "gradual":
        return (
            "#include <cstdio>\n"
            "#include <cstring>\n"
            "#include <cstdlib>\n"
            "#include <unistd.h>\n"
            "#include <vector>\n"
            "int main(){ long n; if(scanf(\"%ld\", &n)!=1) return 1;\n"
            "  const size_t CH = 8u<<20; std::vector<char*> bufs;\n"
            "  for(long i=0;i<n;i+=8){ char* p=(char*)malloc(CH);\n"
            "    memset(p,1,CH); bufs.push_back(p); usleep(120000); }\n"
            "  printf(\"ok\\n\"); return 0; }\n"
        )
    return (
        "#include <cstdio>\n"
        "#include <cstring>\n"
        "#include <cstdlib>\n"
        "int main(){ long n; if(scanf(\"%ld\", &n)!=1) return 1;\n"
        "  size_t sz = (size_t)n*(1u<<20); char* p=(char*)malloc(sz);\n"
        "  if(!p) return 2; memset(p,1,sz); printf(\"ok\\n\"); return 0; }\n"
    )


def java_source(target_mib: int, style: str) -> str:
    # Allocates into the JVM heap. Note the JVM is launched with a fixed -Xmx by
    # the judge, so large targets can fail before a single byte is written.
    if style == "gradual":
        return (
            "import java.util.*;\n"
            "public class Main{ public static void main(String[] a) throws Exception{\n"
            "  Scanner s=new Scanner(System.in); int n=s.nextInt();\n"
            "  List<byte[]> l=new ArrayList<>();\n"
            "  for(int i=0;i<n;i+=8){ byte[] b=new byte[8*1024*1024];\n"
            "    Arrays.fill(b,(byte)1); l.add(b); Thread.sleep(120); }\n"
            "  System.out.println(\"ok\"); } }\n"
        )
    return (
        "import java.util.*;\n"
        "public class Main{ public static void main(String[] x) throws Exception{\n"
        "  Scanner s=new Scanner(System.in); int n=s.nextInt();\n"
        "  byte[] b=new byte[n*1024*1024]; Arrays.fill(b,(byte)1);\n"
        "  System.out.println(\"ok\"); } }\n"
    )


GENERATORS = {"python": py_source, "cpp": cpp_source, "java": java_source}

# --------------------------------------------------------------------------- #
# The matrix
# --------------------------------------------------------------------------- #
# (name, language, target MiB, style, note)
CASES: list[tuple[str, str, int, str, str]] = [
    # --- band: light, far below the watermark ---------------------------------
    ("l005_py",  "python",   5, "instant", "light: trivial footprint"),
    ("l020_py",  "python",  20, "instant", "light: upper edge of the common case"),
    # --- band: medium, below the watermark but no longer trivial --------------
    ("m040_py",  "python",  40, "gradual", "medium: well under watermark"),
    ("m060_py",  "python",  60, "gradual", "medium"),
    ("m080_py",  "python",  80, "gradual", "medium: approaching watermark"),
    # --- band: borderline, straddling 179.2 MiB -------------------------------
    ("b170_py",  "python", 170, "gradual", "just UNDER watermark - must NOT promote"),
    ("b185_py",  "python", 185, "gradual", "just OVER watermark - must promote"),
    ("b200_py",  "python", 200, "gradual", "over watermark"),
    # --- band: heavy, exceeds the tier itself ---------------------------------
    ("h250_py",  "python", 250, "gradual", "near tier ceiling - needs promotion to survive"),
    ("h400_py",  "python", 400, "gradual", "well beyond tier - promotion is mandatory"),
    ("h900_py",  "python", 900, "gradual", "far beyond tier"),
    # --- window race: instant allocation at the same sizes --------------------
    ("h250i_py", "python", 250, "instant", "instant: does promotion win the race?"),
    ("h400i_py", "python", 400, "instant", "instant: window stress"),
    # --- C++: separate allocator and no JIT/heap reservation ------------------
    ("m080_cpp", "cpp",     80, "gradual", "medium, compiled language"),
    ("b185_cpp", "cpp",    185, "gradual", "over watermark, compiled language"),
    ("h400_cpp", "cpp",    400, "gradual", "beyond tier, compiled language"),
    ("h400i_cpp","cpp",    400, "instant", "instant, compiled language"),
    # --- Java: fixed -Xmx at launch interacts with the container cap ----------
    ("m080_java","java",    80, "gradual", "medium, JVM"),
    ("h400_java","java",   400, "gradual", "beyond tier, JVM (-Xmx interaction)"),
]


def submit(payload: dict, timeout: float) -> dict | None:
    req = urllib.request.Request(
        f"{JUDGE_URL}/submit",
        data=json.dumps(payload).encode(),
        headers={"content-type": "application/json", **HEADERS},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            return json.loads(r.read())
    except urllib.error.HTTPError as e:
        body = e.read().decode(errors="replace")[:200]
        print(f"    HTTP {e.code}: {body}", file=sys.stderr)
        return None
    except Exception as e:  # noqa: BLE001
        print(f"    transport: {type(e).__name__}: {e}", file=sys.stderr)
        return None


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--timeout", type=float, default=180.0)
    ap.add_argument("--only", default="", help="substring filter on case name")
    args = ap.parse_args()

    RESULTS.mkdir(parents=True, exist_ok=True)

    cases = [c for c in CASES if args.only in c[0]] if args.only else CASES

    print(f"=== PROMOTION STRESS SUITE ({len(cases)} cases) ===")
    print(f"    judge      : {JUDGE_URL}")
    print(f"    tier       : {TIER_MIB} MiB, watermark {WATERMARK_MIB} MiB")
    print(f"    auth token : {'set' if AUTH_TOKEN else 'NONE (may 401)'}")
    print()

    rows = []
    for i, (name, lang, mib, style, note) in enumerate(cases, 1):
        src = GENERATORS[lang](mib, style)
        payload = {
            "id": f"promo_{name}",
            "language": lang,
            "approach": "reactive",
            "source": src,
            "test_cases": [{"input": str(mib), "expected": "ok"}],
        }
        t0 = time.perf_counter()
        res = submit(payload, args.timeout)
        dt = time.perf_counter() - t0

        if res is None:
            print(f"[{i:>2}/{len(cases)}] {name:<11} {lang:<7} {mib:>4} MiB {style:<8} "
                  f"NO RESPONSE")
            rows.append({
                "case": name, "language": lang, "target_mib": mib, "style": style,
                "note": note, "verdict": "NO_RESPONSE", "tier_started": "",
                "tier_promoted": "", "promotion_time_ms": "", "peak_memory_bytes": "",
                "allocated_memory_bytes": "", "cpu_time_ms": "", "wall_time_ms": "",
                "e2e_ms": round(dt * 1000, 1),
            })
            continue

        r = {
            "case": name, "language": lang, "target_mib": mib, "style": style,
            "note": note,
            "verdict": res.get("verdict", ""),
            "tier_started": res.get("tier_started", ""),
            "tier_promoted": res.get("tier_promoted", ""),
            "promotion_time_ms": res.get("promotion_time_ms", ""),
            "peak_memory_bytes": res.get("peak_memory_bytes", ""),
            "allocated_memory_bytes": res.get("allocated_memory_bytes", ""),
            "cpu_time_ms": res.get("cpu_time_ms", ""),
            "wall_time_ms": res.get("wall_time_ms", ""),
            "e2e_ms": round(dt * 1000, 1),
        }
        rows.append(r)

        peak_mib = (res.get("peak_memory_bytes") or 0) / (1024 * 1024)
        print(f"[{i:>2}/{len(cases)}] {name:<11} {lang:<7} {mib:>4} MiB {style:<8} "
              f"{r['verdict']:<4} start={r['tier_started']:<4} "
              f"prom={str(r['tier_promoted']):<5} "
              f"prom_ms={str(r['promotion_time_ms']):<6} "
              f"peak={peak_mib:>7.1f} MiB")

    with open(OUT_CSV, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        w.writeheader()
        w.writerows(rows)

    print()
    print(f"[OUTPUT] {OUT_CSV}  ({len(rows)} rows)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
