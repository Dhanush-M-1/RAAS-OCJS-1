# RAAS-OJS Model Training Pipeline

This directory contains the dataset extraction, GPU-accelerated XGBoost training, threshold optimization, and Treelite export pipeline for the predictive resource-tier classification layer in **RAAS-OJS**.

---

## 1. Pipeline Overview

```text
[Dataset 1: CodeNet (measured-memory labels)]  OR  [Dataset 2: CodeContests (heuristic labels)]
       │                                                  │
       ▼ [extract_codenet.py]                             ▼ [extract_codecontests.py]
Sampled Dataset Directory (subset/) + Manifest CSV (sample_manifest.csv)
       │
       ▼ [feature-extraction-pipeline] (Rust Tree-sitter Extractor)
features.csv (26 AST features + metadata; +10 engineered in training = 36/40-feature vectors)
       │
       ▼ [train_advanced_xgboost.py] (GroupKFold CV + problem-grouped holdout)
artifacts/  →  model_*.joblib  ·  model_*.json  ·  treelite_*.checkpoint
       │
       ▼ [regenerate_models.sh] (m2cgen — compile XGBoost → Rust)
server/src/generated/*.rs  ← compiled into the judge binary
```

**The whole chain feeds the judge:** the server never loads a model file at runtime —
`regenerate_models.sh` compiles the trained `.joblib` models into Rust source that
`cargo build` bakes straight into the `server` binary.

`evaluate_routing.py` scores the failure mode this whole pipeline exists to minimise — a
genuinely-Heavy program routed to the Low tier ("misroute") — through the deployed
per-language routing (`THRESHOLD_*` in `server/src/predict.rs`).

---

## 2. Prerequisites & Environment Setup

1. **Python $\ge$ 3.10** with `uv` or `pip`:
   ```bash
   cd model-training
   uv venv --python 3.11 .venv
   source .venv/bin/activate
   uv pip install -r requirements.txt
   ```
2. **NVIDIA GPU with CUDA.** `device: 'cuda'` is hardcoded in `train_advanced_xgboost.py`. Note the desktop is effectively CPU-only: xgboost 3.2.0 cannot use its driver 470 GPU, so `device='cuda'` silently falls back to CPU (a full run takes ~25 min). The laptop (RTX 3050 6GB, CUDA 12.9, xgboost `USE_CUDA = True`) runs the same job in ~5 min with **equivalent quality but not bit-identical results** — GPU and CPU runs differ in float-reduction order (unified AUC 0.9502 vs 0.9503).
3. **Compiled Rust Feature Extractor**:
   ```bash
   cd ../feature-extraction-pipeline
   cargo build --release --bin OJ-feature-extraction-spike
   cd ../model-training
   ```

---

## 3. Step-by-Step Execution Guides

### Option A: Training on CodeNet (measured-memory labels)
CodeNet labels are **measured memory**, not a code-length heuristic: `extract_codenet.py` assigns
`Light` below 25 MiB and `Heavy` at 100 MiB or above, and **drops** the ambiguous 25–100 MiB band
rather than guessing it.

```bash
# Step 1: Extract balanced submissions from the CodeNet parquet corpus.
#         Result: 164,686 submissions over 2,520 unique problems
#         (Light 100,000 / Heavy 64,686), ~737 MB on disk.
python3 extract_codenet.py \
    --root "./codenet" \
    --out "./codenet_subset" \
    --manifest-only  # write the manifest without copying sources, if desired

# Step 2: Extract 26 AST features in Rust
../feature-extraction-pipeline/target/release/OJ-feature-extraction-spike \
    ./codenet_subset \
    ./features_codenet.csv

# Step 3: Train XGBoost models & export Treelite checkpoints
python3 train_advanced_xgboost.py \
    --features-csv "./features_codenet.csv" \
    --manifest-csv "./codenet_manifest.csv" \
    --output-dir "./artifacts_codenet_v2"

# Step 4: Compile the trained models into Rust for the judge (m2cgen).
#         The second argument selects the artifacts dir; it defaults to ./artifacts.
./regenerate_models.sh ./.venv/bin/python ./artifacts_codenet_v2

# Step 5: Score the routing failure mode against the deployed thresholds.
python3 evaluate_routing.py \
    --features-csv "./features_codenet.csv" \
    --manifest-csv "./codenet_manifest.csv" \
    --artifacts-dir "./artifacts_codenet_v2"

# Step 6: Sync decision thresholds into the judge (only if you change them), then rebuild.
#   Copy the fixed THRESHOLD_* constants from ../server/src/predict.rs — the new models
#   ship with the existing thresholds (see §4).
cd ../server && cargo build && cd ../model-training
```

