//! Row assembly and CSV output for the feature-extraction pipeline.
//!
//! This module turns walked [`Sample`]s into labeled [`Row`]s (applying the
//! feature extraction) and serializes them to CSV. It lives in the library so
//! the end-to-end pipeline can be exercised by integration tests without
//! spawning the CLI binary.

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::features;
use crate::walker::Sample;

/// One output CSV row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub submission_id: String,
    pub language: String,
    pub nesting_depth: u32,
    pub max_loop_depth: u32,
    pub total_loops: u32,
    pub cyclomatic_complexity: u32,
    pub is_recursive: u32,
    pub recursive_call_count: u32,
    pub large_alloc_flag: u32,
    pub alloc_size_max: u64,
    pub alloc_size_total: u64,
    pub alloc_sites: u32,
    pub alloc_unknown_sites: u32,
    pub has_fast_io: u32,
    pub has_heavy_datastructure: u32,
    pub has_modulo_arithmetic: u32,
    pub has_bitmask_ops: u32,
    pub has_graph_adjacency: u32,
    pub total_functions: u32,
    pub total_calls: u32,
    pub total_subscripts: u32,
    pub total_2d_subscripts: u32,
    pub total_arithmetic_ops: u32,
    pub max_integer_constant: u64,
    pub ast_node_count: u32,
    pub ast_depth: u32,
    pub source_loc: u32,
    pub source_chars: u32,
    pub parse_error_flag: u32,
    pub label: String,
}

/// Diagnostics accumulated while assembling rows.
#[derive(Debug, Clone, Default)]
pub struct Summary {
    pub parse_error_count: usize,
    pub dup_skipped: usize,
    pub per_lang_tier: HashMap<(String, String), usize>,
}

/// Build one [`Row`] per sample, de-duplicating on `(submission_id, language)`.
pub fn build_rows(samples: &[Sample]) -> (Vec<Row>, Summary) {
    let mut rows = Vec::with_capacity(samples.len());
    let mut seen: HashSet<(String, String)> = HashSet::with_capacity(samples.len());
    let mut summary = Summary::default();

    for sample in samples {
        let key = (
            sample.submission_id.clone(),
            sample.language.as_str().to_string(),
        );
        if !seen.insert(key) {
            summary.dup_skipped += 1;
            continue;
        }

        // Pure, reusable logic: source + language in, features out.
        let feats = features::compute_features(&sample.source, sample.language);

        if feats.parse_error_flag {
            summary.parse_error_count += 1;
        }
        *summary
            .per_lang_tier
            .entry((sample.language.as_str().to_string(), sample.label.clone()))
            .or_insert(0) += 1;

        rows.push(Row {
            submission_id: sample.submission_id.clone(),
            language: sample.language.as_str().to_string(),
            nesting_depth: feats.nesting_depth,
            max_loop_depth: feats.max_loop_depth,
            total_loops: feats.total_loops,
            cyclomatic_complexity: feats.cyclomatic_complexity,
            is_recursive: feats.is_recursive as u32,
            recursive_call_count: feats.recursive_call_count,
            large_alloc_flag: feats.large_alloc_flag as u32,
            alloc_size_max: feats.alloc_size_max,
            alloc_size_total: feats.alloc_size_total,
            alloc_sites: feats.alloc_sites,
            alloc_unknown_sites: feats.alloc_unknown_sites,
            has_fast_io: feats.has_fast_io as u32,
            has_heavy_datastructure: feats.has_heavy_datastructure as u32,
            has_modulo_arithmetic: feats.has_modulo_arithmetic as u32,
            has_bitmask_ops: feats.has_bitmask_ops as u32,
            has_graph_adjacency: feats.has_graph_adjacency as u32,
            total_functions: feats.total_functions,
            total_calls: feats.total_calls,
            total_subscripts: feats.total_subscripts,
            total_2d_subscripts: feats.total_2d_subscripts,
            total_arithmetic_ops: feats.total_arithmetic_ops,
            max_integer_constant: feats.max_integer_constant,
            ast_node_count: feats.ast_node_count,
            ast_depth: feats.ast_depth,
            source_loc: feats.source_loc,
            source_chars: feats.source_chars,
            parse_error_flag: feats.parse_error_flag as u32,
            label: sample.label.clone(),
        });
    }

    (rows, summary)
}

/// The CSV header, exported as the single source of truth.
///
/// It used to be written out literally in three places (this writer plus two
/// test files), so adding a feature column silently broke whichever copies were
/// not updated — which is exactly what happened when the allocation columns were
/// added. Tests now assert against this constant instead of a second literal.
pub const CSV_HEADER: &str = "submission_id,language,nesting_depth,max_loop_depth,total_loops,cyclomatic_complexity,is_recursive,recursive_call_count,large_alloc_flag,alloc_size_max,alloc_size_total,alloc_sites,alloc_unknown_sites,has_fast_io,has_heavy_datastructure,has_modulo_arithmetic,has_bitmask_ops,has_graph_adjacency,total_functions,total_calls,total_subscripts,total_2d_subscripts,total_arithmetic_ops,max_integer_constant,ast_node_count,ast_depth,source_loc,source_chars,parse_error_flag,label";

