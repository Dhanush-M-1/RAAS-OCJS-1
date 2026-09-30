# Scheduling Strategies in RAAS-OCJS

RAAS-OCJS evaluates four distinct scheduling paradigms against a common containerized execution substrate. This document details each strategy's operational semantics, decision criteria, trade-offs, and empirical behavior.

---

## 1. Strategy Taxonomy

| Strategy | Static Analysis | Initial Tier | Runtime Monitoring | Live Migration | Key Strength | Main Trade-off |
|---|---|---|---|---|---|---|
| **Baseline** | None | Heavy (Uncapped) | None | No | Maximum execution headroom; zero false OOMs | Heavy resource hoarding; limits queue concurrency |
| **Predictive** | Tree-sitter + XGBoost (static) | ML Predicted (Light/Heavy) | None | No | Fast, zero runtime polling overhead | Vulnerable to mispredictions without live safety net |
| **Reactive** | None | Light (256 MiB) | cgroup v2 `memory.high` (2 ms poll) | Yes | Self-correcting under memory pressure | 2 ms polling overhead; reaction latency |
| **Hybrid** | Tree-sitter + XGBoost (static) | ML Predicted (Light/Heavy) | cgroup v2 `memory.high` (if Light) | Yes | Best of both: optimal start + live safety net | Combines static analysis and monitoring logic |

---

## 2. In-Depth Strategy Analysis

### 2.1 Baseline Strategy
- **Mechanism**: Replicates the current state-of-the-art in most self-hosted online judges. Every submission is treated as potentially resource-exhaustive.
- **Resource Limits**:
  - `memory`: Unlimited (Host Memory)
  - `cpus`: Unlimited (All available host CPU cores)
- **Container Lifecycle**:
  ```
  Submission Intake -> Spawn Uncapped Container -> Execute -> Collect Metrics -> Terminate
  ```
- **Observed Behavior**:
  - For small problems ($O(1)$ prefix sums), Baseline holds an uncapped container for work that would fit in the 256 MiB Low tier, at no performance benefit. (Per-strategy footprint delta: UNVERIFIED - needs measurement.) Across the 400 live runs, peak container usage was min 6.3 MB, mean 14.4 MB, median 10.6 MB, max 53.0 MB, with 95.8% of cases under 25 MB.
  - On multi-tenant systems, concurrent Baseline submissions compete aggressively for host CPU scheduling and cache lines.

---

### 2.2 Predictive Strategy
- **Mechanism**: Uses pre-execution static analysis.
  1. Tree-sitter parses the source code into an Abstract Syntax Tree.
  2. Feature extractor scans the AST in a single pass to compute 26 structural metrics, including four measured allocation aggregates (`alloc_size_max`, `alloc_size_total`, `alloc_sites`, `alloc_unknown_sites`); the training pipeline then adds 10 engineered density ratios (36 features per language, 40 for the unified model).
  3. Pre-compiled XGBoost inference function (`score(features)`) outputs a probability score $P(\text{Heavy})$.
  4. If $P(\text{Heavy}) \ge \tau_{\text{lang}}$, assign `Tier::High`; otherwise, assign `Tier::Low`.
- **Thresholds** (from `model-training/artifacts/model_comparison.csv`):
  - Python: $\tau = 0.200$
  - Java: $\tau = 0.257$
  - C++: $\tau = 0.346$
  - C: $\tau = 0.319$ (scored by the unified model)
- **Model Selection**: Python, Java, and C++ use their specialised models. **C is
  scored by the unified multi-language model** — `predict.rs` routes `Language::C`
  through `model_unified` with `THRESHOLD_UNIFIED` (0.319). A C-specialised model
  is trained and exported but is not routed through, so no C-specific classifier
  is in service; C is data-limited (428 Heavy examples in the 12.7M-row CodeNet
  scan). The unified vector adds the 4 language one-hot columns that make it 40
  features rather than 36.
- **Resource Limits**:
  - If Light: `memory = 256m`, `memory-swap = 256m`, `cpus = 1.0`
  - If Heavy: `memory = uncapped`, `cpus = uncapped`
