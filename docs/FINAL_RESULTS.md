# RAAS-OCJS — Final Verified Results

Two independent evidence bases are reported here, and each figure names which one
it belongs to.

- **Sections 1–6** are **live benchmark runs**. Every number traces to a CSV in
  `benchmarks/results/`, produced by driving the running judge through
  `POST /submit` with the corpus committed under `benchmarks/dataset/`.
- **Sections 7–11** are **model-training results**. Every number traces to an
  artifact under `model-training/` (the v2 models in `artifacts_codenet_v2/`),
  trained on the CodeNet measured-memory labels.
- **Section 12** lists the corrected values that replaced figures from earlier
  drafts.

Two configurations were measured: the shipped 256 MiB light tier, and a 128 MiB
tier used only to locate the floor.

- Judge: `http://192.168.0.111:3000` (calibration host)
- Testbed: 13th Gen Intel Core i5-13420H, 12 threads, 15 GiB usable RAM, Fedora
- Simulation seed: `BENCH_SEED=42`, so every figure is reproducible
- Tier override: `LOW_TIER_MB` (default 256). Watermark is 70% of the tier:
  179.2 MiB at 256, 89.6 MiB at 128.

Before each run the tier was verified two ways: the judge's reported low-tier
allocation had to equal the intended tier, and a probe allocating and touching
a chosen footprint had to survive just under it and be killed just over it.

## Metrology notes

**Memory is charged as reserved limit, not peak RSS.** A submission started in
the low tier is charged the tier size; one started in the uncapped tier, or
promoted during execution, is charged the baseline 2048 MiB ceiling as a
worst case. `Tier::High` is uncapped (`--memory 0`), so it carries no finite
reservation and cannot be counted as dense.

**Allocation comes from the judge's own `tier_started` / `tier_promoted`** for
each submission, never from the corpus's `is_heavy` label. Those disagree in 20
of 72 cells; see section 3.

**The tier is enforced with `--memory-swap` set equal to `--memory`.** Without
it Docker defaults the swap limit to the same value as `--memory`, roughly
doubling the effective ceiling: under a nominal 128 MiB tier a probe survived
235 MB and died at 245 MB. With it, the same probe survives 120 MB and dies at
130 MB. The figures below are from runs with the tier genuinely enforced.

**A capped run reports its own limit as its peak.** `peak_memory_bytes` samples
`memory.current`, which cannot exceed `memory.max`, so true demand is only
observable in uncapped runs.

## 1. Empirical runs, shipped 256 MiB tier

72 live submissions: 18 programs (6 problems x 3-4 languages) x 4 strategies.

| Verdict | Count |
|---|---|
| AC | 69 |
| MLE | 3 |

The three failures are Java on the 2D knapsack, in Predictive, Reactive and
Hybrid alike, peaking at 255-256 MB. The runtime is started with `-Xmx512m`
inside a 256 MiB container, so the heap allowance is twice the container cap
and cannot be honoured.

- **76.8%** of AC runs (53/69) used under 25 MiB; mean 49.2 MB.
- Of the 38 runs that started in the low tier, **30 (78.9%)** never needed
  promotion.

## 2. Empirical runs, 128 MiB tier

| Verdict | Count |
|---|---|
| AC | 52 |
| SE | 8 |
| MLE | 9 |
| RE | 3 |

The tier fails for three independent reasons.

| Language | Runs | AC | SE | MLE | RE | Dominant failure |
|---|---|---|---|---|---|---|
| C | 12 | 9 | 0 | 2 | 1 | knapsack outgrows the tier |
| C++ | 20 | 9 | **8** | 3 | 0 | **will not compile** |
| Python | 20 | 17 | 0 | 2 | 1 | knapsack outgrows the tier |
| Java | 20 | 17 | 0 | 2 | 1 | knapsack outgrows the tier |

1. **The C++ toolchain does not fit.** Eight bare `SE` verdicts, all C++, with
   no case results and zero CPU time. `g++ -O2` peaks at 189-211 MB of
   resident memory on these sources (P1 189.4, P2 190.9, P3 198.2, P4 210.8 MB)
   and compilation runs inside the submission container. The knapsack source
   compiles in 60.6 MB and therefore does reach execution, which is why C++
   shows both `SE` and `MLE`.