`extract_codenet.py` accepts `--per-class`, `--seed`, `--include-mle`, `--manifest-only`, and
`--scan`; the label cut-offs are `--light-max-mib 25.0` and `--heavy-min-mib 100.0`.

> **Working directory note:** Steps 1–3 run from `model-training/`. Step 4 also runs from
> `model-training/` — it writes `server/src/generated/*.rs` automatically.

---

### Option B: Training on DeepMind CodeContests (legacy era)
Streams directly from HuggingFace (`deepmind/code_contests` - Codeforces, CodeChef, HackerEarth) with no external drive required. This is the **old** data era: 30,000 files (5,000 Light / 5,000 Heavy per language, C++/Java/Python), but drawn from only **148 unique problems** (~200 solutions each) and labelled by a **length heuristic** (`extract_codecontests.py`: Light when `difficulty <= 3 or len(code) < 650`, Heavy when `difficulty >= 5 or len(code) >= 1200`, remainder balanced to a 5,000/5,000 quota). No program in this corpus exceeds 50 MiB measured, so its "Heavy" class is not what the Low tier's memory limit actually tests.

> ⚠️ **Destructive step:** `extract_codecontests.py` runs `rm -rf` on both
> `--output-dir` and `--manifest` before it writes. Do not point `--output-dir`
> at a directory you intend to keep, and note it also requires network access to
> HuggingFace plus the `datasets` package (installed via `requirements.txt`).

```bash
# Step 1: Stream and extract balanced submissions (5,000 per stratum - empirical sweet spot)
python3 extract_codecontests.py \
    --output-dir "./codecontests_subset" \
    --manifest "./sample_manifest_codecontests.csv" \
    --per-stratum 5000

# Step 2: Extract 26 AST features in Rust
../feature-extraction-pipeline/target/release/OJ-feature-extraction-spike \
    ./codecontests_subset \
    ./features_codecontests.csv

# Step 3: Train XGBoost models & export Treelite checkpoints
python3 train_advanced_xgboost.py \
    --features-csv "./features_codecontests.csv" \
    --manifest-csv "./sample_manifest_codecontests.csv" \
    --output-dir "./artifacts"

# Step 4: Compile the trained models into Rust for the judge (m2cgen)
./regenerate_models.sh

# Step 5: Sync the new decision thresholds into the judge, then rebuild.
cd ../server && cargo build && cd ../model-training
```

> **Working directory note:** Steps 1–3 run from `model-training/`. Steps 4–5 run
> `regenerate_models.sh` from `model-training/` too — it writes
> `server/src/generated/*.rs` automatically. Both Option A and Option B write
> into `server/src/generated/`, so retraining on a different dataset overwrites
> the models the judge uses. Pass the artifacts directory as `regenerate_models.sh`'s
> second argument (`$2`, default `./artifacts`) to ship a different trained model set
> without moving files around.

---

### Supporting tools

| Tool | Purpose |
|---|---|
| `extract_codenet.py` | CodeNet parquet → measured-memory labels; `--per-class`, `--seed`, `--include-mle`, `--manifest-only`, `--scan`. |
| `evaluate_routing.py` | Misroute rate + AUC through the deployed per-language routing; selects columns via each model's own `feature_names_in_`, so models trained on different feature sets stay comparable. |
| `measure_memory.py` | Peak-RSS harness (`os.wait4`) for measuring memory directly. |
| `build_manifest.py` | Rebuilds a manifest from the on-disk source tree. |

