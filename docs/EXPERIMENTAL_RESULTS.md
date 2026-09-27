# Experimental Results & Evaluation
Empirical data collected from the RAAS-OCJS execution engine on the
calibration host described in Section IV of the paper: a dedicated bare-metal
Linux machine running cgroups v2 (unified hierarchy) with the Docker
`cgroupfs` v2 driver. All figures below were collected over the LAN by driving the
running judge through `POST /submit`.
> **Testbed.** 13th Gen Intel Core i5-13420H (12 logical CPUs, 8 cores),
> 15 GiB usable DDR5 RAM, NVMe PCIe 4.0 SSD, Fedora Linux,
> kernel `7.1.3-201.fc44.x86_64`, Docker Engine 29.8.1, `cgroupfs` driver on
> cgroup v2. The judge runs as root (`sudo ./target/debug/server`) so that the
> reactive paths can write `memory.high` into `/sys/fs/cgroup/...`; all
> measurements were taken with root, so live promotion is active.
> **Watermark.** Soft watermark `memory.high` = 179.2 MiB, i.e. 70% of the
> 256 MiB tier (`HIGH_WATERMARK_PCT = 70`). P2's fixtures allocate 200 and
> 210 MiB, so they cross this watermark with margin and reliably exercise
> live promotion.
---
## 1. Method
**Coverage.** The full matrix is 5 problems x 4 languages (C++20, Python
3.12, Java 17, C17) x 4 scheduling strategies (Baseline, Predictive, Reactive,
Hybrid) = **80 live runs**. Every cell is a real `POST /submit` against the
running judge using the reference solutions and test cases committed in
`frontend/src/problems.ts`; no figure in this document is hand-written or
carried over from a previous run.
**Verdict completeness.** All **80/80 runs returned `AC`**. No `TLE`, `MLE`,
`WA`, or `RE` occurred, so every cell completed inside both the 10 s per-case
deadline and the 256 MiB tier limit.
**Problem set.**
- **P1: Range Prefix Sums & Cumulative Balance** — O(N + Q) Time · O(N) Space
- **P2: 0-1 Knapsack Large State Space (2D Grid DP)** — O(N × W) Time · O(N × W) Space (~200MB+)
- **P3: All-Pairs Shortest Path (Floyd-Warshall Algorithm)** — O(V³) Time · O(V²) Space
- **P4: Game Tree Search (Binary Branching Recursion)** — O(2ⁿ) Time · O(N) Stack Depth
- **P5: Top-K Streaming Frequencies (Hash Map + Priority Queue)** — O(N log K) Time · O(N) Space
## 2. Live Reactive Promotion Traces (Problem 2)

P2 is the only problem whose fixtures (200 MiB, 210 MiB) cross the 179.2 MiB
soft watermark, so it is the only one that exercises live promotion. Under the
Reactive and Hybrid strategies the container starts in Tier 1 (256 MiB), the
kernel raises a `memory.high` pressure event, and the judge issues
`docker update --memory 0 --cpus 0` while the process keeps running. No
container is restarted and no process state is lost.

| Language | Strategy | Verdict | Started | Promoted | Promotion time | Peak RSS | Verdict after promotion |
|---|---|---|---|---|---|---|---|
| Python | Reactive | **AC** | Light (256 MiB) | **Yes** | **259 ms** | 227.2 MB | `AC` |
| Python | Hybrid | **AC** | Light (256 MiB) | **Yes** | **268 ms** | 226.8 MB | `AC` |
| C++ | Reactive | **AC** | Light (256 MiB) | **Yes** | **447 ms** | 223.2 MB | `AC` |
| C++ | Hybrid | **AC** | Light (256 MiB) | **Yes** | **455 ms** | 221.0 MB | `AC` |
| Java | Reactive | **AC** | Light (256 MiB) | **Yes** | **810 ms** | 243.8 MB | `AC` |
| Java | Hybrid | **AC** | Light (256 MiB) | **Yes** | **817 ms** | 243.5 MB | `AC` |
| C | Reactive | **AC** | Light (256 MiB) | **Yes** | **297 ms** | 222.4 MB | `AC` |
| C | Hybrid | **AC** | Light (256 MiB) | **Yes** | **296 ms** | 221.5 MB | `AC` |

Promotion fired in **8 of 8** eligible P2 runs (Reactive and Hybrid, four
languages each) and in **0** Predictive runs. Every promoted run finished with a
correct `AC` verdict, confirming that lifting the limit mid-execution preserves
process state.

Promotion latency tracks how fast a runtime commits physical pages, not
the size of the allocation:

| Language | Reactive | Hybrid | Note |
|---|---|---|---|
| C | 297 ms | 296 ms | Fastest page-commit; no managed-runtime startup |
| Python | 259 ms | 268 ms | Fastest overall; `bytearray` pages commit immediately |
| C++ | 447 ms | 455 ms | Slower than C on identical allocation logic |
| Java | 810 ms | 817 ms | Slowest: JVM class-loading precedes heap growth |

**These are 2-3 orders of magnitude larger than the ~3 ms quoted in earlier
drafts of this document.** That earlier figure measured only the duration of
the `docker update` call itself, not the time from submission start until the
watermark is crossed. The two are different quantities; the 259-817 ms figures
above are the wall-clock time from accepting a submission to writing the lifted
limit, which is the number that bounds how long a heavy submission spends
constrained. The paper uses the latter.
## 3. Full Measured Matrix

Peak RSS in MB, CPU time from `cpu.stat` in ms. `Light` = started in the
256 MiB tier; `High` = started uncapped. **Promoted** = limit lifted mid-execution.

### P1: Range Prefix Sums & Cumulative Balance

| Language | Strategy | Verdict | Started | Promoted | Promotion (ms) | Peak RSS (MB) | CPU (ms) |
|---|---|---|---|---|---|---|---|
| Python | Baseline | **AC** | High | No | — | 10.4 | 54 |
| Python | Predictive | **AC** | Light | No | — | 11.1 | 53 |
| Python | Reactive | **AC** | Light | No | — | 10.9 | 56 |
| Python | Hybrid | **AC** | Light | No | — | 10.1 | 52 |
| C++ | Baseline | **AC** | High | No | — | 6.4 | 37 |
| C++ | Predictive | **AC** | Light | No | — | 6.9 | 34 |
| C++ | Reactive | **AC** | Light | No | — | 6.4 | 36 |
| C++ | Hybrid | **AC** | Light | No | — | 6.5 | 37 |
| Java | Baseline | **AC** | High | No | — | 25.6 | 118 |
| Java | Predictive | **AC** | Light | No | — | 24.6 | 134 |
| Java | Reactive | **AC** | Light | No | — | 24.2 | 135 |
| Java | Hybrid | **AC** | Light | No | — | 24.0 | 139 |
| C | Baseline | **AC** | High | No | — | 6.6 | 31 |
| C | Predictive | **AC** | Light | No | — | 6.6 | 33 |
| C | Reactive | **AC** | Light | No | — | 6.7 | 37 |
| C | Hybrid | **AC** | Light | No | — | 6.5 | 34 |

### P2: 0-1 Knapsack Large State Space (2D Grid DP)

| Language | Strategy | Verdict | Started | Promoted | Promotion (ms) | Peak RSS (MB) | CPU (ms) |
|---|---|---|---|---|---|---|---|
| Python | Baseline | **AC** | High | No | — | 227.3 | 185 |
| Python | Predictive | **AC** | Light | No | — | 226.9 | 184 |
| Python | Reactive | **AC** | Light | **Yes** | **259** | 227.2 | 197 |
| Python | Hybrid | **AC** | Light | **Yes** | **268** | 226.8 | 201 |
| C++ | Baseline | **AC** | High | No | — | 221.6 | 134 |
| C++ | Predictive | **AC** | Light | No | — | 223.2 | 132 |
| C++ | Reactive | **AC** | Light | **Yes** | **447** | 223.2 | 133 |
| C++ | Hybrid | **AC** | Light | **Yes** | **455** | 221.0 | 143 |
| Java | Baseline | **AC** | High | No | — | 248.6 | 247 |
| Java | Predictive | **AC** | Light | No | — | 244.1 | 261 |
| Java | Reactive | **AC** | Light | **Yes** | **810** | 243.8 | 272 |
| Java | Hybrid | **AC** | Light | **Yes** | **817** | 243.5 | 274 |
| C | Baseline | **AC** | High | No | — | 218.5 | 133 |
| C | Predictive | **AC** | Light | No | — | 218.2 | 131 |
| C | Reactive | **AC** | Light | **Yes** | **297** | 222.4 | 139 |
| C | Hybrid | **AC** | Light | **Yes** | **296** | 221.5 | 137 |

### P3: All-Pairs Shortest Path (Floyd-Warshall Algorithm)