2. **The watermark window is too narrow.** Every knapsack cell failed in every
   language under every strategy, including the promoting ones. Promotion
   completed at 253-794 ms but the program crosses 128 MiB first. The window is
   38.4 MiB at this tier against 76.8 MiB at the shipped tier.
3. **The JVM heap allowance contradicts the container cap**, as in section 1.

## 3. Corpus label is not the classifier

The XGBoost classifier's actual routing, scored against ground truth. Only
`P5_knapsack_2d_dp` is genuinely memory-heavy (4 language cells).

| | predicted heavy | predicted light |
|---|---|---|
| **truly heavy (4)** | 0 | 4 |
| **truly light (14)** | 8 | 6 |

Accuracy **33.3%**. Always predicting "light" would score **77.8%**. The
classifier is anti-correlated on this corpus: it misses every heavy cell and
raises 8 false alarms.

Reproduced independently of the harness, by direct API probes:

- `large-buffer-allocation` (206 MB peak) -> predicted **low**
- `heavy-collections` (9.6 MB peak) -> predicted **high**

This calibration corpus is tiny (18 programs) and sits on the CodeContests
distribution. On its own CodeNet distribution the retrained unified model
reaches **87.38%** held-out accuracy and the specialised C++ model **94.39%**
(section 8), so the 33.3% above is distribution shift on a small corpus rather
than model failure. Note that the table itself was measured with the
pre-retrain classifier; how the v2 models route this specific 18-program corpus
is **UNVERIFIED** — confirming it would need a re-run of the calibration corpus
against the deployed v2 models.

## 4. 10,000-submission contest simulation

Seeded. Reserved memory and reserved CPU-core-hours.

### 256 MiB tier

| Strategy | Reserved GB | Saved vs static | Reserved core-hrs | Wasted | P95 turnaround |
|---|---|---|---|---|---|
| Baseline | 20,000.00 | — | 4.337 | 97.47% | 1187.0 ms |
| Predictive | 11,953.50 | 40.23% | 3.620 | 95.76% | 1190.5 ms |
| **Reactive** | **10,557.00** | **47.22%** | **3.379** | 95.76% | 1185.9 ms |
| Hybrid | 15,509.50 | 22.45% | 3.939 | 96.77% | 1192.0 ms |

### 128 MiB tier

| Strategy | Reserved GB | Saved vs static | Reserved core-hrs | Wasted |
|---|---|---|---|---|
| Baseline | 20,000.00 | — | 4.400 | 97.48% |
| Predictive | 11,498.75 | 42.51% | 3.311 | 97.24% |
| **Reactive** | **10,013.75** | **49.93%** | **2.632** | 97.05% |
| Hybrid | 15,308.75 | 23.46% | 3.626 | 97.95% |

**Reactive reserves ~1.5x less than Hybrid.** Hybrid and Predictive share one
classifier for the initial tier, so they start the identical set of submissions
uncapped, and promotion can only raise a reservation. Hybrid therefore reserves
strictly more than Predictive: it pays for the classifier's false alarms and
still needs the watermark. Prediction is not free — it costs reservation.

Steady-state P95 turnaround spans 1185.9-1192.0 ms across all four strategies,
a spread inside run-to-run noise.

## 5. 500-submission freeze burst

### 256 MiB tier (56 adaptive slots)

| Scenario | Slots | Avg queue wait | P95 turnaround | Drain |
|---|---|---|---|---|
| Baseline (safe) | 7 | 12,386.2 ms | 24,918.4 ms | 56.5 s |
| Baseline (overcommitted) | 14 | 250.4 ms | 1,783.0 ms | 31.6 s |
| Adaptive (all three) | 56 | 0.0 ms | ~1190 ms | ~31.2 s |

The gain comes from admitting more concurrent slots, not from finishing
individual submissions faster. Overcommitting the baseline to 14 slots also
recovers most of the wait, but at 186% host memory utilisation.