---

## 4. Benchmark Performance on Unseen Problems

### Authoritative: the v2 models and the thresholds used by the judge

`server/src/predict.rs` compiles against the models exported into
`server/src/generated/`. The decision thresholds in that file are the
`THRESHOLD_*` constants currently compiled into the judge binary
(`THRESHOLD_UNIFIED` 0.319, `THRESHOLD_CPP` 0.346, `THRESHOLD_JAVA` 0.257,
`THRESHOLD_PYTHON` 0.200). These are the numbers the trainer prints for the
CodeNet (measured-label) run:

| Model | CV Acc | Test Acc | Test F1 | Test AUC | CV threshold |
|---|---|---|---|---|---|
| Unified Multi-Language | 89.56% | 87.38% | 88.48% | 0.9503 | 0.409 |
| Specialized C++ | 96.07% | 94.39% | 96.26% | 0.9904 | 0.645 |
| Specialized Java | 86.73% | 84.19% | 83.30% | 0.9172 | 0.452 |
| Specialized Python | 85.02% | 83.71% | 86.51% | 0.9221 | 0.441 |
| Specialized C | 97.76% | 96.64% | 41.12% | 0.9387 | 0.200 |

The split is problem-grouped and identical across runs: **125,159 train
submissions / 1,930 problems** and **28,687 test submissions / 483 problems**
(80/20 `GroupShuffleSplit`, seed 42). 10,840 parse-error rows (6.6%) are filtered
before training — CodeNet is messier than CodeContests.

A submission is classified Heavy when $P(\text{Heavy}) \ge \tau$.

> **Note on C**: `train_advanced_xgboost.py` trains and exports a C-specialised
> model, but `regenerate_models.sh` does not transpile it into
> `server/src/generated/` and `server/src/predict.rs` routes `Language::C` through
> `model_unified`, so **that artifact is unused — no C-specific classifier is in
> service**. This is deliberate: C is data-limited (only **428 Heavy examples** in the
> 12.7M-row CodeNet scan; median C memory 0.6 MiB), which is why its Test F1 is
> 41.12% despite the high accuracy/AUC.

### Routing — the headline result

A **misroute** is a genuinely-Heavy program sent to the Low tier. It is the metric
that matters, because overall accuracy is dominated by the Light majority. All rows
use the same decision rule and the same problem-disjoint test split — only the model
changes:

| model set | features | C | C++ | Java | Python | all | routed High |
|---|---|---|---|---|---|---|---|
| original shipped | 32 | 17.9% | 9.8% | 15.5% | 33.5% | **23.7%** | 52.6% |
| CodeNet labels only | 32 | 46.4% | 4.7% | 8.1% | 6.3% | **6.4%** | 59.4% |
| + allocation features | 36 | 25.0% | 4.8% | 7.5% | 6.3% | **6.3%** | 59.1% |

Attribution: the **label fix did almost all of the work**. The allocation features
are near-neutral overall (6.4% → 6.3%) but they cut C's misroute from 46.4% to 25.0%,
because C routes through the unified model, which received them.

### Thresholds — accuracy-optimal is the wrong objective

The trainer selects thresholds to maximise accuracy, which the Light majority
dominates. Applied to the new models, that choice makes routing *less* safe:

| threshold set | all misroute | routed High |
|---|---|---|
| deployed (tuned for the old models) | 6.3% | 59.1% |
| each model's own CV-optimal | 13.4% | 51.0% |
| misroute-minimising | 2.2% | 70.4% |

The misroute-minimising row is **degenerate** — it pins every threshold to the 0.05
grid floor, i.e. "route almost everything High" — which shows misroute alone is not a
usable objective; it has to be balanced against over-provisioning. The deployed
thresholds are the safer operating point, so the new models ship with the **existing**
thresholds.

### Is a misroute actually a failure?

