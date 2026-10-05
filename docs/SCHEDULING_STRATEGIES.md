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

> **Results withdrawn.** The experimental numbers formerly in this section have been
> removed. They were produced by a harness whose corpus, cloud target and promotion path
> changed repeatedly, and the figures no longer correspond to any single run. A fresh
> experimental programme is specified in [`docs/TEST_PLAN.md`](TEST_PLAN.md); results will be
> re-derived and re-published against it. Do not cite numbers from this repository's history.