### 128 MiB tier (112 adaptive slots)

Baseline 7 slots: 12,360.4 ms wait, 57.1 s drain. Overcommitted 14 slots:
300.5 ms, 31.9 s. Adaptive: 0.0 ms, ~1200 ms, 31.2 s.

## 6. AWS EC2 provisioning projection (theoretical)

`c6i.4xlarge` (16 vCPU, 32 GB), USD 0.68/hr, 87.5% packing. Not a deployment.

| Dimension | Baseline | 256 MiB | 128 MiB |
|---|---|---|---|
| Per-pod memory | 2048 MiB | 256 MiB (**8.0x**) | 128 MiB (**16.0x**) |
| Per-pod CPU | 2.0 vCPU | 1.0 vCPU (2.0x) | 1.0 vCPU (2.0x) |
| Pods per VM | 14 | 112 (8.0x) | 224 (16.0x) |
| VMs for 500-sub burst | 36 | 5 (**86.1%**) | 3 (**91.7%**) |
| Cluster hourly cost | USD 24.48 | USD 3.40 | USD 2.04 |
| **Hourly saving** | — | **USD 21.08** | **USD 22.44** |
| 10k RAM reserved | 20,000 GB | 10,557.00 GB (**47.22%**) | 10,013.75 GB (49.93%) |

The 256 MiB column is the deployable result. The 128 MiB tier does not work
(section 2), so its larger saving is a sensitivity figure, not a proposal.

## 7. Two data eras, and what their labels mean

The classifier has been trained twice, on two different corpora, and the
figures from the two eras are not interchangeable. Each set is labelled below.

### CodeContests — the first era (`model-training/codecontests_subset`)

| Property | Value |
|---|---|
| Size | 30,000 files (5,000 Light / 5,000 Heavy per language, C++ / Java / Python) |
| Problems | **148 unique** (~200 solutions per problem) |
| Labels | **heuristic, not measured** |
| Heaviest program | nothing exceeds 50 MiB measured |

The label is assigned by `extract_codecontests.py`: **Light** when
`difficulty <= 3` or `len(code) < 650`, **Heavy** when `difficulty >= 5` or
`len(code) >= 1200`, and the remainder is balanced to a 5,000/5,000 quota.
Every label in this corpus is therefore a statement about problem difficulty and
source length. **No label in this corpus was ever measured.**

### CodeNet — the second era (`model-training/codenet_subset`)

| Property | Value |
|---|---|
| Size | 164,686 submissions, 737 MB (Light 100,000 / Heavy 64,686) |
| Problems | **2,520 unique** -> the test split measures generalisation to unseen problems |
| Per language | Python 25,000/25,000; C++ 25,000/25,000; Java 25,000/14,258; C 25,000/428 (Light/Heavy) |
| Labels | **measured memory** — Light < 25 MiB, Heavy >= 100 MiB; the ambiguous 25-100 MiB band is dropped, not guessed |
| Source | `iNeil77/CodeNet` on HuggingFace (per-language parquet; `memory` in KB, plus `cpu_time`, `status`, `code_size`); 12.7M rows scanned |

Directory layout mirrors the Rust walker:
`<root>/<C|C++|Java|Python>/<Light|Heavy>/<file>`.

### Why the label change was necessary

Correlation between source length and measured memory over the full 12.7M-row
scan:

| Language | r | r^2 | n |
|---|---|---|---|
| Python | 0.4278 | 0.183 | 3,256,346 |
| Java | 0.2615 | 0.068 | 627,617 |
| C | 0.3107 | 0.097 | 622,644 |
| C++ | 0.3648 | 0.133 | 7,552,553 |

Length explains only **7-18% of the variance** of memory, so a length threshold
is a weak proxy for the quantity being predicted. That is the specific defect of
the CodeContests labels, and it is why the misroute figures below are reported
against measured-memory labels.

## 8. The v2 models

The feature vector grew, but see section 9 for how much that mattered.

