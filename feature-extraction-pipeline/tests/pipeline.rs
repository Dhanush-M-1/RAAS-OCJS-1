//! End-to-end integration tests: mock dataset on disk → walker → feature
//! extraction → CSV, all through the library API (no subprocess).
//!
//! These tests mirror what the CLI binary does, but drive the library
//! functions directly so individual rows can be asserted precisely.

mod common;

use common::{write, TempDir};
use feature_extraction::{output, walker};

/// Build a small mock CodeNet-style dataset on disk.
///
/// Expected feature values are hand-computed and asserted in the tests below.
fn build_mock_dataset(root: &std::path::Path) {
    // C / Light: for > while > if  -> depth 3, complexity 1 + 3 = 4
    write(
        root,
        "C/Light/s001.c",
        r#"int f(int a, int b) {
    for (int i = 0; i < a; i++) {
        while (b > 0) {
            if (i % 2 == 0) {
                b--;
            }
        }
    }
    return b;
}
"#,
    );

    // C / Heavy: recursion + large literal malloc(5MB)
    write(
        root,
        "C/Heavy/s002.c",
        r#"#include <stdlib.h>
long fact(long n) {
    if (n <= 1) return 1;
    char *tmp = malloc(5 * 1024 * 1024);
    return n * fact(n - 1);
}
"#,
    );

    // Same submission_id in two tiers of the same language -> duplicate pair
    // that `build_rows` must de-duplicate (Heavy is walked before Light).
    write(root, "C/Heavy/s007.c", "int main(void){return 0;}");
    write(root, "C/Light/s007.c", "int main(void){return 0;}");

    // C++ / Heavy: large `reserve` and `new int[n]` literals
    write(
        root,
        "C++/Heavy/s003.cpp",
        r#"#include <vector>
int main() {
    std::vector<int> v;
    v.reserve(2000000);
    int *arr = new int[5000];
    int *big = new int[3000000];
    return 0;
}
"#,
    );

    // Java / Light: else-if chain -> depth 1 (chain collapsed), complexity 4
    write(
        root,
        "Java/Light/s004.java",
        r#"class F {
    int f(int x) {
        if (x < 0) return -1;
        else if (x == 0) return 0;
        else if (x < 100) return 1;
        else return 2;
    }
}
"#,
    );

    // Python / Heavy: recursion + large list repetition
    write(
        root,
        "Python/Heavy/s005.py",
        r#"def fact(n):
    if n <= 1:
        return 1
    a = [0] * 5000000
    return n * fact(n - 1)
"#,
    );

    // Python / Light: deliberately broken -> parse_error_flag=1
    write(root, "Python/Light/s006.py", "def f(:\n    x = (\n    return x");
}

fn row<'a>(rows: &'a [output::Row], id: &str, lang: &str) -> &'a output::Row {
    rows.iter()
        .find(|r| r.submission_id == id && r.language == lang)
        .unwrap_or_else(|| panic!("missing row for {id}/{lang}"))
}