- **Container Lifecycle**:
  ```
  Submission Intake -> AST Feature Extraction -> XGBoost Score -> Select Tier -> Spawn Container -> Execute -> Collect Metrics
  ```
- **Observed Behavior**:
  - Routes Light submissions (e.g. Range Prefix Sums, Top-K Streaming) into the Low tier. In the N=10,000 macro simulation the Predictive policy reserved 14860.25 GB against Baseline's 20000.0 GB — a saving of 5139.75 GB (25.7%).
  - Model inference runs in pure compiled Rust on the hot path, without subprocesses. (Inference latency: UNVERIFIED - needs measurement.)

---

### 2.3 Reactive Strategy
- **Mechanism**: Ignores static code properties and assumes every submission is Light (`Tier::Low`) by default. Relies on the Linux kernel's cgroup v2 event mechanism to detect actual memory consumption.
- **Trigger is memory-only**: the monitor watches the `memory.events` `high` counter and `memory.current`. A purely CPU-bound submission is never promoted, so under Reactive it stays pinned to `--cpus=1`.
- **Watermark Architecture**:
  - Hard limit (`memory.max`): 256 MiB
  - Soft watermark (`memory.high`): 179.2 MiB (70% of `memory.max`)
- **Monitoring Loop**:
  - Spawns an asynchronous monitoring task polling every 2 ms.
  - Checks if `memory.current >= 179.2 MiB` (70%) or if the kernel has incremented the monotonic `high` counter in `memory.events`.
  - When the threshold is crossed, the moderator lifts the limits in place: it writes `memory.high=max` and `memory.max=max` directly to the container's host cgroup directory (`promote_to_unlimited()`), then issues `docker update --memory 0 --memory-swap -1 --cpus 0` so the Docker daemon's accounting agrees. (If the host cgroup directory is unreachable — e.g. Docker Desktop in a VM — only the `docker update` fallback runs.)
- **Container Lifecycle**:
  ```
  Spawn Light Container (256M) -> Set memory.high=179.2M (70%) -> Start Exec Task || Start Monitor Task ->
     [If cur >= 179.2M] -> Live promote to Uncapped -> Continue Executing -> Complete
  ```
- **Observed Behavior**:
  - In the 0-1 Knapsack 2D DP workload, the container starts with 256 MiB. As its large DP table is allocated and touched, the monitor catches the breach at the 179.2 MiB threshold and lifts the limits without interrupting execution. Measured, the same heavy program is promoted and passes at 171-203 MB in C/C++/Python. For JVM submissions the Low-tier heap is derived as `-Xmx` = 75% of the hard limit (192m); before this derivation a hardcoded `-Xmx512m` inside the 256 MiB container was OOM-killed on the same workload (255.9 MB peak).

---

### 2.4 Hybrid Strategy
- **Mechanism**: The synthesis of Predictive and Reactive scheduling.
  1. Evaluates XGBoost prediction before execution.
  2. If predicted **Heavy**, starts directly in `Tier::High` (avoiding soft watermark checks).
  3. If predicted **Light**, starts in `Tier::Low` **with** the reactive monitor armed.
  4. If the model underestimated the submission's memory usage (false negative), the reactive monitor catches the spike at 179.2 MiB (70%) and promotes the container live.
- **Advantage**: Eliminates both the cost of over-allocating Light submissions and the risk of OOM kills on misclassified Heavy submissions.

---

## 3. Comparative Benchmark Summary

| Scenario | Baseline | Predictive | Reactive | Hybrid |
|---|:---:|:---:|:---:|:---:|
| Light Task (Prefix Sums) | Over-allocates (Heavy) | Optimal (Light) | Optimal (Light) | Optimal (Light) |
| Large DP (Knapsack, 171-203 MB) | Heavy from start | Depends on AST | Starts Light $\rightarrow$ Promotes live | Starts Light $\rightarrow$ Promotes live |
| CPU Intensive ($O(V^3)$ APSP) | Heavy from start | Classified from AST | Starts Light (1 CPU, **never promotes** — trigger is memory-only) | Classified or Monitored (promotion is memory-only) |
| Heavy STL (priority_queue-heavy) | Over-allocates (Heavy) | Identified (`has_heavy_datastructure`) | Light (fits in 256 MB) | Optimal (Light, 256 MB) |