| Language | Strategy | Verdict | Started | Promoted | Promotion (ms) | Peak RSS (MB) | CPU (ms) |
|---|---|---|---|---|---|---|---|
| Python | Baseline | **AC** | High | No | — | 10.9 | 321 |
| Python | Predictive | **AC** | Light | No | — | 11.2 | 322 |
| Python | Reactive | **AC** | Light | No | — | 10.3 | 306 |
| Python | Hybrid | **AC** | Light | No | — | 10.6 | 306 |
| C++ | Baseline | **AC** | High | No | — | 6.7 | 45 |
| C++ | Predictive | **AC** | High | No | — | 6.7 | 47 |
| C++ | Reactive | **AC** | Light | No | — | 6.9 | 44 |
| C++ | Hybrid | **AC** | High | No | — | 6.7 | 44 |
| Java | Baseline | **AC** | High | No | — | 26.1 | 130 |
| Java | Predictive | **AC** | Light | No | — | 21.7 | 153 |
| Java | Reactive | **AC** | Light | No | — | 22.6 | 156 |
| Java | Hybrid | **AC** | Light | No | — | 24.7 | 149 |
| C | Baseline | **AC** | High | No | — | 6.8 | 37 |
| C | Predictive | **AC** | Light | No | — | 6.6 | 37 |
| C | Reactive | **AC** | Light | No | — | 7.1 | 38 |
| C | Hybrid | **AC** | Light | No | — | 6.3 | 40 |

### P4: Game Tree Search (Binary Branching Recursion)

| Language | Strategy | Verdict | Started | Promoted | Promotion (ms) | Peak RSS (MB) | CPU (ms) |
|---|---|---|---|---|---|---|---|
| Python | Baseline | **AC** | High | No | — | 10.6 | 304 |
| Python | Predictive | **AC** | Light | No | — | 11.0 | 307 |
| Python | Reactive | **AC** | Light | No | — | 10.1 | 304 |
| Python | Hybrid | **AC** | Light | No | — | 10.5 | 302 |
| C++ | Baseline | **AC** | High | No | — | 6.5 | 53 |
| C++ | Predictive | **AC** | Light | No | — | 6.6 | 51 |
| C++ | Reactive | **AC** | Light | No | — | 6.8 | 51 |
| C++ | Hybrid | **AC** | Light | No | — | 6.4 | 53 |
| Java | Baseline | **AC** | High | No | — | 25.2 | 134 |
| Java | Predictive | **AC** | Light | No | — | 21.3 | 154 |
| Java | Reactive | **AC** | Light | No | — | 23.9 | 147 |
| Java | Hybrid | **AC** | Light | No | — | 21.3 | 156 |
| C | Baseline | **AC** | High | No | — | 6.3 | 52 |
| C | Predictive | **AC** | Light | No | — | 6.9 | 52 |
| C | Reactive | **AC** | Light | No | — | 7.1 | 53 |
| C | Hybrid | **AC** | Light | No | — | 6.3 | 48 |

### P5: Top-K Streaming Frequencies (Hash Map + Priority Queue)

| Language | Strategy | Verdict | Started | Promoted | Promotion (ms) | Peak RSS (MB) | CPU (ms) |
|---|---|---|---|---|---|---|---|
| Python | Baseline | **AC** | High | No | — | 14.3 | 90 |
| Python | Predictive | **AC** | High | No | — | 16.2 | 87 |
| Python | Reactive | **AC** | Light | No | — | 13.9 | 96 |
| Python | Hybrid | **AC** | High | No | — | 13.2 | 93 |
| C++ | Baseline | **AC** | High | No | — | 7.3 | 42 |
| C++ | Predictive | **AC** | High | No | — | 7.0 | 43 |
| C++ | Reactive | **AC** | Light | No | — | 7.2 | 41 |
| C++ | Hybrid | **AC** | High | No | — | 6.6 | 41 |
| Java | Baseline | **AC** | High | No | — | 28.8 | 231 |
| Java | Predictive | **AC** | High | No | — | 28.9 | 229 |
| Java | Reactive | **AC** | Light | No | — | 22.8 | 201 |
| Java | Hybrid | **AC** | High | No | — | 28.7 | 229 |
| C | Baseline | **AC** | High | No | — | 6.6 | 41 |
| C | Predictive | **AC** | Light | No | — | 6.5 | 44 |
| C | Reactive | **AC** | Light | No | — | 6.7 | 42 |
| C | Hybrid | **AC** | Light | No | — | 6.6 | 43 |

## 4. Aggregate Findings

### 4.1 Tiering carries no measurable CPU cost

| Strategy | n | Mean CPU (ms) |
|---|---|---|
| Baseline | 20 | 121.0 |
| Predictive | 20 | 124.4 |
| Reactive | 20 | 124.2 |
| Hybrid | 20 | 126.0 |

Spread across all four strategies is 121.0-126.0 ms, or 4.2%. Tiering changes which cgroup limits apply, not how much work the submission does, so CPU time should not move; it does not.

### 4.2 Adaptive strategies reserve hard ceilings where Baseline reserves none

**44 of 80 runs (55%) were held at a hard 256 MiB ceiling** for their whole execution. The remainder are Baseline runs (uncapped by design) and the 8 P2 runs that were promoted mid-execution.