/// Write the CSV, UTF-8, header included, one row per entry.
pub fn write_csv(path: &Path, rows: &[Row]) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut w = BufWriter::new(file);
    w.write_all(CSV_HEADER.as_bytes())?;
    w.write_all(b"\n")?;
    for r in rows {
        writeln!(
            w,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            csv_escape(&r.submission_id),
            csv_escape(&r.language),
            r.nesting_depth,
            r.max_loop_depth,
            r.total_loops,
            r.cyclomatic_complexity,
            r.is_recursive,
            r.recursive_call_count,
            r.large_alloc_flag,
            r.alloc_size_max,
            r.alloc_size_total,
            r.alloc_sites,
            r.alloc_unknown_sites,
            r.has_fast_io,
            r.has_heavy_datastructure,
            r.has_modulo_arithmetic,
            r.has_bitmask_ops,
            r.has_graph_adjacency,
            r.total_functions,
            r.total_calls,
            r.total_subscripts,
            r.total_2d_subscripts,
            r.total_arithmetic_ops,
            r.max_integer_constant,
            r.ast_node_count,
            r.ast_depth,
            r.source_loc,
            r.source_chars,
            r.parse_error_flag,
            csv_escape(&r.label)
        )?;
    }
    w.flush()
}

/// Minimal CSV field escaping: quote any field containing a comma, quote, or
/// newline, doubling embedded quotes.
pub fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::Language;

    fn sample(id: &str, lang: Language, label: &str, src: &str) -> Sample {
        Sample {
            submission_id: id.to_string(),
            language: lang,
            label: label.to_string(),
            source: src.to_string(),
        }
    }

    #[test]
    fn csv_escape_plain_field_unchanged() {
        assert_eq!(csv_escape("s00000001"), "s00000001");
        assert_eq!(csv_escape("C++"), "C++");
        assert_eq!(csv_escape("Heavy"), "Heavy");
    }

    #[test]
    fn csv_escape_quotes_comma_and_newline() {
        assert_eq!(csv_escape("a,b"), "\"a,b\"");
        assert_eq!(csv_escape("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(csv_escape("line1\nline2"), "\"line1\nline2\"");
    }

    #[test]
    fn build_rows_computes_features_and_dedups() {
        let samples = vec![
            sample("a", Language::C, "Light", "int main(void){return 0;}"),
            // Same (submission_id, language) in a second tier -> duplicate.
            sample("a", Language::C, "Heavy", "int main(void){return 0;}"),
            sample(
                "b",
                Language::Python,
                "Heavy",
                "def f(n):\n    if n <= 1:\n        return 1\n    return n * f(n - 1)\n",
            ),
            sample(
                "b",
                Language::Cpp,
                "Heavy",
                "int main(){int *p = new int[3000000];}",
            ),
        ];
        let (rows, summary) = build_rows(&samples);

        assert_eq!(rows.len(), 3, "one of the two 'a'/C samples must be deduped");
        assert_eq!(summary.dup_skipped, 1);

        // The surviving 'a'/C row is the first one seen (order preserved).
        let a_row = rows.iter().find(|r| r.submission_id == "a").unwrap();
        assert_eq!(a_row.language, "C");
        assert_eq!(a_row.label, "Light");
        assert_eq!(a_row.nesting_depth, 0);
        assert_eq!(a_row.cyclomatic_complexity, 1);

        let py_row = rows
            .iter()
            .find(|r| r.submission_id == "b" && r.language == "Python")
            .unwrap();
        assert_eq!(py_row.nesting_depth, 1);
        assert_eq!(py_row.cyclomatic_complexity, 2);
        assert_eq!(py_row.is_recursive, 1);

        let cpp_row = rows.iter().find(|r| r.language == "C++").unwrap();
        assert_eq!(cpp_row.large_alloc_flag, 1);

        assert_eq!(
            summary.per_lang_tier.get(&("C".to_string(), "Light".to_string())),
            Some(&1)
        );
        assert_eq!(
            summary
                .per_lang_tier
                .get(&("Python".to_string(), "Heavy".to_string())),
            Some(&1)
        );
    }

    #[test]
    fn build_rows_flags_parse_errors() {
        let samples = vec![sample(
            "broken",
            Language::Python,
            "Light",
            "def f(:\n    x = (\n    return x",
        )];
        let (rows, summary) = build_rows(&samples);
        assert_eq!(summary.parse_error_count, 1);
        assert_eq!(rows[0].parse_error_flag, 1);
    }

    #[test]
    fn write_csv_produces_header_and_rows() {
        let tmp = std::env::temp_dir().join(format!(
            "output_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let rows = vec![Row {
            submission_id: "s001".into(),
            language: "C".into(),
            nesting_depth: 2,
            max_loop_depth: 1,
            total_loops: 1,
            cyclomatic_complexity: 4,
            is_recursive: 1,
            recursive_call_count: 1,
            large_alloc_flag: 0,
            alloc_size_max: 1024,
            alloc_size_total: 4096,
            alloc_sites: 3,
            alloc_unknown_sites: 1,
            has_fast_io: 0,
            has_heavy_datastructure: 0,
            has_modulo_arithmetic: 0,
            has_bitmask_ops: 0,
            has_graph_adjacency: 0,
            total_functions: 1,
            total_calls: 2,
            total_subscripts: 0,
            total_2d_subscripts: 0,
            total_arithmetic_ops: 3,
            max_integer_constant: 100,
            ast_node_count: 25,
            ast_depth: 6,
            source_loc: 10,
            source_chars: 150,
            parse_error_flag: 0,
            label: "Light".into(),
        }];
        write_csv(&tmp, &rows).unwrap();
        let text = std::fs::read_to_string(&tmp).unwrap();
        let _ = std::fs::remove_file(&tmp);
        let mut lines = text.lines();
        assert_eq!(lines.next().unwrap(), super::CSV_HEADER);
        assert_eq!(lines.next().unwrap(), "s001,C,2,1,1,4,1,1,0,1024,4096,3,1,0,0,0,0,0,1,2,0,0,3,100,25,6,10,150,0,Light");
        assert_eq!(lines.next(), None);
    }
}
