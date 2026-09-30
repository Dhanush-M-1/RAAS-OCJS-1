# Experimental Results & Evaluation

Empirical data collected from the RAAS-OCJS execution engine on the calibration
host: a dedicated bare-metal Linux machine running cgroups v2 (unified hierarchy)
with the Docker `cgroupfs` v2 driver. All figures below were collected over the LAN
by driving the running judge through `POST /submit`, and every one of them is
stored under `benchmarks/results/*_tier256.csv` by
[`benchmarks/raas_benchmark.py`](../benchmarks/raas_benchmark.py). The corpus,
its composition and the harness subcommands are documented in
[`BENCHMARK_SUITE.md`](BENCHMARK_SUITE.md).

> **Testbed.** 15 GiB usable RAM — the burst-stress scenario set is computed
> against this ceiling. The CPU model, kernel build and Docker Engine version
> quoted in earlier drafts of this document are `UNVERIFIED - needs
> measurement`; they are not part of the 2026-09-30 record. The judge runs as
> root (`sudo ./target/debug/server`) so that the reactive paths can write
> `memory.high` into `/sys/fs/cgroup/...`; all measurements were taken with root,
> so live promotion is active.
> **Tier.** The low tier is 256 MiB (`LIGHT_TIER_MB=256`), and it must match the
> server's own low-tier size or every derived slot, density and cloud figure is
> invalid. Every result file is suffixed with the tier that produced it
> (`_tier256`), so a figure can always be traced to the run that produced it.
> The soft watermark is `HIGH_WATERMARK_PCT = 70` of the low-tier hard limit
> (`server/src/docker.rs:21`, computed at :38/:48), i.e. **179.2 MiB** at the
> default 256 MiB tier. Docker sets `memory.max` but not `memory.high`, so the
> monitor drives `memory.high` itself.

---

## 1. Method

**Coverage.** The matrix is 100 real corpus submissions x 4 scheduling
strategies (Baseline, Predictive, Reactive, Hybrid) = **400 live runs** at the
256 MiB tier. Every cell is a real `POST /submit` against the running judge using
the submissions committed under `benchmarks/dataset/sources/`; no figure in this
document is hand-written.

**Verdict completeness.** All **400/400 runs returned `AC`**, with **0 transport
failures**, completing in **394 s** total. No `TLE`, `MLE`, `WA`, or `RE`
occurred, so every cell completed inside both the 10 s per-case deadline and the
256 MiB tier limit. **0 runs were promoted.** 246 runs started high (uncapped)
and 154 started low (256 MiB).

**Peak memory.** Across the 400 live runs: min 6.3 MB, mean 14.4 MB, median
10.6 MB, max 53.0 MB. **383/400 (95.8%) peaked under 25 MB.** Each run was
allocated 256 MB when it started low, or 2048 MB when it started high.

**Problem set.** The corpus is real CodeContests data, not our own fixtures:
100 submissions over 43 unique problems, 100% `CODEFORCES`, spread across cpp 42
/ python 30 / java 28. By program type: Brute Force/Simulation 29, Data
Structures 20, DP/State Table 16, Graph/Trees 12, Number Theory/Math 10,
Greedy/Sorting 9, Bitmask/Bits 4. By difficulty letter: A 20, B 19, E 19, D 18,
C 12, F 9, J 2, G 1. Input sizes run from 26 B to 975 B (mean 267 B).

## 2. Live Reactive Promotion Traces

**There are none in this corpus, and that is the finding.** A pure CodeContests
corpus contains no memory-heavy programs, so across all 400 live runs the
Reactive and Hybrid monitor **never fired: 0 promotions**. Peak use was 53.0 MB
against a 256 MiB tier. This corpus therefore does **not** demonstrate the
adaptive mechanism, and no promotion latency or promotion-time figure should be
quoted from it.

The mechanism is exercised by the **synthetic set** (`--with-synthetic`): 10
submissions, 30 s, **8/40 promotions**. That set is the only path in the suite
that reaches the watermark, because it is the only one that contains a program
sized to do so.

