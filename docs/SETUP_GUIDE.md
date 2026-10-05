# System Setup & Developer Guide

This guide provides step-by-step instructions for building, configuring, and running the entire RAAS-OCJS system locally.

---

## 1. Prerequisites & System Requirements

- **Operating System**: Linux with **cgroups v2** enabled (Ubuntu 22.04+, Debian 12+, Arch Linux, Fedora).
- **Rust Toolchain**: `rustc` and `cargo` (1.75+ or edition 2024).
- **Node.js**: Node 18+ and `npm`.
- **Docker Engine**: Native Linux Docker daemon (`/var/run/docker.sock`).
- **Python (Optional for model training only)**: Python 3.10+ with `virtualenv`.

---

## 2. Docker Runtime Images

The judge runs each submission inside a language-specific Docker sandbox. Build the runtime images from the repository root:

```bash
# From repository root (RAAS-OCJS/)
docker build -t python-judge-runtime server/runtimes/python
docker build -t cpp-judge-runtime    server/runtimes/cpp
docker build -t java-judge-runtime   server/runtimes/java
```

> **Note**: `C` and `C++` submissions both execute within `cpp-judge-runtime`.

---

## 3. Building & Running the Judge Server

### 3.1 Build the Server
The predictive XGBoost models are pre-compiled into Rust code (`server/src/generated/`), meaning **no Python environment is required to build or run the judge server**:

```bash
cd server
cargo build
```

### 3.2 Run the Server with Root / Sudo (Required for Live Promotion)

> [!IMPORTANT]
> **Why `sudo` is mandatory for Reactive & Hybrid Promotion:**
> During execution of Reactive or Hybrid submissions, the judge dynamically writes the ~179.2 MiB (70%) watermark to the container's kernel cgroup file:
> `/sys/fs/cgroup/system.slice/docker-<id>.scope/memory.high`
> 
> Under standard Linux systemd cgroup hierarchies, unprivileged processes cannot write to `/sys/fs/cgroup`.
> - **Without `sudo`**: The server starts, but logs `[moderator] failed to arm memory.high for oj_...: Permission denied (os error 13)`. The kernel never increments `memory.events` pressure counters, and **reactive promotion will never trigger**.
> - **With `sudo`**: The server writes `memory.high` cleanly. Memory-heavy jobs (e.g. the Problem 2 0-1 knapsack in C, C++ or Python) cross the watermark and are promoted from Low (256 MiB) to Uncapped at runtime, passing at 171–203 MB peak.
>
> The Low tier is enforced with `--cpus=1 --memory=256m --memory-swap=256m`. Swap is pinned equal to the memory limit deliberately: with `--memory` set and `--memory-swap` omitted, Docker defaults swap to the same value and a container can draw roughly 2x its nominal size from RAM+swap.

```bash
# Build the binary
cd server
cargo build

# Execute with root privileges:
sudo ./target/debug/server
# Or via cargo:
sudo -E cargo run
```

Expected output:
```
Judge is online and listening on :3000
```

> [!WARNING]
> **Do not expose the judge to a public network.** It binds `0.0.0.0:3000` with **no authentication** and executes untrusted submitted code. It must never sit in a public security group; keep it reachable only over the LAN / tailnet, or put it behind an authenticated tunnel.

#### How to verify live promotion is active:
Submit Problem 2 (**0-1 Knapsack Large State Space**) using the Reactive strategy:
1. In the terminal running the server, confirm there are **no** `Permission denied (os error 13)` warnings.
2. In the server output or UI, observe:
   - `tier_started: "low"`
   - `tier_promoted: true`
   - `promotion_time_ms: UNVERIFIED - needs measurement`
   - `allocated_memory_bytes` transitions from `256 MB` to `Uncapped`.

### 3.3 Verifying Docker Context
If you have Docker Desktop installed alongside native Docker, ensure the native daemon is used so the host `/sys/fs/cgroup` tree is accessible:
```bash
docker context use default
```
*(The server automatically enforces `DOCKER_CONTEXT=default` at startup when `/var/run/docker.sock` is detected).*

---

## 4. Running the Interactive Frontend

The frontend is built with **React**, **TypeScript**, **Vite**, **Tailwind CSS**, and **Recharts**:

```bash
cd frontend
npm install
npm run dev
```

Open your browser at:
```
http://localhost:5173
```

- **Problem Switcher**: Choose from the 5 competition problems in the left sidebar.
- **Language Switcher**: Toggle between Python, C++, Java, and C.
- **Run Strategies**: Select a single strategy or click **"Run all four strategies"** to benchmark Baseline, Predictive, Reactive, and Hybrid side-by-side.

