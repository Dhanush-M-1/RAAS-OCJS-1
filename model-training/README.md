# RAAS-OJS Model Training Pipeline

This directory contains the dataset extraction, GPU-accelerated XGBoost training, threshold optimization, and Treelite export pipeline for the predictive resource-tier classification layer in **RAAS-OJS**.

---

## 1. Pipeline Overview

```text
[Dataset 1: IBM Project CodeNet]  OR  [Dataset 2: DeepMind CodeContests]
       │                                     │
       ▼ [extract_dataset.py]                ▼ [extract_codecontests.py]
Sampled Dataset Directory (subset/) + Manifest CSV (sample_manifest.csv)
       │
       ▼ [feature-extraction-pipeline] (Rust Tree-sitter Extractor)
features.csv (22 AST features + metadata; +10 engineered in training = 32/36-feature vectors)
       │
       ▼ [train_advanced_xgboost.py] (5-Fold GroupKFold on Unseen Problems)
artifacts/  →  model_*.joblib  ·  model_*.json  ·  treelite_*.checkpoint
       │
       ▼ [regenerate_models.sh] (m2cgen — compile XGBoost → Rust)
server/src/generated/*.rs  ← compiled into the judge binary
```

**The whole chain feeds the judge:** the server never loads a model file at runtime —
`regenerate_models.sh` compiles the trained `.joblib` models into Rust source that
`cargo build` bakes straight into the `server` binary.

---

## 2. Prerequisites & Environment Setup

1. **Python $\ge$ 3.10** with `uv` or `pip`:
   ```bash
   cd model-training
   uv venv --python 3.11 .venv
   source .venv/bin/activate
   uv pip install -r requirements.txt
   ```
2. **NVIDIA GPU with CUDA — required as shipped.** `device: 'cuda'` is hardcoded in `train_advanced_xgboost.py`, so training requires an NVIDIA GPU. On a machine without one, override the device (e.g. `device='cpu'`) before running.
3. **Compiled Rust Feature Extractor**:
   ```bash
   cd ../feature-extraction-pipeline
   cargo build --release --bin OJ-feature-extraction-spike
   cd ../model-training
   ```

---

## 3. Step-by-Step Execution Guides

### Option A: Training on IBM Project CodeNet (Real Server Execution Logs)
> **Prerequisite:** Set `--codenet-root` to where the IBM Project CodeNet dataset is mounted on your machine.

```bash
# Step 1: Extract stratified submissions across C, C++, Java, and Python
#         (submission count UNVERIFIED - needs measurement)
python3 extract_dataset.py \
    --codenet-root "/path/to/Project_CodeNet" \
    --output-dir "./codenet_subset" \
    --manifest "./sample_manifest.csv" \
    --per-stratum 10000 \
    --workers 64

# Step 2: Extract 22 AST features in Rust (timing UNVERIFIED - needs measurement)
../feature-extraction-pipeline/target/release/OJ-feature-extraction-spike \
    ./codenet_subset \
    ./features.csv

# Step 3: Train XGBoost models & export Treelite checkpoints
python3 train_advanced_xgboost.py \
    --features-csv "./features.csv" \
    --manifest-csv "./sample_manifest.csv" \
    --output-dir "./artifacts"

# Step 4: Compile the trained models into Rust for the judge (m2cgen)
./regenerate_models.sh

# Step 5: Sync the new decision thresholds into the judge, then rebuild.
#   Copy the "Optimal Threshold" values from ./artifacts/model_comparison.csv
#   into the THRESHOLD_* constants in ../server/src/predict.rs.
cd ../server && cargo build && cd ../model-training
```

> **Working directory note:** Steps 1–3 run from `model-training/`. Steps 4–5 run
> `regenerate_models.sh` from `model-training/` too — it writes
> `server/src/generated/*.rs` automatically.

---

### Option B: Training on DeepMind CodeContests (Modern Algorithmic Benchmark)
Streams directly from HuggingFace (`deepmind/code_contests` - Codeforces, CodeChef, HackerEarth) with no external drive required.

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

