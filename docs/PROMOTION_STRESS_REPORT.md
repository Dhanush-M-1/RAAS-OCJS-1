# Promotion Stress Report

**What this is:** a purpose-built test of the promotion mechanism, run against the
deployed judge on Google Cloud Platform (`e2-standard-4`, `asia-south1`), plus the
bug it found and the fix that followed.

The existing benchmark measures whatever corpus we happen to have. That corpus
turned out to contain **no memory-heavy submissions at all** (max 47.8 MiB), so it
could never exercise promotion. This suite inverts that: it *authors* submissions
whose demand we choose, sweeping allocation size across the soft watermark.

Reproduce with:

```bash
export JUDGE_URL=http://localhost:3000 RAAS_AUTH_TOKEN=<token>
python3 benchmarks/promotion_suite.py --timeout 180
```

---

## 1. The test matrix

19 submissions, 5 MiB to 900 MiB, across the 179.2 MiB watermark (70% of the
256 MiB tier), in three languages and two allocation styles.

**Allocation style is the important axis.** It separates two different questions:

- **gradual** — 8 MiB chunks with a 120 ms pause. The watermark is crossed while
  the program still runs, so promotion has time to complete. This asks *does
  promotion work?*
- **instant** — one allocation pass. Promotion must race the allocation itself.
  This asks *does the promotion window hold?*

| Band | Targets | Why |
|---|---|---|
| light | 5, 20 MiB | the common case; must not promote |
| medium | 40, 60, 80 MiB | below the watermark, not trivial |
| borderline | 170, 185, 200 MiB | straddles 179.2 MiB — 170 must not promote, 185 must |
| heavy | 250, 400, 900 MiB | exceeds the tier; promotion is mandatory to survive |

---

## 2. The bug

### 2.1 Symptom

Submissions needing more than the tier died at **exactly** the tier ceiling while
the judge reported `tier_promoted: true`:

```json
{"verdict":"MLE","peak_memory_bytes":268435456,
 "tier_promoted":true,"promotion_time_ms":3315}
```

`268435456` bytes is exactly 256 MiB. The container hit the cap *after* being
promoted.

### 2.2 Root cause

The promotion path did two things in sequence, and the second undid the first:

```rust
let _ = cg.promote_to_unlimited();          // writes "max" to memory.high/max  ✓
let _ = Command::new("docker")              // ← THIS REVERTS IT
    .args(["update", container, "--memory", "0",
           "--memory-swap", "-1", "--cpus", "0"])
    .output().await;
*promoted = true;                           // recorded regardless of outcome
```

For `docker update`, `--memory 0` does **not** mean "unlimited". Docker reads `0`
as *"no change"*, re-reconciles the container's cgroup against its configured
limit, and silently discards the promotion.

Measured directly on the host, one step at a time:

| Step | `memory.max` |
|---|---|
| container created (`--memory=256m`) | `268435456` |
| arm watermark (`memory.high` = 179.2 MiB) | `268435456` |
| **`promote_to_unlimited()`** | **`max`** ← promotion works |
| **then `docker update --memory 0`** | **`268435456`** ← reverted |

A 400 MiB allocation afterwards dies with exit `137` (SIGKILL/OOM).

The three command forms, isolated:

| Command | Effect on `memory.max` |
|---|---|
| `--memory 0 --memory-swap -1` (what the judge ran) | **unchanged** — silent no-op |
| `--memory 4g --memory-swap 4g` | **changed to `max`** ✓ |
| `--memory 0` after raising swap | unchanged |

So `docker update` works fine — it is specifically the value `0` that is inert.

### 2.3 Why this went unnoticed

Every previous "promotion works" check — including `deploy.sh`'s smoke test —
used a workload that peaked **below** 256 MiB. Such a workload passes whether
promotion works or not, because it never needs the cap raised. The check was
vacuous: it confirmed that promotion was *reported*, never that it *functioned*.

The kernel's own counters were the tell. Across the broken run:

| Counter | Δ | Meaning |
|---|---|---|
| `oom_kill` | **+5** | five containers killed by the kernel |
| `memory.max` hits | **+189** | allocation attempts blocked at the ceiling |
| `high` | +46 | watermark pressure signals |

Five `oom_kill`s matched the five MLE cases exactly.

### 2.4 The fix

- **Primary path:** the cgroup write *is* the promotion. The reverting
  `docker update` is removed, and the write's result is now checked instead of
  discarded — `tier_promoted` is only set on success.
- **Fallback path** (no host cgroup, Docker-Desktop-style): passes an explicit
  `--memory 8g` rather than `0`, and only records success when Docker exits 0.

Commits: `0dbe4ec` (fix), `c49049a` (results).

---

## 3. Results

### 3.1 Before vs after