| Vector | Per-language | Unified |
|---|---|---|
| old | 22 base AST + 10 engineered = **32** | **36** |
| new | 26 base AST + 10 engineered = **36** | **40** (+4 language one-hots) |

The old vector's only allocation signal was `large_alloc_flag`, a boolean
thresholded at `LARGE_ALLOC_THRESHOLD = 1_000_000`. The four added features are
measured aggregates, not booleans:

| Feature | Meaning |
|---|---|
| `alloc_size_max` | largest single statically-known allocation size at any site |
| `alloc_size_total` | sum of statically-known sizes across all sites |
| `alloc_sites` | number of recognised allocation sites |
| `alloc_unknown_sites` | sites whose size is only known at runtime |

**Units differ by language and the field names are deliberately neutral.** C
`malloc`/`calloc` sizes are bytes; Java/Python `new T[n]` and container
capacities are element counts. The field names are `size_max`/`size_total`
rather than `bytes_max` for this reason; calling them bytes would be a false
claim. `large_alloc_flag` is now derived (`AllocStats.any_large()`), so its
semantics and existing tests hold unchanged.

The training split is identical across runs: **125,159 train / 1,930 problems**
and **28,687 test / 483 problems**, 80/20 problem-grouped via
`GroupShuffleSplit`, seed 42. 10,840 parse-error rows were filtered (6.6%) —
CodeNet is messier than CodeContests.

v2 per-model metrics (the numbers the trainer prints):

| model | CV Acc | Test Acc | Test F1 | Test AUC | CV threshold |
|---|---|---|---|---|---|
| Unified | 89.56% | 87.38% | 88.48% | 0.9503 | 0.409 |
| C++ | 96.07% | 94.39% | 96.26% | 0.9904 | 0.645 |
| Java | 86.73% | 84.19% | 83.30% | 0.9172 | 0.452 |
| Python | 85.02% | 83.71% | 86.51% | 0.9221 | 0.441 |
| C | 97.76% | 96.64% | 41.12% | 0.9387 | 0.200 |

The C row's F1 is the tell: C is **data-limited** (428 Heavy examples in a
12.7M-row corpus, median C memory 0.6 MiB). A specialised C model is trained and
exported, but `predict.rs` routes C through the **unified** model, so that
artifact is unused. Do not claim a C-specific classifier is in service.

Feature importance on the new models, for the record:

- Java: `alloc_unknown_sites` **0.4060 (rank 1 of 36)**; `alloc_sites` 0.0691
  (rank 2); `large_alloc_flag` 0.0094 (rank 29).
- Unified: `alloc_unknown_sites` 0.0352 (rank 9 of 40); `large_alloc_flag`
  0.0032 (**rank 40, last**).
- Python: `alloc_unknown_sites` 0.0339 (rank 7 of 36).
- C++: the whole allocation family sums to 0.0087; `large_alloc_flag` 0.0000
  (rank 36, last).

## 9. Routing: the headline result

A **misroute** is a genuinely-Heavy program sent to the Low tier. It is the
metric that matters, because accuracy is dominated by the Light majority.

All three rows use the **same decision rule** — the thresholds currently in
`server/src/predict.rs`: `THRESHOLD_UNIFIED` 0.319, `THRESHOLD_CPP` 0.346,
`THRESHOLD_JAVA` 0.257, `THRESHOLD_PYTHON` 0.200 — and the same
problem-disjoint test split. Only the model changes.

| model set | features | C | C++ | Java | Python | all | routed High |
|---|---|---|---|---|---|---|---|
| original shipped | 32 | 17.9% | 9.8% | 15.5% | 33.5% | **23.7%** | 52.6% |
| CodeNet labels only | 32 | 46.4% | 4.7% | 8.1% | 6.3% | **6.4%** | 59.4% |
| + allocation features | 36 | 25.0% | 4.8% | 7.5% | 6.3% | **6.3%** | 59.1% |

Attribution — and this is the honest reading of the result: the **label fix did
almost all of the work** (23.7% -> 6.4%). The allocation features are
near-neutral overall (6.4% -> 6.3%) and they cut C's misroute from 46.4% to
25.0%, because C routes through the unified model, which received them. The
retrain's win is the **labels**, not the features; the features should not be
presented as the headline.

