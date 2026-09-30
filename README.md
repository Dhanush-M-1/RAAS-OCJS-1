# RAAS-OCJS

**Resource-Aware Adaptive Scheduling for Online Competitive Judge Systems**

A next-generation competitive programming judge combining **predictive AST-based classification** with **reactive, event-driven Linux cgroup v2 monitoring** to enable dynamic, mid-execution isolation-tier migration on fixed, self-hosted hardware.

> **Final-Year Project**  
> Department of Computer Science and Engineering, Easwari Engineering College.  
> Guided by **Mrs. Indumathy P**, Assistant Professor / CSE.

---

## Table of Contents

- [01. Problem Statement](#01-problem-statement)
- [02. Scheduling Strategies](#02-scheduling-strategies)
- [03. System Architecture](#03-system-architecture)
- [04. Real-World Competition Benchmark Suite](#04-real-world-competition-benchmark-suite)
- [05. Key Innovations & Measurements](#05-key-innovations--measurements)
- [06. Experimental Results](#06-experimental-results)
- [07. Tech Stack](#07-tech-stack)
- [08. Documentation Index](#08-documentation-index)
- [09. Quick Start](#09-quick-start)
- [10. Team](#10-team)

---

## 01. Problem Statement

Standard Online Judges (DOMjudge, DMOJ, VJudge) apply uniform isolation limits to every submission. Evaluating a trivial $O(1)$ query reserves identical CPU and memory headroom as an intensive $O(N^3)$ graph or dynamic programming algorithm. On self-hosted, non-elastic servers, this practice leads to:
1. **Severe Resource Hoarding**: Light jobs tie up server memory reservations, capping submission throughput.
2. **OOM Vuln or Overkill**: Setting limits low causes false Memory-Limit-Exceeded (MLE) verdicts on legitimate heavy programs; setting limits high creates multi-tenant CPU starvation.

**RAAS-OCJS** introduces adaptive multi-tier scheduling that optimizes the sandbox isolation substrate beneath judging without compromising grading criteria.

---

## 02. Scheduling Strategies

RAAS-OCJS provides four switchable scheduling engines:

| Strategy | When Evaluated | Mechanism | Isolation Profile |
|---|---|---|---|
| **Baseline** | Intake | Current standard practice | Always assigns Heavy tier (Uncapped Host Memory & CPU) |
| **Predictive** | Pre-Execution | Tree-sitter AST $\rightarrow$ 32 features $\rightarrow$ Compiled XGBoost | Assigns Light (256 MiB, 1 CPU) or Heavy tier before launching container |
| **Reactive** | Mid-Execution | Linux cgroup v2 event-driven monitoring | Starts in Light tier (256 MiB); promotes to Uncapped (memory *and* CPU caps lifted) once the `memory.events` `high` counter crosses the 70% (~179.2 MiB) watermark. The trigger is memory pressure only - a purely CPU-bound submission is never promoted |
| **Hybrid** | Both | Predictive start + Reactive live safety net | Starts in ML-predicted tier; actively promotes if memory spikes exceed prediction |

---

## 03. System Architecture
```mermaid
flowchart TD
    A["Incoming Submission"]
    B["Tree-sitter AST Parser<br/>(C++, Python, Java, C)"]
    C["Feature Extraction<br/>(22 Base AST + 10 Engineered Ratios)"]
    D["Rust-Compiled XGBoost Inference<br/>(Zero Python Runtime Dependency)"]

    A --> B --> C --> D

    D --> E["Light Tier<br/>256 MiB / 1 CPU<br/><br/>memory.max = 256 MiB<br/>memory.high = 179.2 MiB (70% Soft Watermark)"]
    D --> F["Heavy Tier<br/>Uncapped<br/><br/>memory = Unlimited<br/>cpus = Unlimited"]

    E --> G["cgroup v2 Reactive Monitor<br/>(2ms tick)"]
    G --> H{"memory.events high / memory.current >= 70%?"}

    H -->|YES| I["Live Promotion<br/><br/>write memory.high=max<br/>write memory.max=max<br/>docker update --memory 0 --memory-swap -1 --cpus 0<br/>→ Uncapped (CPU cap lifted too)"]
    H -->|NO| J["Continue in Light Tier"]

    I --> K["Microsecond CFS cpu.stat Accounting"]
    J --> K
    F --> K

    K --> L["Interactive Comparison UI"]
```

---

## 04. Real-World Competition Benchmark Suite

The system includes five high-stakes competition problems modeled after **Codeforces**, **ICPC**, **LeetCode Hard**, and **AtCoder DP Contest**:

1. **Range Prefix Sums & Cumulative Balance** (`Prefix Sums`, Light)
   - $O(N + Q)$ Time · $O(N)$ Space. Evaluates Light-tier performance with zero cgroup watermark events.
2. **0-1 Knapsack Large State Space (2D Grid DP)** (`Dynamic Programming`, Memory-Heavy)
   - $O(N \times W)$ Time · $O(N \times W)$ Space (~200 MiB RSS). Intentionally breaches the 70% (~179.2 MiB) watermark to verify **live reactive container promotion**.
3. **All-Pairs Shortest Path (Floyd-Warshall Algorithm)** (`Graph`, CPU-Bound)
   - $O(V^3)$ Time · $O(V^2)$ Space ($V=100, 120$). Evaluates Predictive AST detection of triply nested loops (`max_loop_depth = 3`).
4. **Game Tree Search (Binary Branching Recursion)** (`Game Theory`, Recursive)
   - $O(2^N)$ Time · $O(N)$ Stack Depth ($N=30, 32$). Evaluates Tree-sitter detection of branching recursion (`is_recursive`, `recursive_call_count = 2`).
5. **Top-K Streaming Frequencies (Hash Map + Priority Queue)** (`Streaming / Heaps`, Collections)
   - $O(N \log K)$ Time · $O(N)$ Space ($N=30\text{k}, 50\text{k}$). Evaluates heavy STL container detection and demonstrates stable CFS CPU accounting.

---

## 05. Key Innovations & Measurements

1. **Microsecond CFS Kernel CPU Timing (`cpu.stat`)**:
   - Rather than measuring host wall-time around `docker exec` (which adds 150–200 ms of container startup noise), the judge reads `/sys/fs/cgroup/.../cpu.stat` deltas directly from the Linux kernel scheduler.
   - Reduces execution measurement variance from $\pm 200\%$ down to $\le \pm 4\%$.
2. **Soft Watermark Live Migration (`memory.high`)**:
   - Uses `memory.high = 179.2 MiB` (70% of `memory.max`) to detect pressure *before* reaching the 256 MiB hard limit (`memory.max`), preventing kernel OOM-killer panics while avoiding premature tier migration.
   - Executes live container expansion by writing `memory.high=max` and `memory.max=max` directly to the container's host cgroup v2 directory, then issuing `docker update --memory 0 --memory-swap -1 --cpus 0` to keep the Docker daemon's view in sync, in $< 15\text{ ms}$ without dropping running processes.
3. **Unified Allocated vs. Used Memory Tracking**:
   - Explicitly records both the **Peak Memory Used** (actual RSS footprint) and **Memory Allocated** (assigned tier ceiling), enabling direct quantification of infrastructure savings.

---

## 06. Experimental Results

*All figures below come from the live judge on the bare-metal calibration host (Fedora 44 KDE, 15 GiB usable RAM, cgroup v2; Low tier 256 MiB; watermark 70% = 179.2 MiB), driven by the single harness [`benchmarks/raas_benchmark.py`](benchmarks/raas_benchmark.py).*

### 6.1 Live corpus — 100 submissions × 4 strategies

**400/400 `AC`, 0 transport failures, 394 s wall clock.** A pure CodeContests corpus contains no memory-heavy programs, so **0 promotions** fired across the whole suite.

| Metric | Value |
|---|---|
| Runs started in the High tier | 246 |
| Runs started in the Low tier | 154 |
| Peak memory used — min / median / mean / max | 6.3 MB / 10.6 MB / 14.4 MB / 53.0 MB |
| Runs under 25 MB | 383 / 400 (95.8%) |
| Allocated per run | 256 MB (Low start) or 2048 MB (High start) |

### 6.2 Verdict-path probes

All five verdict paths are verified against ground truth — **AC, WA, RE, TLE, MLE (5/5)**. The TLE probe is killed by the 10 s per-case guard (~10.03 s CPU / ~10.26 s wall). The MLE probe starts in the Low tier and peaks at ~255.5–256.0 MB before the kernel OOM-kills it.

### 6.3 Synthetic suite — live reactive promotion

On the synthetic heavy knapsack the same program behaves differently by language, now that the JVM heap is sized from the tier:

| Language | Promotion | Peak | Verdict |
|---|:---:|---:|:---:|
| C, C++, Python | promoted | 171–203 MB | **AC** |
| Java (Reactive / Hybrid) | promoted `true` | 202.6 MB | **RE** |
| Java (Predictive) | promoted `false` | 202.5 MB | **RE** |

> **The JVM is not rescued by promotion.** `-Xmx` is fixed at launch (75% of the Low tier = 192 MiB), so once a Java submission is started in the Low tier it cannot grow past its launch-time heap even after the container is promoted. Reactive promotion therefore ends a genuinely oversized Java submission in `RE` instead of an `MLE` race. For JVM languages, correct **routing** (Predictive) is the mechanism that matters — the watermark alone is not enough.

### 6.4 Macro contest simulation

*Seeded (`BENCH_SEED=42`), N = 10,000 submissions drawn from the 43 real problems, 256 MiB tier.*

| Strategy | Slots | Allocated GB | Used GB | Waste | CPU core-h | Avg queue wait | P95 turnaround |
|---|:---:|---:|---:|---:|---:|---:|---:|
| Baseline | 7 | 20000.0 | 119.71 | 99.4% | 5.622 | 2.36 ms | 1448.52 ms |
| Predictive | 56 | 14860.25 | 116.71 | 99.21% | 4.827 | 0.0 ms | 1434.47 ms |
| Reactive | 56 | 2500.0 | 116.79 | 95.33% | 2.876 | 0.0 ms | 1471.9 ms |
| Hybrid | 56 | 14860.25 | 118.93 | 99.2% | 4.831 | 0.0 ms | 1418.68 ms |

Memory saved against Baseline: **Predictive 5139.75 GB (25.7%)**, **Reactive 17500.0 GB (87.5%)**, **Hybrid 5139.75 GB (25.7%)**. Live promotions in the simulation: **0** for every strategy.

### 6.5 Burst stress

*N = 500 submissions in a 30 s window on the 15 GiB host. Baseline safe = `floor(14336 / 2048)` = 7 slots; adaptive = `floor(14336 / 256)` = 56 slots.*

| Scenario | Slots | Avg queue wait | P95 turnaround | Drain | Host RAM util |
|---|:---:|---:|---:|---:|---:|
| Baseline (safe) | 7 | 20966.9 ms | 41794.6 ms | 73.8 s | 93.3% |
| Baseline (2x overcommit) | 14 | 2996.0 ms | 7401.6 ms | 37.7 s | 186.7% |
| Predictive (adaptive) | 56 | 0.0 ms | 1421.8 ms | 31.3 s | 93.3% |
| Reactive (adaptive) | 56 | 0.0 ms | 1482.8 ms | 31.2 s | 93.3% |
| Hybrid (adaptive) | 56 | 0.0 ms | 1394.5 ms | 31.3 s | 93.3% |

### 6.6 Cloud provisioning projection

- Per-pod memory reservation: **2048 MiB → 256 MiB (8.0x)**; per-pod CPU: **2.0 → 1.0 vCPU (2.0x)**.
- Packing density on an AWS `c6i.4xlarge` (32 GB): **14 → 112 concurrent pods (8.0x)**.
- 500-submission burst fleet: **36 VMs → 5 VMs (86.1% fewer)**.
- Cluster cost at USD 0.68/hr per VM: **USD 24.48/hr → USD 3.40/hr (USD 21.08/hr saved, 86.1%)**.
- Total reserved RAM over 10,000 submissions under Reactive: **20000.0 GB → 2500.0 GB (17500.0 GB reclaimed, 87.5%)**.

### 6.7 Per-language profile

*256 MiB tier, Baseline → Reactive.*

| Language | n | Share | Avg CPU | Avg container wall | P95 turnaround | Waste | Saved |
|---|---:|---:|---:|---:|---:|---:|---:|
| C++ | 6743 | 67.4% | 115.7 → 116.4 ms | 1219.1 → 1216.8 ms | 1514.1 → 1526.8 ms | 99.47% → 95.87% | 87.5% |
| Java | 1008 | 10.1% | 290.2 → 292.1 ms | 876.8 → 1127.9 ms | 1003.8 → 1332.8 ms | 98.82% → 90.99% | 87.5% |
| Python | 2249 | 22.5% | 152.9 → 153.0 ms | 451.4 → 449.3 ms | 663.2 → 659.5 ms | 99.46% → 95.65% | 87.5% |

Raw per-cell measurements are committed under [`benchmarks/results/`](benchmarks/results/) as `real_dataset_*_tier256.csv`.

Full per-language matrix, promotion traces, and threats to validity: [`docs/EXPERIMENTAL_RESULTS.md`](docs/EXPERIMENTAL_RESULTS.md).

---

## 07. Tech Stack

- **Judge Server**: Rust (Tokio, Axum, cgroups v2, Linux namespaces).
- **AST Parsing**: Tree-sitter Rust bindings (C, C++, Java, Python).
- **ML Inference**: XGBoost transpiled to pure Rust via `m2cgen` (zero Python dependency at runtime).
- **Frontend Visualizer**: React 19, TypeScript, Vite, Tailwind CSS, Recharts.
- **Training Data**: IBM Project CodeNet (13.9M submissions) *or* DeepMind CodeContests (streamed directly from HuggingFace — no external drive required).

---

## 08. Documentation Index

Detailed architectural and technical documentation is available in the [`docs/`](docs/) directory:

- [**System Architecture** (`docs/ARCHITECTURE.md`)](docs/ARCHITECTURE.md): Deep dive into the AST pipeline, cgroup controllers, and dual watermark design.
- [**Scheduling Strategies** (`docs/SCHEDULING_STRATEGIES.md`)](docs/SCHEDULING_STRATEGIES.md): Formal breakdown of Baseline, Predictive, Reactive, and Hybrid policies.
- [**Benchmark Suite** (`docs/BENCHMARK_SUITE.md`)](docs/BENCHMARK_SUITE.md): Mathematical formulations, complexity, and test cases for all 5 competition problems.
- [**Experimental Results** (`docs/EXPERIMENTAL_RESULTS.md`)](docs/EXPERIMENTAL_RESULTS.md): Empirical data, stability measurements, and memory savings analysis.
- [**Setup & Developer Guide** (`docs/SETUP_GUIDE.md`)](docs/SETUP_GUIDE.md): Complete setup instructions for the judge server, the frontend, the model pipeline, and the benchmark harness.
- [**Model Training Pipeline** (`model-training/README.md`)](model-training/README.md): Dataset extraction (CodeNet **or** CodeContests) → Rust AST feature extraction → XGBoost training → `m2cgen` transpilation into the judge binary.
- [**Feature Extraction Pipeline** (`feature-extraction-pipeline/README.md`)](feature-extraction-pipeline/README.md): The 22 core AST features emitted by the Rust Tree-sitter extractor, plus the 10 engineered ratios added during training (32 features per language; 36 unified).

---

## 09. Quick Start

The predictive models are **already compiled into the judge** ([`server/src/generated/`](server/src/generated/)), so steps 1–3 are the only ones required to *run* the system. Step 0 is only needed if you want to retrain on a different dataset (e.g. CodeContests).

### 0. (Optional) Retrain the models on CodeContests
```bash
cd model-training
python3 -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt

python3 extract_codecontests.py --output-dir ./codecontests_subset \
    --manifest ./sample_manifest_codecontests.csv --per-stratum 5000

(cd ../feature-extraction-pipeline && cargo build --release --bin OJ-feature-extraction-spike)
../feature-extraction-pipeline/target/release/OJ-feature-extraction-spike \
    ./codecontests_subset ./features_codecontests.csv

python3 train_advanced_xgboost.py --features-csv ./features_codecontests.csv \
    --manifest-csv ./sample_manifest_codecontests.csv --output-dir ./artifacts

./regenerate_models.sh        # artifacts/*.joblib -> server/src/generated/*.rs
```
Then **sync the new decision thresholds** from `artifacts/model_comparison.csv` into [`server/src/predict.rs`](server/src/predict.rs:22) before building the server. Full details: [`model-training/README.md`](model-training/README.md).

> ⚠️ `extract_codecontests.py` runs `rm -rf` on `--output-dir` and `--manifest` before writing — do not point it at data you need.

### 1. Build Sandbox Images
```bash
docker build -t python-judge-runtime server/runtimes/python
docker build -t cpp-judge-runtime    server/runtimes/cpp
docker build -t java-judge-runtime   server/runtimes/java
```

### 2. Run Judge Server (Must run with `sudo` for Live Promotion)
```bash
cd server
cargo build

# Run with sudo so the server has permissions to write cgroup v2 soft watermarks:
sudo ./target/debug/server
```
> **Note on Live Promotion**: Running with `sudo` is mandatory for the **Reactive** and **Hybrid** strategies to write to `/sys/fs/cgroup/.../memory.high`. Without `sudo`, the kernel returns `Permission denied (os error 13)` and submissions cannot be promoted mid-execution.

### 3. Start Frontend UI
```bash
cd frontend
npm install
npm run dev
```
Navigate to `http://localhost:5173` to launch the multi-strategy visualizer.

### 4. Benchmark Against the Real Dataset (optional)
With the server still running, in a third terminal:
```bash
# The harness defaults to a LAN IP (http://192.168.0.111:3000) — override it:
JUDGE_URL=http://localhost:3000 python3 benchmarks/raas_benchmark.py all
```
`benchmarks/raas_benchmark.py` is the single entry point (`preflight`, `probe`, `fetch`, `run`, `simulate`, `all`, `status`). It streams real problems from `deepmind/code_contests`, validates each solution, caches the corpus under `benchmarks/dataset/`, and writes `benchmarks/results/real_dataset_*_tier256.csv`. Useful knobs: `LIGHT_TIER_MB`, `BENCH_SEED`, `--count`, `--with-synthetic`, `--langs-per-problem`, `--max-candidates`, `--validate-cases`. See [`docs/SETUP_GUIDE.md`](docs/SETUP_GUIDE.md) §6.

---

## 10. Team

| Name | Role | Responsibilities |
|---|---|---|
| **Hemanthkumar K** | Systems & Infrastructure Lead | Server, Linux cgroup v2 Isolation Manager, CFS Timing, Backend API |
| **Bharath Aashish R** | Frontend & UI/UX Lead | React Visualizer, Recharts Strategy Graphs, Benchmark Code Editor |
| **Dhanush M** | Machine Learning Lead | Dataset Preprocessing, XGBoost Model Training, m2cgen Transpilation |
| **Iniyaa P** | Static Analysis Lead | Tree-sitter Integration, AST Feature Extraction Logic |