The Heavy label means ">= 100 MiB", but the Low tier's hard limit is **256 MiB**. A
program measured between those two numbers is labelled Heavy; routing it to Low is
counted as a misroute, yet it completes inside Low anyway. Of the 945 misrouted
programs in the test split (memory 100.0–955.9 MiB, median 142.4), **834 (88.3%)
would have fit inside the 256 MiB Low tier**; only **111 (11.7%) would have exceeded
it**. So the raw 6.3% decomposes into ~**0.7% genuine over-limit failures** and ~**5.5%
boundary artefacts**.

> **Caveat:** CodeNet's `memory` was measured on IBM/Aizu hardware, not this project's
> cgroup judge, so absolute MiB does not map one-to-one. The honest range is
> **0.7%–6.3% real failures**; 0.7% is the optimistic end.

Two mechanisms reduce the real-world impact further: the promotion watermark is
`HIGH_WATERMARK_PCT = 70` of the Low limit = **179.2 MiB** (`server/src/docker.rs`), so
most misroutes between 150 and 256 MiB are promoted before they OOM. **Java cannot be
rescued this way** — `-Xmx` is fixed at JVM launch, so for Java a misroute is fatal
regardless of promotion, and its figure should not be discounted the way the aggregate
can be.

### Historical runs (superseded)

Two data eras exist, and the tables below are **not** the models currently compiled
into the judge (those are in the authoritative table above):

| Era | Source | Size | Unique problems | Labels |
|---|---|---|---|---|
| CodeContests | `deepmind/code_contests` via `extract_codecontests.py` | 30,000 files (5,000/5,000 per language) | 148 | length heuristic |
| CodeNet | `iNeil77/CodeNet` parquet via `extract_codenet.py` | 164,686 submissions (Light 100,000 / Heavy 64,686) | 2,520 | measured memory |

The label change was necessary: over the full 12.7M-row CodeNet scan, source length
explains only **7–18% of the variance** of measured memory (Python r=0.4278,
C++ r=0.3648, C r=0.3107, Java r=0.2615), so a length threshold is a weak proxy for the
thing being predicted. The 2,520 unique problems (vs CodeContests' 148) also let the
test split measure generalisation to unseen problems.

![Model Comparison Chart](model_comparison.png)

---

## 5. Exported Model Artifacts

The output directory (`./artifacts/` by default, or whatever is passed to the trainer) will contain:
- **`model_*.joblib`**: Pickled XGBoost models — **this is what `regenerate_models.sh` reads** to produce the Rust code for the judge.
- **`model_*.json`**: Native XGBoost model representations.
- **`treelite_*.checkpoint`**: Standalone compiled decision trees for pure C/Rust runtime inference, with no Python interpreter needed in the judging engine hot path (sub-microsecond latency UNVERIFIED - needs measurement).
- **`model_comparison.csv` / `.png`**: Evaluation metrics and cross-language performance visualizations.

Step 4 (`regenerate_models.sh`) additionally writes **`server/src/generated/*.rs`** —
the XGBoost models compiled to Rust via `m2cgen` that `cargo build` bakes into the judge.

> **`regenerate_models.sh` needs `m2cgen` + `joblib`** installed in a Python venv at
> `model-training/.venv` (see §2). It also handles the XGBoost 3.x `base_score=None`
> quirk that otherwise breaks `m2cgen`. It takes an optional artifacts directory as its
> second argument (`$2`, default `./artifacts`).

> **Feature counts are hardcoded in FOUR unconnected places:** `server/src/predict.rs`
> (36 per-language / 40 unified), `train_advanced_xgboost.py`, `regenerate_models.sh`,
> and the generated Rust under `server/src/generated/`. They cannot see each other.
> `regenerate_models.sh` now verifies each trained model's own `num_features()` and
> **refuses on mismatch** — pointed at the old 32-feature models it exits with an error
> instead of writing. Changing the feature set therefore still means updating the
> extractor, the trainer, this script and `predict.rs` together.
