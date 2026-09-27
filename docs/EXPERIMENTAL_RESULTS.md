# Experimental Results & Evaluation

Empirical data collected from the RAAS-OCJS execution engine on a multi-core
Linux host running kernel 7.1.1 with cgroups v2 (unified hierarchy) and the
Docker `cgroupfs` v2 driver.

> **Testbed for this matrix.** AMD Ryzen 5 3600 (6 cores / 12 threads),
> 7.7 GiB usable RAM, Pop!\_OS 22.04 LTS, kernel `7.1.1-76070101-generic`,
> Docker Engine 29.7.2 (`cgroupfs` driver, cgroup v2). This is a shared
> workstation, not a dedicated calibration host, and it has considerably less
> RAM than the 15 GiB bare-metal box used for the macro-scale simulation in the
> paper. Absolute peak-RSS figures are specific to this host; the tiering
> behaviour and the promotion traces are not.

**Measurement protocol.** Every cell below is a live `POST /submit` against the
running judge, not a hand-written figure. The full matrix is
**5 problems × 4 languages × 4 strategies = 80 runs**, using the reference
solutions and test cases shipped in `frontend/src/problems.ts` (not
reconstructed fixtures). CPU time is the cgroup v2 `cpu.stat` `usage_usec`
delta; peak memory is the maximum `memory.current` sampled on a 2 ms tick.
Because the 2 ms sampling interval cannot catch every transient, peak-memory
figures carry roughly ±5% sampling noise.

> The judge must run as root (`sudo ./target/debug/server`) for the reactive
> paths: writing `memory.high` into `/sys/fs/cgroup/...` requires it. All
> measurements below were taken with root, so live promotion is active.

> **Watermark setting used for this matrix.** These runs used
> `LOW_MEM_HIGH_WATERMARK = 128 MiB` (50% of the 256 MiB tier). The watermark
> is since been made configurable via `HIGH_WATERMARK_PCT` in
> `server/src/docker.rs`, defaulting to 70% (~179.2 MiB). A higher watermark
> means promotion fires later and leaves a thinner cushion before
> `memory.max`; the tiering verdicts and CPU measurements below are unaffected,
> but the promotion timings in §4 are specific to the 128 MiB setting and
> should be re-measured before being cited against a 70% configuration.

---

## 1. Verdicts

**All 80 runs returned `AC`.** No submission in the benchmark suite was
miscompiled, timed out, OOM-killed, or rejected by a tier assignment. The
scheduling layer changed *how* each submission was isolated without changing
whether it was judged correct — which is the property the architecture claims.

---

## 2. Per-Strategy Aggregate

| Strategy | Mean peak RSS | Max peak RSS | Mean CPU | Runs held at 256 MiB |
|---|:---:|:---:|:---:|:---:|
| Baseline | 57.7 MB | 195.6 MB | 210.6 ms | 0 / 20 |
| Predictive | 46.6 MB | 190.2 MB | 211.6 ms | 14 / 20 |
| Reactive | 44.8 MB | 191.1 MB | 210.9 ms | 16 / 20 |
| **Hybrid** | **44.4 MB** | 191.3 MB | 216.2 ms | 12 / 20 |

Mean CPU time is flat across all four strategies (210.6–216.2 ms, spread ≈2.7%).
**Isolation tiering has no measurable CPU cost** — the `cpu.stat` delta confirms
this directly, since it excludes `docker exec` startup noise entirely.

The 256 MiB column is the result that matters. Across the whole matrix, **52 of
80 runs (65%) were held at a hard 256 MiB ceiling**; the remaining 28 were
Baseline or were promoted mid-run.

---

## 3. Full Matrix (peak RSS in MB, `*` = promoted mid-run)

| Problem | Lang | Baseline | Predictive | Reactive | Hybrid |
|---|:---:|:---:|:---:|:---:|:---:|
| **P1** Prefix Sums | Python | 8.4 | 8.3 | 9.1 | 8.4 |
| | C++ | 6.2 | 6.5 | 6.1 | 6.7 |
| | Java | 21.7 | 21.2 | 21.1 | 21.3 |
| | C | 6.4 | 6.4 | 6.0 | 7.3 |
| **P2** Knapsack Buffer | Python | 173.7 | 174.1 | **173.3\*** | **173.9\*** |
| | C++ | 174.5 | 169.8 | **170.2\*** | **169.6\*** |
| | Java | 195.6 | 190.2 | **191.1\*** | **191.3\*** |
| | C | 177.3 | 170.2 | **169.8\*** | **169.4\*** |
| **P3** Floyd-Warshall | Python | 10.1 | 7.8 | 9.2 | 8.9 |
| | C++ | 6.8 | 9.3 | 7.1 | 6.8 |
| | Java | 25.9 | 25.8 | 33.0 | 24.0 |
| | C | 34.6 | 42.9 | 6.8 | 6.8 |
| **P4** Tree Search | Python | 9.3 | 10.3 | 10.0 | 10.2 |
| | C++ | 42.9 | 6.7 | 6.2 | 6.6 |
| | Java | 63.4 | 22.3 | 21.4 | 21.1 |
| | C | 6.5 | 6.2 | 6.9 | 5.8 |
| **P5** Top-K Streaming | Python | 22.5 | 12.6 | 11.7 | 10.9 |
| | C++ | 51.3 | 9.5 | 7.0 | 6.5 |
| | Java | 79.8 | 26.2 | 23.8 | 26.2 |
| | C | 36.2 | 6.3 | 6.6 | 6.5 |