## 10. Is a misroute actually a failure?

The Heavy label means ">= 100 MiB", but the Low tier's hard limit is
**256 MiB**. A program measured between those two numbers is labelled Heavy;
routing it to Low is counted as a misroute, yet it completes inside Low anyway.

Of the 945 misrouted programs in the test split (memory 100.0-955.9 MiB, median
142.4):

- **834 (88.3%) would have fit inside the 256 MiB Low tier**
- **111 (11.7%) would have exceeded it**

By band: 100-150 MiB 621; 150-200 MiB 127; 200-256 MiB 86; 256-512 MiB 71;
512-1024 MiB 40.

So the raw 6.3% decomposes into **0.7% genuine over-limit failures** and **5.5%
boundary artefacts**. Caveat that must be stated wherever this is used:
CodeNet's `memory` was measured on IBM/Aizu hardware, not on this project's
cgroup judge, so absolute MiB does not map one-to-one. The honest range is
**0.7%-6.3% real failures**; 0.7% is the optimistic end.

Two mechanisms reduce the real-world impact further, and both are verified in
code:

- The promotion watermark is `HIGH_WATERMARK_PCT = 70` of the Low limit =
  **179.2 MiB** (`server/src/docker.rs`). Most misroutes between 150 and 256 MiB
  therefore cross the watermark and are promoted before they OOM.
- **Java cannot be rescued this way**: `-Xmx` is fixed at JVM launch, so for
  Java a misroute is fatal regardless of promotion. Java's figure should NOT be
  discounted the way the aggregate can be.

## 11. Thresholds: accuracy-optimal is the wrong objective

The trainer selects thresholds to maximise accuracy, which the Light majority
dominates. Applied to the new models, that choice makes routing *less* safe:

| threshold set | all misroute | routed High |
|---|---|---|
| deployed (tuned for the old models) | 6.3% | 59.1% |
| each model's own CV-optimal | 13.4% | 51.0% |
| misroute-minimising | 2.2% | 70.4% |

The misroute-minimising row is **degenerate** — it pins every threshold to the
0.05 grid floor, i.e. "route almost everything High" — which shows misroute
alone is not a usable objective; it has to be balanced against
over-provisioning. The deployed thresholds are the safer operating point, so
the new models ship with the **existing** thresholds.

## 12. Claims that did not survive

Earlier drafts of this document, and of the companion evaluation report,
asserted figures that did not survive re-measurement. Only the corrected values
are reproduced here; the refuted figures themselves are recorded in the artifact
history, not in this document.

| Item | Corrected result |
|---|---|
| RAM saved (aggregate) | Reactive **47.22%**; Hybrid 22.45% |
| CPU saving | Reactive **22.09%**; Hybrid 9.18%. Metric is *reserved* core-hours, not consumed |
| Adaptive slots at 256 MiB | **56** (14,336 / 256) |
| Safe-baseline average queue wait | **12,386.2 ms** |
| Burst turnaround | **24,918.4 -> 1190.8 ms (20.9x)** |
| Burst drain | **56.5 -> 31.2 s** |
| Hourly cloud saving | **USD 21.08/hr** |
| VM fleet reduction | **7.2x** (36 vs 5) |
| Feature counts | **22 + 10** (old), **26 + 10** (new); 32/36 and 36/40 per-language/unified |
| Model accuracy | v2 unified **87.38%** test accuracy, C++ **94.39%** (section 8). Earlier artifact sets that conflicted on every model are superseded by this retrain |
| Promotion target | limits are raised to **uncapped**, not to a 2048 MiB Tier 2 |
| Promotion completion | **253-794 ms** across 16 promoted runs |
| Java OOM verdict | cgroup **MLE** |
| 128 MiB tier safety | False. C++ cannot compile there; the knapsack fails in every language |
| "over 90% of solutions used less than 25 MiB" | **76.8%** on this corpus |
| "C, C++, Python are all safe at 128 MiB" | False at that tier; C++ cannot compile and the knapsack fails in every language |