---

## 5. Model Training & Regeneration Workflow (Optional)

The predictive models are **already compiled into the judge** (`server/src/generated/`), so this section is only needed if you change the dataset or hyperparameters. Both datasets write into the **same** `./artifacts/` and `server/src/generated/`, so a retrain simply overwrites the models the judge uses.

Full details and benchmark tables: [`model-training/README.md`](../model-training/README.md).

### 5.1 Environment setup

```bash
cd model-training
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt

# Build the Rust Tree-sitter feature extractor once:
cd ../feature-extraction-pipeline
cargo build --release --bin OJ-feature-extraction-spike
cd ../model-training
```

### 5.2 Option A — IBM Project CodeNet

```bash
python3 extract_dataset.py \
    --codenet-root "/path/to/Project_CodeNet" \
    --output-dir "./codenet_subset" \
    --manifest "./sample_manifest.csv" \
    --per-stratum 10000 --workers 64

../feature-extraction-pipeline/target/release/OJ-feature-extraction-spike \
    ./codenet_subset ./features.csv

python3 train_advanced_xgboost.py \
    --features-csv "./features.csv" \
    --manifest-csv "./sample_manifest.csv" \
    --output-dir "./artifacts"
```

### 5.3 Option B — DeepMind CodeContests (streamed from HuggingFace, no external drive)

```bash
python3 extract_codecontests.py \
    --output-dir "./codecontests_subset" \
    --manifest "./sample_manifest_codecontests.csv" \
    --per-stratum 5000

../feature-extraction-pipeline/target/release/OJ-feature-extraction-spike \
    ./codecontests_subset ./features_codecontests.csv

python3 train_advanced_xgboost.py \
    --features-csv "./features_codecontests.csv" \
    --manifest-csv "./sample_manifest_codecontests.csv" \
    --output-dir "./artifacts"
```

> ⚠️ **Destructive:** `extract_codecontests.py` runs `rm -rf` on `--output-dir` and `--manifest` before writing. It also needs network access to HuggingFace and the `datasets` package (installed via `requirements.txt`).

### 5.4 Transpile the models into Rust (m2cgen)

```bash
./regenerate_models.sh        # artifacts/*.joblib -> ../server/src/generated/*.rs
```

### 5.5 Sync the decision thresholds — do not skip this step

`train_advanced_xgboost.py` writes an **Optimal Threshold** per model into `artifacts/model_comparison.csv`. The judge hard-codes those thresholds as constants in [`server/src/predict.rs`](../server/src/predict.rs), so a retrain that skips this step ships **new weights with stale decision boundaries**.

Copy each model's threshold from `artifacts/model_comparison.csv` into the matching `THRESHOLD_*` constant, then rebuild the server:

```bash
cd ../server
cargo build
```

> Only the Python, C++, Java, and unified models are exported. **C submissions are scored by the unified model** (there is no specialised C model), so they use the unified threshold.

---

## 6. Real-Dataset Benchmark Harness (Optional)

No benchmark harness is currently committed. The former single-entry-point harness and its results
were withdrawn because the corpus, cloud target and promotion path changed repeatedly and the
published figures no longer corresponded to any single run. The replacement experimental programme
is specified in [`TEST_PLAN.md`](TEST_PLAN.md), which also defines the harness interface the new
script must expose (see TEST_PLAN E0).

Environment knobs:

| Variable | Default | Meaning |
|---|---|---|
| `JUDGE_URL` | `http://192.168.0.111:3000` | Judge base URL (used for `/health` and `/submit`) |
| `LIGHT_TIER_MB` | `256` | Must match the server's own low-tier size, or all derived slot/cloud figures are invalid |
| `BENCH_SEED` | `42` | Seeds the stochastic simulation scenarios for reproducible figures |

The harness runs four phases in order: empirical per-submission evaluation → macro contest simulation (N = 10,000) → summary export → burst-stress simulation. It exits immediately if `/health` does not return `{"status":"OK"}` — start the server first.

---

## 7. Troubleshooting

### Permission denied (os error 13) on `memory.high`
**Cause**: The judge server process does not have write permissions to the cgroup controller file.  
**Fix**: Run the server with `sudo ./target/debug/server` or configure systemd cgroup delegation for your user slice.

### "Address already in use" (port 3000)
**Cause**: Another judge process or service is running on port 3000.  
**Fix**: Find and terminate the process:
```bash
sudo lsof -i :3000
sudo kill -9 <PID>
```