#[test]
fn pipeline_end_to_end_matches_hand_computed_values() {
    let tmp = TempDir::new("e2e");
    let root = tmp.path();
    build_mock_dataset(root);

    let samples = walker::walk_dataset(root);
    // 8 files on disk; the s007/C duplication is a row-assembly concern, so
    // the walker still returns all 8.
    assert_eq!(samples.len(), 8);

    let (rows, summary) = output::build_rows(&samples);
    assert_eq!(rows.len(), 7, "the s007/C duplicate must be removed");
    assert_eq!(summary.dup_skipped, 1);
    assert_eq!(summary.parse_error_count, 1);

    // C / Light nested loops.
    let r = row(&rows, "s001", "C");
    assert_eq!(
        (r.nesting_depth, r.cyclomatic_complexity, r.is_recursive, r.large_alloc_flag, r.parse_error_flag, r.label.as_str()),
        (3, 4, 0, 0, 0, "Light")
    );

    // C / Heavy recursive + large alloc.
    let r = row(&rows, "s002", "C");
    assert_eq!(
        (r.nesting_depth, r.cyclomatic_complexity, r.is_recursive, r.large_alloc_flag, r.parse_error_flag, r.label.as_str()),
        (1, 2, 1, 1, 0, "Heavy")
    );

    // C++ / Heavy large allocations (no control flow).
    let r = row(&rows, "s003", "C++");
    assert_eq!(
        (r.nesting_depth, r.cyclomatic_complexity, r.is_recursive, r.large_alloc_flag, r.parse_error_flag, r.label.as_str()),
        (0, 1, 0, 1, 0, "Heavy")
    );

    // Java / Light else-if chain.
    let r = row(&rows, "s004", "Java");
    assert_eq!(
        (r.nesting_depth, r.cyclomatic_complexity, r.is_recursive, r.large_alloc_flag, r.parse_error_flag, r.label.as_str()),
        (1, 4, 0, 0, 0, "Light")
    );

    // Python / Heavy recursive + large list repetition.
    let r = row(&rows, "s005", "Python");
    assert_eq!(
        (r.nesting_depth, r.cyclomatic_complexity, r.is_recursive, r.large_alloc_flag, r.parse_error_flag, r.label.as_str()),
        (1, 2, 1, 1, 0, "Heavy")
    );

    // Python / Light broken file is flagged.
    let r = row(&rows, "s006", "Python");
    assert_eq!(r.parse_error_flag, 1);

    // The duplicated s007/C survives exactly once, as the Heavy tier (tiers
    // are walked in sorted order, Heavy before Light).
    let s007: Vec<&output::Row> = rows
        .iter()
        .filter(|r| r.submission_id == "s007" && r.language == "C")
        .collect();
    assert_eq!(s007.len(), 1);
    assert_eq!(s007[0].label, "Heavy");

    // CSV serialization round-trips to the expected header + row count.
    // The header is asserted against the exported constant rather than a second
    // literal, so adding a feature column cannot leave this test stale.
    let csv_path = tmp.path().join("out.csv");
    output::write_csv(&csv_path, &rows).unwrap();
    let text = std::fs::read_to_string(&csv_path).unwrap();
    let mut lines = text.lines();
    assert_eq!(lines.next().unwrap(), output::CSV_HEADER);
    let data_lines = lines.filter(|l| !l.is_empty()).count();
    assert_eq!(data_lines, 7);
}

#[test]
fn pipeline_handles_full_language_tier_matrix() {
    let tmp = TempDir::new("matrix");
    let root = tmp.path();
    // NOTE: each file must have a unique stem — a shared stem like `m` across
    // the two tiers of the same language would be de-duplicated by `build_rows`
    // on the (submission_id, language) key.
    for (lang, ext) in [("C", "c"), ("C++", "cpp"), ("Java", "java"), ("Python", "py")] {
        for tier in ["Light", "Heavy"] {
            let src = match lang {
                "Java" => "class M { public static void main(String[] a) { return; } }",
                "Python" => "print(1)",
                _ => "int main(void){return 0;}",
            };
            write(root, &format!("{lang}/{tier}/m_{tier}.{ext}"), src);
        }
    }

    let samples = walker::walk_dataset(root);
    assert_eq!(samples.len(), 8, "4 languages x 2 tiers");

    let (rows, summary) = output::build_rows(&samples);
    assert_eq!(rows.len(), 8);
    assert_eq!(summary.dup_skipped, 0);
    assert_eq!(summary.parse_error_count, 0, "all mock files are valid");

    for lang in ["C", "C++", "Java", "Python"] {
        for tier in ["Light", "Heavy"] {
            assert_eq!(
                summary.per_lang_tier.get(&(lang.to_string(), tier.to_string())),
                Some(&1),
                "expected exactly one sample in {lang}/{tier}"
            );
        }
    }
}

#[test]
fn walker_preserves_duplicate_tier_files_before_dedup() {
    // The walker alone must NOT de-duplicate: it returns every file on disk.
    let tmp = TempDir::new("walker-dup");
    let root = tmp.path();
    write(root, "C/Heavy/x.c", "int main(void){return 0;}");
    write(root, "C/Light/x.c", "int main(void){return 0;}");

    let samples = walker::walk_dataset(root);
    assert_eq!(samples.len(), 2, "walker sees both tier copies");
    assert!(samples.iter().any(|s| s.label == "Heavy"));
    assert!(samples.iter().any(|s| s.label == "Light"));

    // De-duplication happens only at row assembly time.
    let (rows, summary) = output::build_rows(&samples);
    assert_eq!(rows.len(), 1);
    assert_eq!(summary.dup_skipped, 1);
}