### 2.1 Java heap fix (synthetic heavy knapsack; same judge, same tier)

The heavyweight probe exposed a real defect. The runtime launched **every** Java
submission with a hardcoded `-Xmx512m` inside a 256 MiB container — a heap
allowance twice the container cap, which cannot be honoured. Heap is now derived
from the tier (75% of the Low limit = `192m`; 2x Low for High), with
`-XX:+UseSerialGC`.

| Build | Strategy | Promoted | Outcome | Peak |
|---|---|---|---|---|
| Before | Reactive, Hybrid | Yes | **MLE** | 255.9 MB |
| Before | Predictive | No | **MLE** | 256.0 MB |
| After | Reactive, Hybrid | Yes | **RE** | 202.6 MB |
| After | Predictive | No | **RE** | 202.5 MB |

**Consequence.** Because `-Xmx` is fixed at launch, reactive promotion can no
longer rescue a Java submission that needs more than the tier: the heap ceiling
is already set when the process starts. For JVM languages, *routing*
(predictive, i.e. getting the submission into the right tier up front) is the
mechanism that matters. The same program in C, C++ and Python is promoted and
returns `AC` at 171–203 MB.

**Wide-suite impact.** All 400 runs were re-run against the fixed build: the
result is unchanged (400/400 `AC`), because corpus Java peaks near 25 MB, well
inside a `192m` heap.

## 3. Full Measured Matrix

### 3.1 Macro simulation (N = 10,000, seeded 42, 43 real problems, 256 MiB tier)

| Strategy | slots | allocated GB | used GB | wasted | CPU core-h | avg queue wait ms | P95 turnaround ms |
|---|---|---|---|---|---|---|---|
| Baseline | 7 | 20000.0 | 119.71 | 99.4% | 5.622 | 2.36 | 1448.52 |
| Predictive | 56 | 14860.25 | 116.71 | 99.21% | 4.827 | 0.0 | 1434.47 |
| Reactive | 56 | 2500.0 | 116.79 | 95.33% | 2.876 | 0.0 | 1471.9 |
| Hybrid | 56 | 14860.25 | 118.93 | 99.2% | 4.831 | 0.0 | 1418.68 |

Memory saved vs baseline: Predictive **5139.75 GB (25.7%)**, Reactive
**17500.0 GB (87.5%)**, Hybrid **5139.75 GB (25.7%)**. Live promotions: **0** for
all four strategies — the simulation inherits the corpus, which does not promote.

**Read the Baseline column with care.** Baseline assumes the 2048 MiB worst-case
per pod, whereas the adaptive strategies use the 256 MiB tier. That column is the
whole source of the savings, and the 256 MiB tier is only deployable if the tier
holds at that size (Section 4.2).

### 3.2 Per-language metrics (baseline -> reactive, `_tier256`)

| Language | n | share | CPU ms | wall ms | P95 ms | waste |
|---|---|---|---|---|---|---|
| C++ | 6743 | 67.4% | 115.7 -> 116.4 | 1219.1 -> 1216.8 | 1514.1 -> 1526.8 | 99.47% -> 95.87% |
| Java | 1008 | 10.1% | 290.2 -> 292.1 | 876.8 -> 1127.9 | 1003.8 -> 1332.8 | 98.82% -> 90.99% |
| Python | 2249 | 22.5% | 152.9 -> 153.0 | 451.4 -> 449.3 | 663.2 -> 659.5 | 99.46% -> 95.65% |

All three languages sit at the same **87.5%** memory saving under Reactive.

### 3.3 Burst stress (N = 500, 30 s window, host 15 GiB)

Slots: baseline safe **7**, baseline 2x overcommit **14**, adaptive
`floor(14336/256)` = **56**.