### 3.1 Measured Results (2026-09-30)

**Live runs** (100 CodeContests submissions × 4 strategies, 256 MiB tier): 400/400 AC, 0 transport failures, 394 s total, 0 promotions. Started high 246 / low 154. Peak container usage: min 6.3 MB, mean 14.4 MB, median 10.6 MB, max 53.0 MB; 95.8% under 25 MB.

**Macro simulation** (N = 10,000, seed 42, 43 real problems, 256 MiB):

| Strategy | Slots | Memory allocated | Wasted | CPU core-hours | Avg queue wait | P95 turnaround | Saved vs Baseline |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| Baseline | 7 | 20000.0 GB | 99.4% | 5.622 | 2.36 ms | 1448.52 ms | — |
| Predictive | 56 | 14860.25 GB | 99.21% | 4.827 | 0.0 | 1434.47 ms | 5139.75 GB (25.7%) |
| Reactive | 56 | 2500.0 GB | 95.33% | 2.876 | 0.0 | 1471.9 ms | 17500.0 GB (87.5%) |
| Hybrid | 56 | 14860.25 GB | 99.2% | 4.831 | 0.0 | 1418.68 ms | 5139.75 GB (25.7%) |

Live promotions were 0 for all four strategies. Slots are derived, never hardcoded: the baseline-safe count is `floor(14336/2048) = 7` and the adaptive count is `floor(14336/256) = 56` on a 15 GiB host (14336 MiB usable).

**Burst stress** (N = 500, 30 s window, 15 GiB host): Baseline (safe, 7 slots) avg queue wait 20966.9 ms, P95 turnaround 41794.6 ms, drain 73.8 s, util 93.3%. Baseline overcommitted (14 slots) wait 2996.0 ms, P95 7401.6 ms, drain 37.7 s, util 186.7%. Predictive (56 slots) wait 0.0, P95 1421.8 ms, drain 31.3 s. Reactive wait 0.0, P95 1482.8 ms, drain 31.2 s. Hybrid wait 0.0, P95 1394.5 ms, drain 31.3 s.

**Cloud projection**: per-pod memory 2048 → 256 MiB (8.0x); CPU 2.0 → 1.0 vCPU (2.0x); density on c6i.4xlarge (32 GB) 14 → 112 pods (8.0x); 500-sub burst fleet 36 → 5 VMs (86.1%); cost at USD 0.68/hr 24.48 → 3.40 USD/hr (21.08 saved, 86.1%); reserved RAM over 10,000 submissions 20000.0 → 2500.0 GB (87.5%).

**Per-language** (Baseline → Reactive, shares among the simulated submissions):

| Language | n | Share | Avg CPU (ms) | Avg wall (ms) | P95 (ms) | Waste |
|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| C++ | 6743 | 67.4% | 115.7 → 116.4 | 1219.1 → 1216.8 | 1514.1 → 1526.8 | 99.47% → 95.87% |
| Java | 1008 | 10.1% | 290.2 → 292.1 | 876.8 → 1127.9 | 1003.8 → 1332.8 | 98.82% → 90.99% |
| Python | 2249 | 22.5% | 152.9 → 153.0 | 451.4 → 449.3 | 663.2 → 659.5 | 99.46% → 95.65% |

All three languages save 87.5% under Reactive.

**Verdict probes**: AC, WA, RE, TLE, MLE all verified from ground truth (5/5); TLE killed at ~10.03 s CPU; MLE peaks ~255.5-256.0 MB in the low tier.

Source data is produced by `benchmarks/raas_benchmark.py`, which writes `benchmarks/results/real_dataset_*_tier256.csv`.
