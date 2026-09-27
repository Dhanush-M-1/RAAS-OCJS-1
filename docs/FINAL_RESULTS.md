# RAAS-OCJS — Final Verified Results

Every number below traces to a CSV in `benchmarks/` produced by a live run
against the judge. Two configurations were measured: the shipped 256 MiB light
tier, and a 128 MiB tier used only for the boundary experiment.

- Judge: `http://192.168.0.111:3000` (calibration host)
- Testbed: 13th Gen Intel Core i5-13420H, 12 threads, 15 GiB usable RAM, Fedora
- Simulation seed: `BENCH_SEED=42`, so every figure is reproducible
- Tier override: `LOW_TIER_MB` (default 256). Watermark is 70% of the tier:
  179.2 MiB at 256, 89.6 MiB at 128.

## Metrology note

Memory is charged as **reserved limit**, not peak RSS. A submission started in
the low tier is charged the tier size; one started in the uncapped tier, or
promoted during execution, is charged the baseline 2048 MiB ceiling as a
worst-case reservation. `Tier::High` is uncapped (`--memory 0`), so it carries
no finite reservation and cannot be counted as dense.

Allocation is taken from the judge's own reported `tier_started` and
`tier_promoted` per submission — never from the corpus's `is_heavy` label.
Those two disagree in 20 of 72 cells; see "Corpus label is not the classifier"
below.

## 1. Empirical runs, shipped 256 MiB tier

72 live submissions: 18 programs (6 problems x 3-4 languages) x 4 strategies.

| Verdict | Count |
|---|---|
| AC | 72 |
| Promotions | 8 |

- **75.0%** of AC runs (54/72) used under 25 MiB; mean 59.1 MB, min 5.95 MB,
  max 256.1 MB.
- Of the 38 runs that started in the low tier, **30 (78.9%)** never needed
  promotion.

## 2. Empirical runs, 128 MiB experimental tier

| Verdict | Count |
|---|---|
| AC | 69 |
| MLE | 3 |

All three failures are Java on the 2D knapsack, in Predictive, Reactive and
Hybrid alike.

| Language | Promotion (promoted strategies) | Verdict | Baseline peak |
|---|---|---|---|
| Python | 248 / 267 ms | AC | 221.7 MB |
| C | 300 / 326 ms | AC | 199.6 MB |
| C++ | 500 / 578 ms | AC | 205.0 MB |
| Java | 797 / 808 ms | MLE | 253.8 MB |

Java's promotion is recorded as completing, and it still ends in MLE. The
watermark fires and the limit is lifted, but only after HotSpot has committed
enough heap to trip the killer. This separates "the mechanism works" from "the
mechanism is fast enough for this runtime", and is the reason 256 MiB is the
shipped tier.

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

This is why Reactive/Reactive-style on-demand promotion outperforms
prediction: starting everything bounded and escalating on demonstrated demand
beats guessing from AST features.

## 4. 10,000-submission contest simulation

Seeded. Reserved memory and reserved CPU-core-hours.

### 256 MiB tier

| Strategy | Reserved GB | Saved vs static | Reserved core-hrs | Wasted | P95 turnaround |
|---|---|---|---|---|---|
| Baseline | 20,000.00 | — | 4.551 | 97.44% | 1248.89 ms |
| Predictive | 11,901.00 | 40.49% | 3.781 | 95.76% | 1250.65 ms |
| **Reactive** | **10,485.25** | **47.57%** | **3.527** | 95.25% | 1250.59 ms |
| Hybrid | 15,457.00 | 22.71% | 4.156 | 96.75% | 1253.98 ms |

### 128 MiB tier

| Strategy | Reserved GB | Saved vs static | Reserved core-hrs | Wasted |
|---|---|---|---|---|
| Baseline | 20,000.00 | — | 4.832 | 97.51% |
| Predictive | 11,326.25 | 43.37% | 4.074 | 97.16% |
| **Reactive** | **9,846.88** | **50.77%** | **3.931** | 96.74% |
| Hybrid | 15,136.25 | 24.32% | 4.408 | 97.86% |

**Reactive reserves ~1.5x less than Hybrid.** Hybrid pre-assigns every
submission it predicts heavy to the uncapped tier; those carry no reservation
and must be charged worst-case. Prediction is not free — it costs reservation.

Steady-state P95 turnaround is within 5 ms across all strategies, so the
saving costs no latency.

## 5. 500-submission freeze burst

### 256 MiB tier (56 adaptive slots)

| Scenario | Slots | Avg queue wait | P95 turnaround | Drain |
|---|---|---|---|---|
| Baseline (safe) | 7 | 13,879.4 ms | 27,410.7 ms | 59.3 s |
| Baseline (overcommitted) | 14 | 437.0 ms | 2,180.5 ms | 31.8 s |
| Adaptive (all three) | 56 | 0.0 ms | ~1252 ms | ~31.1 s |

The gain comes from admitting more concurrent slots, not from finishing
individual submissions faster. Overcommitting the baseline to 14 slots also
recovers most of the wait, but at 186% host memory utilisation.

### 128 MiB tier (112 adaptive slots)

Baseline 7 slots: 15,722.8 ms wait, 63.7 s drain. Adaptive: 0.0 ms, ~31.2 s.

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
| 10k RAM reserved | 20,000 GB | 10,485.25 GB (**47.57%**) | 9,846.88 GB (50.77%) |

The 256 MiB column is the deployable result. 128 MiB is not safe for JVM
submissions, so its larger saving is a sensitivity figure, not a proposal.

## 7. Claims that did not survive

| Original claim | Status |
|---|---|
| "over 90% of solutions used less than 25 MiB" | **75.0%** — not supportable |
| 47.68% RAM saved | 47.57% is **Reactive's**; Hybrid is 22.71% |
| 21.58% CPU saving | 22.50% is Reactive's; Hybrid is 8.68%. Metric is *reserved* core-hours, not consumed |
| 28 adaptive slots | **56** at 256 MiB (14,336 / 256) |
| 15,253.0 ms queue wait | **13,879.4 ms** |
| 29,011.2 -> 1,311.0 ms (22.1x) | **27,410.7 -> ~1252 ms (21.9x)** |
| 61.0 -> 31.0 s drain | **59.3 -> 31.0 s** |
| USD 19.04/hr saving | **USD 21.08/hr** |
| "roughly 4x more VMs" | **7.2x** (36 vs 5) |
| 18 structural + 6 engineered features | **22 + 10** |
| Promotion expands to 2048 MiB Tier 2 | Promotion lifts limits to **uncapped** |
| Promotion in "under 3 ms" | 248-808 ms, runtime-dependent |
| "9,332 Tier 1 / 668 promotions" | Not reproducible from any data file |
| Java verdict `java.lang.OutOfMemoryError` | cgroup `MLE`; that string cannot come from a Java submission |
| 2.8/2.9/3.1 ms promotions at 128 MiB | 248-808 ms |