| Scenario | slots | avg queue wait ms | P95 queue wait ms | P95 turnaround ms | drain s | host util |
|---|---|---|---|---|---|---|
| Baseline (Safe) | 7 | 20966.9 | 40683.8 | 41794.6 | 73.8 | 93.3% |
| Baseline (Overcommitted) | 14 | 2996.0 | 6348.2 | 7401.6 | 37.7 | 186.7% |
| Predictive (adaptive) | 56 | 0.0 | 0.0 | 1421.8 | 31.3 | 93.3% |
| Reactive (adaptive) | 56 | 0.0 | 0.0 | 1482.8 | 31.2 | 93.3% |
| Hybrid (adaptive) | 56 | 0.0 | 0.0 | 1394.5 | 31.3 | 93.3% |

The overcommitted baseline is the cautionary row: it drains faster than the safe
baseline only because it swaps 186.7% of host RAM against the disk. The adaptive
rows hold 93.3% utilisation with zero queueing at 8x the slot count.

## 4. Aggregate Findings

### 4.1 Tiering carries no measurable CPU cost

| Language | CPU ms (baseline) | CPU ms (reactive) | delta |
|---|---|---|---|
| C++ | 115.7 | 116.4 | +0.7 |
| Java | 290.2 | 292.1 | +1.9 |
| Python | 152.9 | 153.0 | +0.1 |

Tiering changes which cgroup limits apply, not how much work the submission does,
so CPU time should not move; across all three languages it moves by under 2 ms.
Wall time is likewise flat-to-better for C++ and Python (1219.1 -> 1216.8 ms and
451.4 -> 449.3 ms); Java wall time *rises* (876.8 -> 1127.9 ms), which is the
JVM paying for the tier at start rather than in the compute.

### 4.2 The savings are reserved capacity, not RSS

Peak RSS frequently sits well below the tier, and the live runs peaked at
6.3–53.0 MB (Section 1), so the adaptive strategies are not saving *used* memory
— `used GB` is 116.71–119.71 across all four simulation strategies. What the
scheduler controls is **reserved capacity**: a bounded run cannot consume more
than 256 MiB, so a host admits a predictable number of concurrent submissions,
whereas Baseline reserves 2048 MiB per pod against a workload that mostly needs
tens of MB. Over 10,000 submissions that is reserved RAM **20000.0 -> 2500.0 GB,
an 87.5% reduction**. This entire claim is conditional on the 256 MiB tier
holding.

**Provenance, and an open discrepancy.** The 87.5% figure comes from the harness's
own macro simulation
(`benchmarks/results/real_dataset_strategy_summary_tier256.csv`), in which Reactive
records **zero** live promotions — so it reserves exactly $10{,}000 \times 256$ MiB
$= 2{,}500$ GB. The project paper reports the *same nominal experiment* — 10,000
submissions at the 256 MiB tier — as **10,557.00 GB reserved, a 47.22% reduction**,
at a **20.3%** promotion rate. Both cannot describe the same workload: charging
20.3% of submissions the 2048 MiB ceiling is incompatible with a 2,500 GB total.
The two runs sample different workload mixes and model promotion differently. This
document reports the harness CSV because that is the reproducible in-repo artifact;
`CONFERENCE_EVALUATION_REPORT.md` reports the paper's figures because the paper is
the authoritative written record. **Reconciling the two is an open item — do not
cite both as if they measured the same thing.**

### 4.3 Promotion is memory-pressure-only, and routing is what matters for JVM languages

Promotion lifts memory and CPU limits, but it is triggered by memory pressure
only: a purely CPU-bound submission is never promoted (the synthetic
`S3_floyd_cpu` program exists to cover exactly that case). And with the Java heap
fix in place, `-Xmx` is fixed at launch, so reactive promotion cannot rescue a
JVM submission that needs more than its tier — routing it correctly up front can.
This is the empirical argument for combining prediction with kernel-level
enforcement rather than relying on either alone.

### 4.4 Cloud provisioning projection

| Dimension | Static baseline | RAAS-OCJS adaptive | Gain |
|---|---|---|---|
| Per-pod memory | 2048 MiB | 256 MiB | 8.0x |
| Per-pod CPU | 2.0 vCPU | 1.0 vCPU | 2.0x |
| Density (c6i.4xlarge, 32 GB) | 14 pods | 112 pods | 8.0x |
| 500-sub burst fleet | 36 VMs | 5 VMs | 86.1% |
| Cost @ USD 0.68/hr | 24.48 USD/hr | 3.40 USD/hr | 86.1% (21.08 USD/hr saved) |
| Reserved RAM over 10,000 subs | 20000.0 GB | 2500.0 GB | 87.5% |

