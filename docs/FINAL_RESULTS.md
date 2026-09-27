# RAAS-OCJS — Final Verified Results

Every number below traces to a CSV in `benchmarks/`, produced by a live run
against the judge. Two configurations were measured: the shipped 256 MiB light
tier, and a 128 MiB tier used only to locate the floor.

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

On its own CodeNet distribution the unified model reaches 94.45% held-out
accuracy and the specialised C++ model 95.45% (`model_comparison.csv`), so this
is distribution shift on a small corpus, not model failure.

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

## 7. Claims that did not survive

| Original claim | Status |
|---|---|
| "over 90% of solutions used less than 25 MiB" | **76.8%** — not supportable |
| "94.45% held-out accuracy" | Unsupported by the paper's own Table II. The table matches `model-training/README.md` (unified **83.71%**), not `artifacts/model_comparison.csv` (**94.45%**). The two artifact sets conflict on every model (C++ 90.01 vs 95.45, Python 79.45 vs 98.88). README states the most recent run is authoritative; the CSV's test-over-CV gap (84.73 → 94.45) suggests a different or leakier split |
| 47.68% RAM saved | 47.22% is **Reactive's**; Hybrid is 22.45% |
| 21.58% CPU saving | 22.09% is Reactive's; Hybrid is 9.18%. Metric is *reserved* core-hours, not consumed |
| 28 adaptive slots | **56** at 256 MiB (14,336 / 256) |
| 15,253.0 ms queue wait | **12,386.2 ms** |
| 29,011.2 -> 1,311.0 ms (22.1x) | **24,918.4 -> 1190.8 ms (20.9x)** |
| 61.0 -> 31.0 s drain | **56.5 -> 31.2 s** |
| USD 19.04/hr saving | **USD 21.08/hr** |
| "roughly 4x more VMs" | **7.2x** (36 vs 5) |
| 18 structural + 6 engineered features | **22 + 10** |
| Promotion expands to 2048 MiB Tier 2 | Promotion lifts limits to **uncapped** |
| Promotion in "under 3 ms" | 253-794 ms across 16 promoted runs |
| "9,332 Tier 1 / 668 promotions" | Not reproducible from any data file |
| Java verdict `java.lang.OutOfMemoryError` | cgroup `MLE`; that string cannot come from a Java submission |
| 2.8/2.9/3.1 ms promotions at 128 MiB | 253-794 ms |
| "C, C++, Python are all safe at 128 MiB" | False. C++ cannot compile there; the knapsack fails in every language. The first run only appeared to pass because Docker's default swap doubled the tier |