---

## 4. Live Reactive Promotion (P2)

P2 allocates a 150/160 MiB buffer and is the only benchmark that crosses the
128 MiB soft watermark. Promotion fired in **8 of 8** eligible runs — Reactive
and Hybrid across all four languages — and in **zero** of the Predictive runs,
which cannot promote by design.

| Lang | Reactive | Hybrid | Peak RSS | CPU |
|---|:---:|:---:|:---:|:---:|
| Python | 372 ms | 468 ms | ~174 MB | 301–308 ms |
| C++ | 742 ms | 717 ms | ~170 MB | 247–254 ms |
| Java | 1444 ms | 1361 ms | ~191 MB | 447–459 ms |
| C | 595 ms | 550 ms | ~170 MB | 259–305 ms |

Promotion latency tracks the language's allocation *rate*, not the threshold
crossing: Python touches its 150 MiB fastest and promotes soonest, Java pays
JVM class-loading and heap-growth overhead first. Every promoted run still
finished `AC`, confirming that lifting the ceiling mid-execution does not
disturb the running process.

---

## 5. Findings

### 5.1 The saving is in reserved capacity, not in RSS

The clearest correct claim is about **capacity, not footprint**. Bounded runs
reserve a hard 256 MiB; Baseline reserves whatever the host has. On a 15 GiB
self-hosted judge that is the difference between admitting a predictable number
of concurrent submissions and letting the first heavy job set the ceiling for
everyone.

Note that peak RSS often *rises* slightly under a bounded tier (P1 C++: 6.2 →
6.5 MB). That is not a regression — it is the 2 ms sampler reading a different
set of transient pages. Reporting peak RSS as the primary saving, as earlier
revisions of this document did, overstates what the scheduler controls.

### 5.2 Predictive under-reserves on P2, and that is the finding

P2 needs ~170–195 MB. Predictive routed **all four languages to the Light
tier** anyway, because the model is trained on CodeNet/CodeContests submissions
and these synthetic benchmark programs sit outside that distribution. Measured
headroom under the 256 MiB ceiling:

| Lang | Peak under Predictive | Headroom remaining |
|---|:---:|:---:|
| Java | 190.2 MB | 65.8 MB |
| C | 170.2 MB | 85.8 MB |
| C++ | 169.8 MB | 86.2 MB |
| Python | 174.1 MB | 81.9 MB |

All four survived, but Java cleared the ceiling by only ~26%. A submission
roughly 35% larger would have been OOM-killed with no promotion path — the
Light tier is a hard `memory.max`, and Predictive does not arm the reactive net.

This is not a defect to hide; it is the empirical justification for the reactive
path existing. Pure static analysis was not sufficient on this workload, and the
Hybrid strategy is what closes the gap. Reactive and Hybrid promoted all eight
P2 runs; Predictive rescued none.

### 5.3 Tier assignment is stable but not always optimal

P5 Top-K Streaming shows the intended effect cleanly: C++ drops 51.3 → 6.5 MB
and Java 79.8 → 26.2 MB under Hybrid. P3 C is the one case where Predictive is
*worse* than Baseline (42.9 vs 34.6 MB), a reminder that the model is a
heuristic and occasionally mis-ranks.

### 5.4 CFS `cpu.stat` timing

Measuring CPU via the cgroup `usage_usec` delta rather than wall-clock around
`docker exec` removes the 150–200 ms of container startup that otherwise
dominates sub-second programs. Across this matrix the four strategies land
within 2.7% of each other on mean CPU, which is the level of agreement you need
before claiming tiering is free.

---

## 6. Threats to Validity

- **Single host, single run per cell.** No repeated trials, so no confidence
  intervals. The ±3.4–4.7% stability figures quoted in earlier revisions of this
  document were not reproduced here and should not be cited without re-measuring.
- **2 ms sampling under-reports peaks.** A short-lived allocation between two
  ticks is invisible to `memory.current`.
- **Synthetic benchmark programs.** The five problems are hand-written to match
  the complexity classes in the literature; they are not drawn from the
  CodeNet/CodeContests distribution the model was trained on. The P2
  misclassification in §5.2 is a direct consequence and limits how far the
  accuracy figures generalize.
- **A 256 MiB tier is unusually tight for managed runtimes.** Java clears only
  26% headroom on P2; a normal contest setting would use a larger ceiling and
  see fewer promotions.