### 4.5 Routing quality: the misroute metric

The tiering claims above assume each submission reaches the tier it needs. Those
classifier figures are **not** produced by the live harness; they come from
[`model-training/evaluate_routing.py`](../model-training/evaluate_routing.py),
which replays the trained models through the deployed per-language routing rule
on a held-out, problem-disjoint split.

**Why misroute rather than accuracy.** A **misroute** is a genuinely-Heavy
program sent to the Low tier. It is the metric that matters, because plain
accuracy is dominated by the Light majority: a model that leans Low can post a
high accuracy while failing exactly the submissions the Low/High split exists to
protect. (Note the vocabulary: "Light/Heavy" are the *labels*; the tiers in code
are `Low`/`High`, `server/src/policy.rs`.)

**Data eras.** The original shipped models were trained on CodeContests, whose
Light/Heavy labels are a length/difficulty heuristic (`len(code) < 650` /
`>= 1200`), not a memory measurement; no program in that corpus exceeded 50 MiB
measured. The new models use CodeNet with **measured** memory labels — Light
< 25 MiB, Heavy >= 100 MiB, with the ambiguous 25-100 MiB band dropped rather
than guessed — over 164,686 submissions and 2,520 unique problems. Across
languages, source length explains only 7-18% of the variance of measured memory,
which is why the label change was necessary.

**Headline.** All three rows use the same decision rule — the thresholds
currently shipped in `server/src/predict.rs` (`THRESHOLD_UNIFIED` 0.319,
`THRESHOLD_CPP` 0.346, `THRESHOLD_JAVA` 0.257, `THRESHOLD_PYTHON` 0.200) — and
the same problem-disjoint test split (28,687 submissions over 483 problems, held
out from 125,159 train / 1,930 problems; 80/20 problem-grouped split, seed 42).
Only the model changes.

| model set | features | C | C++ | Java | Python | all | routed High |
|---|---|---|---|---|---|---|---|
| original shipped | 32 | 17.9% | 9.8% | 15.5% | 33.5% | **23.7%** | 52.6% |
| CodeNet labels only | 32 | 46.4% | 4.7% | 8.1% | 6.3% | **6.4%** | 59.4% |
| + allocation features | 36 | 25.0% | 4.8% | 7.5% | 6.3% | **6.3%** | 59.1% |

**Attribution.** The **label fix did almost all of the work**. The four added
allocation features are near-neutral overall (6.4% -> 6.3%), but they cut C's
misroute from 46.4% to 25.0%, because C routes through the unified model, which
received them.

**Accuracy-optimal thresholds are the wrong objective.** The trainer selects
thresholds by maximising accuracy, which the Light majority dominates; applied to
the new models that choice makes routing *less* safe:

| threshold set | all misroute | routed High |
|---|---|---|
| deployed (tuned for the old models) | 6.3% | 59.1% |
| each model's own CV-optimal | 13.4% | 51.0% |
| misroute-minimising | 2.2% | 70.4% |

The misroute-minimising row is **degenerate** — it pins every threshold to the
0.05 grid floor, i.e. "route almost everything High" — which shows misroute alone
is not a usable objective; it has to be balanced against over-provisioning. The
deployed thresholds are the safer operating point, so the new models ship with
the **existing** thresholds.

**How much of a misroute is a real failure?** Heavy means ">= 100 MiB", but the
Low tier's hard limit is 256 MiB, so a program measured between those two numbers
is labelled Heavy yet completes inside Low anyway. Of the **945** misrouted
programs in the test split (memory 100.0-955.9 MiB, median 142.4):

- **834 (88.3%) would have fit inside the 256 MiB Low tier**
- **111 (11.7%) would have exceeded it**

