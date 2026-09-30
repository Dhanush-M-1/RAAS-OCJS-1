//! Predictive tier selection: source -> AST features -> XGBoost (m2cgen) -> Tier.
//!
//! The model weights are compiled into Rust by m2cgen (see `src/generated/`).
//! Feature order must match `get_feature_cols()` in `model-training/train_advanced_xgboost.py`:
//!   26 base AST features, then 10 engineered features, then (unified only) 4 language one-hots.
//! The counts are also hardcoded in `model-training/regenerate_models.sh`; a
//! mismatch between the four sites produces silently wrong scores rather than an
//! error, which is what `feature_vector_len_is_pinned` below guards against.

use crate::policy::Tier;
use feature_extraction::features::{compute_features, Features};
use feature_extraction::language::Language;

// Isolated modules so each generated `pub fn score` has its own namespace.
#[path = "generated/specialized_python.rs"]
mod model_python;
#[path = "generated/specialized_cpp.rs"]
mod model_cpp;
#[path = "generated/specialized_java.rs"]
mod model_java;
#[path = "generated/unified_multi_language.rs"]
mod model_unified;

/// Per-language optimal decision thresholds from `model-training/artifacts/model_comparison.csv`.
const THRESHOLD_UNIFIED: f64 = 0.319;
const THRESHOLD_CPP: f64 = 0.346;
const THRESHOLD_JAVA: f64 = 0.257;
const THRESHOLD_PYTHON: f64 = 0.200;

/// Map the judge's lowercase language string to the pipeline's `Language`.
fn to_language(language: &str) -> Option<Language> {
    match language {
        "python" => Some(Language::Python),
        "cpp" | "c++" => Some(Language::Cpp),
        "java" => Some(Language::Java),
        "c" => Some(Language::C),
        _ => None,
    }
}

/// Map `bool` to float for the model.
fn b(x: bool) -> f64 {
    if x { 1.0 } else { 0.0 }
}
fn u32(x: u32) -> f64 {
    x as f64
}
fn u64(x: u64) -> f64 {
    x as f64
}

/// Build the 36-feature vector for the specialized (per-language) models.
/// Order: 26 base AST + 10 engineered. Matches `get_feature_cols(include_language=False)`.
fn specialized_features(f: &Features) -> Vec<f64> {
    let max_nodes = f.ast_node_count.max(1) as f64;
    let max_loc = f.source_loc.max(1) as f64;
    let max_subscripts = f.total_subscripts.max(1) as f64;
    let max_fns = f.total_functions.max(1) as f64;

    vec![
        u32(f.nesting_depth),               // 1 nesting_depth
        u32(f.max_loop_depth),              // 2 max_loop_depth
        u32(f.total_loops),                 // 3 total_loops
        u32(f.cyclomatic_complexity),       // 4 cyclomatic_complexity
        b(f.is_recursive),                  // 5 is_recursive
        u32(f.recursive_call_count),        // 6 recursive_call_count
        b(f.large_alloc_flag),              // 7 large_alloc_flag
        u64(f.alloc_size_max),              // 8 alloc_size_max
        u64(f.alloc_size_total),            // 9 alloc_size_total
        u32(f.alloc_sites),                 // 10 alloc_sites
        u32(f.alloc_unknown_sites),         // 11 alloc_unknown_sites
        b(f.has_fast_io),                   // 12 has_fast_io
        b(f.has_heavy_datastructure),       // 13 has_heavy_datastructure
        b(f.has_modulo_arithmetic),         // 14 has_modulo_arithmetic
        b(f.has_bitmask_ops),               // 15 has_bitmask_ops
        b(f.has_graph_adjacency),           // 16 has_graph_adjacency
        u32(f.total_functions),             // 17 total_functions
        u32(f.total_calls),                 // 18 total_calls
        u32(f.total_subscripts),            // 19 total_subscripts
        u32(f.total_2d_subscripts),         // 20 total_2d_subscripts
        u32(f.total_arithmetic_ops),        // 21 total_arithmetic_ops
        u64(f.max_integer_constant),        // 22 max_integer_constant
        u32(f.ast_node_count),              // 23 ast_node_count
        u32(f.ast_depth),                   // 24 ast_depth
        u32(f.source_loc),                  // 25 source_loc
        u32(f.source_chars),                // 26 source_chars
        // 10 engineered
        u32(f.total_loops) / max_nodes,             // loop_density
        u32(f.total_calls) / max_nodes,             // call_density
        u32(f.total_subscripts) / max_nodes,        // subscript_density
        u32(f.cyclomatic_complexity) / max_loc,     // branch_density
        u32(f.total_arithmetic_ops) / max_loc,      // arithmetic_density
        u32(f.total_2d_subscripts) / max_subscripts,// subscript_2d_ratio
        u32(f.recursive_call_count) / max_fns,      // recursion_intensity
        (f.max_integer_constant.max(1) as f64).log10(), // log_max_constant
        (1.0 + f.ast_node_count as f64).ln(),       // log_ast_nodes
        (1.0 + f.source_chars as f64).ln(),         // log_source_chars
    ]
}