# Step 2: Extract 22 AST features in Rust (timing UNVERIFIED - needs measurement)
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
#   Copy the "Optimal Threshold" values from ./artifacts/model_comparison.csv
#   into the THRESHOLD_* constants in ../server/src/predict.rs.
cd ../server && cargo build && cd ../model-training
```

> **Working directory note:** Steps 1–3 run from `model-training/`. Steps 4–5 run
> `regenerate_models.sh` from `model-training/` too — it writes
> `server/src/generated/*.rs` automatically. Both Option A and Option B write
> into the **same** `./artifacts/` and `server/src/generated/`, so retraining on
> a different dataset simply overwrites the models the judge uses.

---

## 4. Benchmark Performance on Unseen Problems

### Authoritative: thresholds used by the judge

`server/src/predict.rs` compiles against the models exported into
`server/src/generated/`. The decision thresholds in that file are the
`THRESHOLD_*` constants currently compiled into the judge binary:

| Model | Test Acc | Test F1 | ROC-AUC | Optimal Threshold |
|---|---|---|---|---|
| Unified Multi-Language | 94.45% *(UNVERIFIED - not reproducible; retrain in progress)* | 92.03% | 0.9886 | 0.319 |
| Specialized C++ | 95.45% *(UNVERIFIED - not reproducible; retrain in progress)* | 94.27% | 0.9938 | 0.346 |
| Specialized Java | 82.82% *(UNVERIFIED - not reproducible; retrain in progress)* | 81.37% | 0.9380 | 0.257 |
| Specialized Python | 98.88% *(UNVERIFIED - not reproducible; retrain in progress)* | 97.85% | 0.9961 | 0.200 |

> **⚠️ CONFLICT — both sets of accuracy figures in this section are
> `UNVERIFIED - not reproducible; retrain in progress`.** `artifacts/model_comparison.csv`
> reports 94.45 / 95.45 / 82.82 / 98.88 % while the CodeNet table further down this
> file reports 83.71 / 90.01 / 78.20 / 79.45 %. The `features.csv` and
> `sample_manifest*.csv` intermediates are absent from the working tree and
> gitignored, so **neither figure set can currently be reproduced from a clean
> checkout**. A retrain is in progress. The F1 and ROC-AUC columns above come from
> the same run and are likewise unverifiable; the threshold column is the set of
> constants currently compiled into `server/src/predict.rs`.

A submission is classified Heavy when $P(\text{Heavy}) \ge \tau$.

> **Note on C**: `train_advanced_xgboost.py` trains a C-specialised model, but
> `regenerate_models.sh` does not export it — only Python, C++, Java, and the
> unified model are transpiled into `server/src/generated/`. C submissions are
> therefore scored by the **unified multi-language model** (`server/src/predict.rs`
> routes `Language::C` through `model_unified`).

### Historical runs (superseded)

The tables below record earlier training runs on the two dataset sources. They
are kept for reference but **do not correspond to the models currently compiled
into the judge** — see the authoritative table above. A new run overwrites
`./artifacts/`, so the authoritative numbers are always the most recent.

> All figures in these two tables are
> `UNVERIFIED - not reproducible; retrain in progress`. See the conflict note above.

### IBM Project CodeNet (submission count UNVERIFIED - needs measurement)
| Model Architecture | 5-Fold CV Accuracy | Test Accuracy (Unseen Problems) | F1-Score | Precision | Recall | ROC-AUC | Optimal Threshold |
|---|---|---|---|---|---|---|---|
| **Specialized C++ Model** | $87.32\%$ | **$90.01\%$** *(std: 90.01%)* — UNVERIFIED - not reproducible; retrain in progress | **$90.91\%$** | $88.54\%$ | $93.41\%$ | **$0.9612$** | $0.439$ |
| **Specialized C Model** | $89.27\%$ | **$89.26\%$** *(std: 91.95%)* — UNVERIFIED - not reproducible; retrain in progress | **$68.82\%$** | $55.98\%$ | $89.31\%$ | **$0.9575$** | $0.200$ |
| **Specialized Python Model** | $76.97\%$ | **$79.45\%$** *(std: 79.13%)* — UNVERIFIED - not reproducible; retrain in progress | **$82.01\%$** | $80.33\%$ | $83.76\%$ | **$0.8740$** | $0.456$ |
| **Specialized Java Model** | $79.30\%$ | **$78.20\%$** *(std: 77.52%)* — UNVERIFIED - not reproducible; retrain in progress | **$81.54\%$** | $82.03\%$ | $81.06\%$ | **$0.8537$** | $0.482$ |
| **Unified Multi-Language Model** | $82.19\%$ | **$83.71\%$** *(std: 83.76%)* — UNVERIFIED - not reproducible; retrain in progress | **$84.81\%$** | $80.19\%$ | $89.99\%$ | **$0.9149$** | $0.457$ |

### DeepMind CodeContests (30,000 Submissions - 5,000 Sweet Spot Strata)
| Model Architecture | 5-Fold CV Accuracy | Test Accuracy (Unseen Problems) | F1-Score | Precision | Recall | ROC-AUC | Optimal Threshold |
|---|---|---|---|---|---|---|---|
| **Unified Multi-Language Model** | $86.74\%$ | **$86.78\%$** *(std: 88.01%)* — UNVERIFIED - not reproducible; retrain in progress | **$89.64\%$** | $82.74\%$ | $97.79\%$ | **$0.9606$** | $0.231$ |
| **Specialized Python Model** | $95.76\%$ | **$98.88\%$** *(std: 98.32%)* — UNVERIFIED - not reproducible; retrain in progress | **$99.41\%$** | $98.82\%$ | $100.0\%$ | **$0.9812$** | $0.100$ |
| **Specialized C++ Model** | $84.27\%$ | **$83.94\%$** *(std: 85.92%)* — UNVERIFIED - not reproducible; retrain in progress | **$87.49\%$** | $78.67\%$ | $98.53\%$ | **$0.9474$** | $0.100$ |
| **Specialized Java Model** | $79.43\%$ | **$82.32\%$** *(std: 84.60%)* — UNVERIFIED - not reproducible; retrain in progress | **$83.28\%$** | $73.09\%$ | $96.76\%$ | **$0.9430$** | $0.200$ |

![Model Comparison Chart](model_comparison.png)

---

## 5. Exported Model Artifacts

The output directory `./artifacts/` will contain:
- **`model_*.joblib`**: Pickled XGBoost models — **this is what `regenerate_models.sh` reads** to produce the Rust code for the judge.
- **`model_*.json`**: Native XGBoost model representations.
- **`treelite_*.checkpoint`**: Standalone compiled decision trees for pure C/Rust runtime inference, with no Python interpreter needed in the judging engine hot path (sub-microsecond latency UNVERIFIED - needs measurement).
- **`model_comparison.csv` / `.png`**: Evaluation metrics and cross-language performance visualizations.

Step 4 (`regenerate_models.sh`) additionally writes **`server/src/generated/*.rs`** —
the XGBoost models compiled to Rust via `m2cgen` that `cargo build` bakes into the judge.

> **`regenerate_models.sh` needs `m2cgen` + `joblib`** installed in a Python venv at
> `model-training/.venv` (see §2). It also handles the XGBoost 3.x `base_score=None`
> quirk that otherwise breaks `m2cgen`.

> **Feature counts are hardcoded in two places:** `regenerate_models.sh` (32 features
> for the specialised models, 36 for the unified model) and `server/src/predict.rs`.
> Changing the feature set means updating the extractor, the trainer, this script and
> `predict.rs` together. There is **no automated test** that the judge's feature vector
> matches the extractor's feature order/count — treat this as a known risk when
> changing features.