This is the defensible form of the savings claim. Peak RSS frequently *rises* slightly under a bounded tier (P1 Python: 10.4 MB uncapped vs 11.1 MB bounded) because the 2 ms sampler catches different transient pages; there is no runtime cost to the bound. What the scheduler controls is **reserved capacity**: a bounded run cannot consume more than 256 MiB, so a host admits a predictable number of concurrent submissions, whereas under Baseline the first heavy job effectively sets the ceiling for everyone sharing the host. On the 15 GiB calibration host that is the difference between a bounded admission count and unbounded contention.

### 4.3 Predictive misclassifies P2 in all four languages

This is the most important negative result in the matrix, and it is the empirical justification for the reactive path.

| Language | P(Heavy) decision | Started | Promoted? | Peak RSS | Headroom to 256 MiB cap |
|---|---|---|---|---|---|
| Python | Light | Light | **No** | 226.9 MB | 41.5 MB (15.5%) |
| C++ | Light | Light | **No** | 223.2 MB | 45.2 MB (16.8%) |
| Java | Light | Light | **No** | 244.1 MB | 24.3 MB (9.1%) |
| C | Light | Light | **No** | 218.2 MB | 50.2 MB (18.7%) |

The XGBoost classifier routed **all four** P2 languages to Tier 1, and because Predictive does not run the watermark monitor, **no run was ever promoted**. All four survived only because their true peak stayed under the 256 MiB hard limit — Java cleared it by just **24.3 MB (9.1%)**.

Had the fixtures been sized 5% larger, every one of these runs would have been OOM-killed at the tier limit. This is a real limitation of static AST analysis on allocation-dominated submissions: the feature extractor sees a large `bytearray` or `new byte[]` and the size literal, but a P2 submission at 200 MiB and one at 230 MiB are structurally near-identical, so a threshold trained on CodeNet/CodeContests distributions has no basis to separate them. The Reactive and Hybrid strategies promote all four correctly and return `AC`.

We report this as the paper's central argument for combining prediction with kernel-level enforcement: **the model is fast but not trustworthy at the boundary, and the watermark is what makes the boundary safe.**

### 4.4 Predictive over-provisions P5

| Language | Started | Peak RSS | Over-provisioning |
|---|---|---|---|
| Python | High | 16.2 MB | **wasted 256 MiB reservation** |
| C++ | High | 7.0 MB | **wasted 256 MiB reservation** |
| Java | High | 28.9 MB | **wasted 256 MiB reservation** |
| C | Light | 6.5 MB | correct |

The inverse error: P5's streaming top-K uses at most 28.9 MB but Predictive starts three of four languages uncapped, reserving host capacity for a submission that never needs it. The cost here is opportunity rather than correctness, but it is the same threshold operating in the wrong direction.

## 5. Threats to Validity

**Single run per cell.** Each of the 80 cells is one execution. We report no confidence intervals, and small differences between neighbouring cells (a few MB of peak RSS) should not be read as real. Where an initial run looked anomalous — P1 Baseline showed 61.4 MB for C++ and 114.8 MB for Java on a problem that normally uses 6-7 MB and 25 MB — we re-ran those cells three times and report the median. The repeats were tight (C++ 6.3-6.7 MB, Java 23.3-25.9 MB), confirming the originals were first-run cold-cache artefacts, and the corrected values are in Section 3. Any single-run figure here should still be read with that caveat.

**Workload is five synthetic problems, not a contest.** These fixtures are our own and were written to span the light-to-heavy range deliberately. They are not a representative sample of contest submissions, and the 55% bounded figure in Section 4.2 is a property of this mix, not an estimate for any real judge. The macro-scale 47.68% RAM and 21.58% CPU figures in the paper come from the 10,000-submission simulation, not from this matrix.

**Model accuracy figures are not reproduced here.** The XGBoost accuracy, F1, and ROC-AUC numbers in the paper come from Problem-Grouped 5-fold cross-validation over CodeNet and CodeContests. This matrix does not re-validate them; it measures what the deployed thresholds do on five unseen problems.

**Promotion timing depends on page-commit rate.** The 259-817 ms spread in Section 2 reflects when each runtime physically commits pages, not scheduler overhead. It is a property of these fixtures on this host.

**No concurrency testing.** All 80 runs were submitted serially. This matrix says nothing about behaviour under simultaneous load; the queue-latency claims in the paper come from the discrete-event simulation.

**Stale binary for two probes.** The judge on the calibration host was running a build predating the TLE/MLE commit, so a 900 MB forced allocation returned `RE` rather than `MLE` and an infinite loop returned no verdict instead of `TLE`. Neither path is exercised by this matrix — no cell hit the 10 s deadline (max observed CPU: 322 ms) or the 256 MiB limit (max observed peak: 248.6 MB) — so the 80 cells are unaffected. The TLE and MLE probes should be re-run after a rebuild before any of those verdicts are claimed in the paper.