/// Build the 40-feature vector for the unified multi-language model.
/// Order: 36 features + 4 language one-hots (lang_C, lang_C++, lang_Java, lang_Python).
fn unified_features(f: &Features, lang: Language) -> Vec<f64> {
    let mut v = specialized_features(f);
    let (c, cpp, java, python) = match lang {
        Language::C => (1.0, 0.0, 0.0, 0.0),
        Language::Cpp => (0.0, 1.0, 0.0, 0.0),
        Language::Java => (0.0, 0.0, 1.0, 0.0),
        Language::Python => (0.0, 0.0, 0.0, 1.0),
    };
    v.extend([c, cpp, java, python]);
    v
}

/// Predict whether a submission is Heavy (true) or Light (false), returning the proposed Tier.
/// Uses the per-language specialized model when available, falling back to the unified model.
pub fn predict_tier(source: &str, language: &str) -> Tier {
    let Some(lang) = to_language(language) else {
        // Unsupported language: default to High to be safe (never underestimate).
        return Tier::High;
    };
    let f = compute_features(source, lang);

    // Heavy probability from the appropriate model + threshold.
    let (prob, threshold) = match lang {
        Language::Python => (model_python::score(specialized_features(&f))[1], THRESHOLD_PYTHON),
        Language::Cpp => (model_cpp::score(specialized_features(&f))[1], THRESHOLD_CPP),
        Language::Java => (model_java::score(specialized_features(&f))[1], THRESHOLD_JAVA),
        Language::C => (model_unified::score(unified_features(&f, lang))[1], THRESHOLD_UNIFIED),
    };

    if prob >= threshold {
        Tier::High
    } else {
        Tier::Low
    }
}

#[cfg(test)]
mod feature_vector_tests {
    use super::*;

    /// The feature count lives in four places that cannot see each other: this file,
    /// `train_advanced_xgboost.py` (`get_feature_cols`), `regenerate_models.sh`, and the
    /// m2cgen output under `src/generated/`. A mismatch is not reliably caught at runtime:
    /// the generated `score` functions index the vector positionally, so a wrong-length
    /// vector yields silently wrong scores (or a panic mid-inference) rather than an error.
    /// Pinning the lengths here makes any single-site change fail loudly.
    #[test]
    fn feature_vector_len_is_pinned() {
        let f = compute_features("int main() { return 0; }", Language::C);

        assert_eq!(
            specialized_features(&f).len(),
            36,
            "specialized vector = 26 base AST + 10 engineered"
        );
        assert_eq!(
            unified_features(&f, Language::C).len(),
            40,
            "unified vector = specialized 36 + 4 language one-hots"
        );
    }

    /// The one-hots are positional, so the order must match `get_feature_cols`. A
    /// reordering here would leave every length assertion above still passing.
    #[test]
    fn language_one_hots_are_positioned_correctly() {
        let f = compute_features("int main() { return 0; }", Language::C);
        let unified = unified_features(&f, Language::C);
        assert_eq!(&unified[36..], &[1.0, 0.0, 0.0, 0.0], "C one-hot");

        let java = unified_features(&f, Language::Java);
        assert_eq!(&java[36..], &[0.0, 0.0, 1.0, 0.0], "Java one-hot");

        // Specialized vectors must NOT carry the one-hots.
        assert_eq!(specialized_features(&f).len(), 36);
    }
}
