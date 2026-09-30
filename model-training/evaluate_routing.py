#!/usr/bin/env python3
"""Score the failure mode the tiering system exists to prevent.

The headline number for RAAS-OCJS is not accuracy. Accuracy is dominated by the
Light class, which is the easy one, so a model can look strong while failing on
exactly the submissions that matter. The number that matters is the *misroute
rate*: of the test programs that genuinely need more than the Low tier's memory,
how many get routed to Low anyway.

This reproduces the trainer's problem-grouped split (same inputs, same merge order,
same seed) and reports, using the same per-language routing the judge performs
(C -> unified, everything else -> its specialized model, per `server/src/predict.rs`):

  misroute  - fraction of genuinely-Heavy test programs predicted Light
  auc       - ROC-AUC over the test set
  thr       - the threshold the trainer's own search would pick

Usage:
    ./.venv/bin/python evaluate_routing.py \
        --features-csv features_codenet_v2.csv \
        --manifest-csv codenet_manifest.csv \
        --artifacts-dir artifacts_codenet_v2

    # compare two runs side by side:
    ./.venv/bin/python evaluate_routing.py ... --compare-to artifacts_codenet
"""

import argparse
import os
import sys

import joblib
import numpy as np
import pandas as pd
from sklearn.metrics import roc_auc_score
from sklearn.model_selection import GroupShuffleSplit

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from train_advanced_xgboost import (  # noqa: E402
    find_optimal_threshold,
    get_feature_cols,
    load_and_engineer_features,
)

# Mirrors the match arms in server/src/predict.rs::predict_tier. C has no
# specialized model of its own and is served by the unified one.
ROUTING = {
    "C": "model_unified_multi-language",
    "C++": "model_specialized_cpp",
    "Java": "model_specialized_java",
    "Python": "model_specialized_python",
}


def build_split(df, seed):
    """Reproduce the trainer's split exactly (main() in train_advanced_xgboost.py)."""
    lang_dummies = pd.get_dummies(df["language"], prefix="lang", dtype=int)
    for expected in ["C", "C++", "Java", "Python"]:
        col = f"lang_{expected}"
        if col not in lang_dummies.columns:
            lang_dummies[col] = 0
    df_full = pd.concat([df, lang_dummies], axis=1)

    groups = df_full["problem_id"]
    gss = GroupShuffleSplit(n_splits=1, test_size=0.2, random_state=seed)
    train_idx, test_idx = next(gss.split(df_full, df_full["target"], groups))
    return df_full.iloc[train_idx].copy(), df_full.iloc[test_idx].copy()


def score_run(df_train, df_test, artifacts_dir, label):
    """Predict every test row with the model the judge would actually use."""
    unified_cols = get_feature_cols(include_language=True)
    per_lang_cols = get_feature_cols(include_language=False)

    models = {}
    for name in set(ROUTING.values()):
        path = os.path.join(artifacts_dir, f"{name}.joblib")
        if not os.path.exists(path):
            raise SystemExit(f"missing artifact: {path}")
        models[name] = joblib.load(path)

    # Score each row with the model the judge would actually use. Column selection
    # comes from the model's own recorded feature names when available, so an older
    # artifact (trained on a different feature set) can still be scored on the same
    # test split without silently feeding it misaligned columns.
    prob = np.full(len(df_test), np.nan)
    for lang, model_name in ROUTING.items():
        mask = (df_test["language"] == lang).to_numpy()
        if not mask.any():
            continue
        model = models[model_name]
        recorded = getattr(model, "feature_names_in_", None)
        cols = list(recorded) if recorded is not None else (
            unified_cols if lang == "C" else per_lang_cols
        )
        missing = [c for c in cols if c not in df_test.columns]
        if missing:
            raise SystemExit(
                f"{model_name} expects columns absent from the features CSV: {missing[:5]}"
            )
        prob[mask] = model.predict_proba(df_test.loc[mask, cols])[:, 1]

    if np.isnan(prob).any():
        missing = sorted(df_test.loc[np.isnan(prob), "language"].unique())
        raise SystemExit(f"unrouted languages: {missing}")

    y = df_test["target"].to_numpy()
    auc = roc_auc_score(y, prob)
    thr = find_optimal_threshold(y, prob)
    pred = (prob >= thr).astype(int)

    # A misroute is a genuinely-Heavy program sent to the Low tier.
    heavy = y == 1
    misroute = float((pred[heavy] == 0).mean()) if heavy.any() else float("nan")

    rows = [{
        "run": label,
        "language": "ALL",
        "n_test": len(y),
        "n_heavy": int(heavy.sum()),
        "misroute": misroute,
        "auc": auc,
        "thr": thr,
    }]
    for lang in sorted(ROUTING):
        mask = (df_test["language"] == lang).to_numpy()
        h = mask & heavy
        if not mask.any():
            continue
        lang_auc = roc_auc_score(y[mask], prob[mask]) if len(np.unique(y[mask])) > 1 else float("nan")
        rows.append({
            "run": label,
            "language": lang,
            "n_test": int(mask.sum()),
            "n_heavy": int(h.sum()),
            "misroute": float((pred[h] == 0).mean()) if h.any() else float("nan"),
            "auc": lang_auc,
            "thr": find_optimal_threshold(y[mask], prob[mask]),
        })
    return pd.DataFrame(rows)


def main():
    p = argparse.ArgumentParser(description="Misroute-rate evaluation for the RAAS-OCJS router")
    p.add_argument("--features-csv", required=True)
    p.add_argument("--manifest-csv", required=True)
    p.add_argument("--artifacts-dir", required=True, help="dir containing the .joblib models")
    p.add_argument("--compare-to", default=None, help="second artifacts dir to score side by side")
    p.add_argument("--seed", type=int, default=42)
    p.add_argument("--out", default=None, help="write the table to this CSV")
    args = p.parse_args()

    df = load_and_engineer_features(args.features_csv, args.manifest_csv)
    df_train, df_test = build_split(df, args.seed)
    print(
        f"\nSplit: {len(df_train)} train across {df_train['problem_id'].nunique()} problems; "
        f"{len(df_test)} test across {df_test['problem_id'].nunique()} problems"
    )

    frames = [score_run(df_train, df_test, args.artifacts_dir, os.path.basename(args.artifacts_dir.rstrip("/")))]
    if args.compare_to:
        frames.append(
            score_run(df_train, df_test, args.compare_to, os.path.basename(args.compare_to.rstrip("/")))
        )
    table = pd.concat(frames, ignore_index=True)

    show = table.copy()
    show["misroute"] = (show["misroute"] * 100).round(1).astype(str) + "%"
    show["auc"] = show["auc"].round(4)
    show["thr"] = show["thr"].round(3)
    print("\n--- Routing quality (lower misroute is better) ---")
    print(show.to_string(index=False))

    if args.out:
        table.to_csv(args.out, index=False)
        print(f"\nwrote {args.out}")


if __name__ == "__main__":
    main()