| Case | Target | Before | After | prom_ms | Peak after | Promoted |
|---|---|---|---|---|---|---|
| `l005_py` | 5 MiB | — | AC | 0 | 9.4 MiB | no |
| `l020_py` | 20 MiB | — | AC | 0 | 24.5 MiB | no |
| `m040_py` | 40 MiB | — | AC | 0 | 44.6 MiB | no |
| `m060_py` | 60 MiB | — | AC | 0 | 60.7 MiB | no |
| `m080_py` | 80 MiB | — | AC | 0 | 84.1 MiB | no |
| `b170_py` | 170 MiB | AC | AC | 0 | 172.4 MiB | **no** ✓ |
| `b185_py` | 185 MiB | AC | AC | 3183 | 188.7 MiB | **yes** ✓ |
| `b200_py` | 200 MiB | — | AC | 3256 | 205.1 MiB | yes |
| `h250_py` | 250 MiB | AC | AC | 3158 | 252.5 MiB | yes |
| `h400_py` | 400 MiB | **MLE** | **AC** | 3276 | **405.2 MiB** | yes |
| `h900_py` | 900 MiB | **MLE** | **TLE** | 3190 | **629.3 MiB** | yes |
| `h250i_py` | 250 MiB instant | AC | AC | 610 | 254.6 MiB | yes |
| `h400i_py` | 400 MiB instant | **MLE** | **AC** | 661 | **404.7 MiB** | yes |
| `m080_cpp` | 80 MiB | — | AC | 0 | 81.5 MiB | no |
| `b185_cpp` | 185 MiB | — | AC | 3813 | 193.7 MiB | yes |
| `h400_cpp` | 400 MiB | **MLE** | **AC** | 3657 | **402.0 MiB** | yes |
| `h400i_cpp` | 400 MiB instant | **MLE** | **AC** | 913 | **401.7 MiB** | yes |
| `m080_java` | 80 MiB | AC | AC | 0 | 99.5 MiB | no |
| `h400_java` | 400 MiB | RE | RE | 5035 | 187.6 MiB | yes |

**Memory-caused failures: 6 → 1.** The remaining one is Java.

### 3.2 Kernel verification

Over the post-fix 19-case run:

| Counter | Δ before fix | Δ after fix |
|---|---|---|
| `oom_kill` | **+5** | **+0** |
| `memory.max` hits | **+189** | **+0** |
| `high` | +46 | +41 |

**Not one container was OOM-killed after the fix**, and no allocation was ever
blocked at the ceiling. Containers reached 405, 402, 404 and 401 MiB — genuinely
beyond the 256 MiB tier, which is the behaviour the mechanism always claimed but
never delivered.

### 3.3 Watermark precision confirmed

- **170 MiB → no promotion**, peak 172.4 MiB. Correct: below the 179.2 MiB watermark.
- **185 MiB → promoted**, peak 188.7 MiB. Correct: above it.

The trigger boundary is exactly where it should be. This part of the design was
always sound; it was the action that was broken.

### 3.4 Promotion latency

`promotion_time_ms` is measured from submission **acceptance**, so it includes the
time for the workload to reach the watermark — which is why allocation style
dominates it:

| Style | Latency range |
|---|---|
| instant (watermark crossed immediately) | **610–913 ms** |
| gradual (8 MiB/120 ms ramp) | **3158–5035 ms** |

An earlier figure in the paper quoted **248–808 ms**. That measured the promotion
*action* alone, not acceptance-to-promoted. The instant-allocation cases
(610–913 ms) are the closest match to it; the gradual cases are legitimately
slower because the program spends ~2.7 s reaching the watermark before there is
anything to promote.

### 3.5 Language interactions

These are runtime quirks, not tier defects, and they matter for interpreting any
MLE:

- **Java (`h400_java` → RE, peak 187.6 MiB).** The JVM's `-Xmx` is fixed at launch,
  so it throws `OutOfMemoryError` inside its own heap and exits as RE before the
  container limit is relevant. Promotion cannot rescue it. This is a JVM-level
  failure that happens to look like a memory failure.
- **C++.** The compiler peaks at 189–211 MB (per the paper's boundary study), so a
  C++ case can fail at the *compiler* rather than at runtime. All C++ cases here
  returned AC, so no ambiguity arose — but an `SE` on a C++ case should never be
  read as a tier failure.

`h900_py` now returns **TLE rather than MLE, at 629.3 MiB**. That is the correct
outcome: the gradual 900 MiB program needs ~13.5 s to finish, exceeding the 10 s
wall guard, so it is time-bound rather than memory-bound. Memory stopped being the
constraint — which is precisely what the fix was for.

---

## 4. Consequences for the paper

1. **The headline mechanism claim was false in the shipped code and is now true.**
   Any earlier result that depended on promotion *functioning* was not measuring
   what it claimed. This is the most important correction.
2. **The 256 MiB floor claim is unaffected** — it rests on the 128 MiB boundary
   study, which is about resource *limits*, not promotion.
3. **Promotion latency figures need restating** as acceptance-to-promoted
   (610–5035 ms), with the action-only figure noted separately.
4. **The Java finding should be kept prominent.** It is a genuine limitation:
   promotion is memory-only, and the JVM's fixed heap bypasses it entirely.
5. Because the measured corpus contained no heavy submissions, the projection's
   heavy fraction comes from synthetic profiles — the two should not be blended
   without saying so.

## 5. Open

- The heavy fraction of the contest projection still rests on synthetic profiles,
  not cloud-measured ones.
- `--cpus 0` in the same `docker update` was equally inert; CPU promotion is
  documented as a follow-up and remains unimplemented.
- The judge's monitor polls every 2 ms, so very short spikes below one poll
  interval can still be missed. Not exercised here.