By band: 100-150 MiB 621; 150-200 MiB 127; 200-256 MiB 86; 256-512 MiB 71;
512-1024 MiB 40. So the raw **6.3% decomposes into 0.7% genuine over-limit
failures and 5.5% boundary artefacts**. **Caveat:** CodeNet's `memory` was
measured on IBM/Aizu hardware, not on this project's cgroup judge, so absolute
MiB does not map one-to-one; the honest range is **0.7%-6.3% real failures**,
with 0.7% the optimistic end.

Two verified mechanisms reduce the real-world impact further. The promotion
watermark is `HIGH_WATERMARK_PCT = 70` of the Low limit = **179.2 MiB**
(`server/src/docker.rs`), so most non-Java misroutes between 150 and 256 MiB
cross the watermark and are promoted before they OOM. **Java cannot be rescued
this way**: `-Xmx` is fixed at JVM launch, so for Java a misroute is fatal
regardless of promotion, and Java's figure (7.5%) should **not** be discounted
the way the aggregate can be.

## 5. Threats to Validity

**Single run per cell.** Each of the 400 cells is one execution. We report no
confidence intervals, and small differences between neighbouring cells should
not be read as real. The one deliberate repeat is the wide-suite re-run after the
Java heap fix, which reproduced 400/400 `AC`.

**The workload is real but narrow, and it contains no heavy programs.** These are
genuine CodeContests submissions rather than our own fixtures, which is an
improvement on earlier drafts of this document — but they are 100% `CODEFORCES`,
100% `AC`, and none of them is memory-heavy. That last property is why **0/400
live runs promoted**: the corpus cannot exercise the adaptive mechanism at all.
Any promotion claim must come from the synthetic set, not from this matrix.

**Model accuracy figures are no longer provisional.** Earlier drafts marked them
`UNVERIFIED - superseded by retrain in progress`. That retrain is complete: the
new models use CodeNet measured-memory labels with the 36 per-language / 40
unified feature vectors, and their test metrics and routing behaviour are
reported in Section 4.5. The v2 per-model metrics are: Unified CV 89.56% / test
87.38% / F1 88.48% / AUC 0.9503; C++ 96.07% / 94.39% / 96.26% / 0.9904; Java
86.73% / 84.19% / 83.30% / 0.9172; Python 85.02% / 83.71% / 86.51% / 0.9221; C
97.76% / 96.64% / 41.12% / 0.9387. C is **data-limited** — only 428 Heavy
examples in the 12.7M-row CodeNet scan, median C memory 0.6 MiB — and its low F1
follows from that. A specialised C model is trained and exported, but
`predict.rs` routes C through the **unified** model, so no C-specific classifier
is in service. This matrix measures what the deployed thresholds do on unseen
real submissions; it does not re-validate the classifier.

**The frontend does not consume these figures.** `frontend/src/App.tsx` calls
only `/health` and `/submit`; it does not read the benchmark CSVs, so the UI
cannot be used to corroborate any number in this document. The traceable source
is the `benchmarks/results/*_tier256.csv` files.

**No concurrency testing.** All 400 live runs were submitted serially. This
matrix says nothing about behaviour under simultaneous load; the queue-latency,
drain-time and host-utilisation claims come from the discrete-event simulation,
which assumes the 256 MiB tier holds.

**The judge is unsafe to expose.** It binds `0.0.0.0:3000` with no
authentication and executes untrusted code. It must never be exposed publicly.

**Closed item: the TLE and MLE probes now pass.** Earlier drafts of this document
recorded that the judge on the calibration host was running a build predating the
TLE/MLE commit, so the `TLE` and `MLE` paths had only ever been exercised against
a build that did not yet contain them. That item is now **closed**: `probe`
re-runs the five verdicts from ground truth and all five pass **5/5** — `AC`,
`WA`, `RE`, `TLE`, `MLE`. `TLE` is killed by the 10 s per-case guard (observed
~10.03 s CPU / ~10.26 s wall) and `MLE` starts in the low tier, peaks
~255.5–256.0 MB and is OOM-killed. None of these paths is exercised by the 400
live cells, so the matrix is unaffected either way.
