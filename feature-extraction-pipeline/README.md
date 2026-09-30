# OJ Feature-Extraction Pipeline

A high-performance Rust tool that walks a pre-organized code-judge dataset, parses every source
file using [Tree-sitter](https://tree-sitter.github.io/), extracts multi-dimensional Abstract Syntax Tree (AST) & structural complexity features per file, and writes a single labeled CSV suitable for training and evaluating XGBoost resource-tier classifiers.

---

## 1. Features Extracted

The pipeline extracts **26 core AST and structural features** in pure Rust, and the modeling pipeline adds **10 engineered interaction ratios / log transforms**. That gives a **36-feature vector** for each per-language model; the unified multi-language model adds 4 language one-hot columns for **40 features** total.

The 26 core features plus the `parse_error_flag` quality flag and the `submission_id` / `language` / `label` metadata columns make up the **30-column CSV**. That header is now a single exported constant, `output::CSV_HEADER`, shared by the writer and every test file (see §4).

### Core AST & Structural Features (Rust Extractor)

| Column                    | Type | Category          | Description                                                                                                                                                                               |
| ------------------------- | ---- | ----------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --- | -------------------- |
| `submission_id`           | text | Metadata          | Submission identifier (filename without extension).                                                                                                                                       |
| `language`                | text | Metadata          | Programming language: `C`, `C++`, `Java`, or `Python`.                                                                                                                                    |
| `nesting_depth`           | uint | Control Flow      | Maximum nesting depth of `for` / `while` / `if` blocks. (`else if` / `elif` chains are flat and do not artificially inflate depth).                                                       |
| `max_loop_depth`          | uint | Loop Topology     | Deepest nesting specifically of loops (`for`, `while`, `do-while`). Correlates directly with polynomial time complexity ($O(N^k)$).                                                       |
| `total_loops`             | uint | Loop Topology     | Total count of all loop statements in the source file.                                                                                                                                    |
| `cyclomatic_complexity`   | uint | Control Flow      | McCabe cyclomatic complexity = $1 +$ decision points (`if`, `for`, `while`, `case`, `&&`, `                                                                                               |     | `, ternary, `elif`). |
| `is_recursive`            | 0/1  | Recursion         | `1` if any function directly calls itself by name inside its own body.                                                                                                                    |
| `recursive_call_count`    | uint | Recursion         | Count of self-call sites in recursive functions. Distinguishes linear recursion ($O(N)$) from branching / tree recursion ($O(2^N)$ divide-and-conquer / backtracking).                    |
| `large_alloc_flag`        | 0/1  | Memory Scale      | `1` if a statically-known allocation exceeds `LARGE_ALLOC_THRESHOLD` (`1_000_000`). Still emitted, now **derived** via `AllocStats.any_large()` — its semantics and existing tests are unchanged. |
| `alloc_size_max`          | uint | Memory Scale      | Largest single statically-known allocation size at any recognised site. Units differ by language (see §2), so the name is deliberately neutral (`size`, not `bytes`).                     |
| `alloc_size_total`        | uint | Memory Scale      | Sum of the statically-known sizes across all recognised allocation sites. Same unit caveat as `alloc_size_max`.                                                                            |
| `alloc_sites`             | uint | Memory Scale      | Number of recognised allocation sites.                                                                                                                                                    |
| `alloc_unknown_sites`     | uint | Memory Scale      | Number of allocation sites whose size is only known at **runtime** (`malloc(n)`, `new byte[chunks]`) — precisely the case the old boolean structurally could not express.                  |
| `has_fast_io`             | 0/1  | Input Scale       | `1` if high-throughput I/O boilerplate is detected (`sync_with_stdio`, `cin.tie`, `BufferedReader`, `StringTokenizer`, `sys.stdin.readline`), signaling large input scale ($N \ge 10^5$). |
| `has_heavy_datastructure` | 0/1  | Collections       | `1` if heavy standard library containers are used (`unordered_map`, `priority_queue`, `BigInteger`, `defaultdict`, `heapq`, `bitset`, `multiset`).                                        |
| `has_modulo_arithmetic`   | 0/1  | Operator Markers  | `1` if modulo (`%`) arithmetic appears — typical of hashing, number theory, and cycle-detection solutions.                                                                                |
| `has_bitmask_ops`         | 0/1  | Operator Markers  | `1` if bitwise operators (`<<`, `>>`, `&`, `\\|`, `^`, `~`) are used — typical of subset-DP and bitmask state encodings.                                                                   |
| `has_graph_adjacency`     | 0/1  | Structure Markers | `1` if adjacency-list / adjacency-matrix construction patterns are detected — signals graph traversal workloads.                                                                          |
| `total_functions`         | uint | Code Scale        | Total count of function, method, and constructor declarations.                                                                                                                            |
| `total_calls`             | uint | Code Scale        | Total count of function and method call sites.                                                                                                                                            |
| `total_subscripts`        | uint | Memory Access     | Total count of array and collection indexing expressions (`a[i]`).                                                                                                                        |
| `total_2d_subscripts`     | uint | Memory Access     | Total count of chained multi-dimensional array accesses (`grid[i][j]`), detecting 2D dynamic programming tables and graph adjacency matrices.                                             |
| `total_arithmetic_ops`    | uint | Arithmetic        | Total count of arithmetic operators (`+`, `-`, `*`, `/`, `%`).                                                                                                                            |
| `max_integer_constant`    | uint | Arithmetic        | Largest integer literal appearing in the source (after evaluating simple constant expressions such as `2e5 + 5` or `1 << 20`), used as a proxy for declared problem scale.                |
| `ast_node_count`          | uint | AST Scale         | Total count of AST nodes generated by Tree-sitter.                                                                                                                                        |
| `ast_depth`               | uint | AST Scale         | Maximum depth of the syntax tree hierarchy.                                                                                                                                               |
| `source_loc`              | uint | Code Volume       | Count of non-empty source lines of code.                                                                                                                                                  |
| `source_chars`            | uint | Code Volume       | Total character count of the source file.                                                                                                                                                 |
| `parse_error_flag`        | 0/1  | Quality           | `1` if the Tree-sitter parse tree contains any `ERROR` node. Enables clean filtering of malformed submissions.                                                                            |
| `label`                   | text | Target            | Target isolation tier: `Light` or `Heavy`.                                                                                                                                                |

### Derived Interaction Features (Training Pipeline) — 10 total

- **`loop_density`**: `total_loops / max(ast_node_count, 1)`
- **`call_density`**: `total_calls / max(ast_node_count, 1)`
- **`subscript_density`**: `total_subscripts / max(ast_node_count, 1)`
- **`branch_density`**: `cyclomatic_complexity / max(source_loc, 1)`
- **`arithmetic_density`**: `total_arithmetic_ops / max(source_loc, 1)`
- **`subscript_2d_ratio`**: `total_2d_subscripts / max(total_subscripts, 1)`
- **`recursion_intensity`**: `recursive_call_count / max(total_functions, 1)`
- **`log_max_constant`**: $\log_{10}(\max(\text{max\_integer\_constant}, 1))$
- **`log_ast_nodes`**: $\ln(1 + \text{ast\_node\_count})$
- **`log_source_chars`**: $\ln(1 + \text{source\_chars})$

> These are defined in [`train_advanced_xgboost.py`](../model-training/train_advanced_xgboost.py), **not** emitted by the Rust extractor — the extractor's CSV contains the 26 core feature columns plus the `parse_error_flag` quality flag and the `submission_id` / `language` / `label` metadata columns (30 columns total).

---

## 2. Memory Allocation & Upfront Pre-Allocation Detection

The extractor evaluates static and compile-time constant arithmetic (e.g. `5 * 1024 * 1024`, `1 << 20`, `2e5 + 5`):

| Language   | Detected Allocation Patterns                                                                                                             |
| ---------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| **C**      | `malloc(bytes)`, `calloc(count, size)`, `realloc(bytes)`, `aligned_alloc(bytes)`, and global fixed-size array declarations `int arr[N]`. |
| **C++**    | `new T[n]`, `v.reserve(n)`, `v.resize(n)`, and multi-dimensional fixed-size array declarations `int dp[N][M]`.                           |
| **Java**   | `new int[n]` array instantiations, `new ArrayList<>(capacity)`, `new HashMap<>(capacity)`.                                               |
| **Python** | Container repetition `[x] * n` (lists/tuples/bytes), `bytearray(n)`.                                                                     |

Each recognised site feeds four measured outputs — `alloc_size_max`, `alloc_size_total`,
`alloc_sites`, `alloc_unknown_sites` — which **replace the old single boolean as the allocation
signal**. `alloc_unknown_sites` is the field that carries the signal the old design could not
express: it counts allocation sites whose size is only known at **runtime** (`malloc(n)`,
`new byte[chunks]`). A runtime-sized allocation is invisible to a static threshold — `const_eval`
fails, so it contributes nothing to `alloc_size_max` or `alloc_size_total` and could never trip
`large_alloc_flag` — yet it is often the allocation that actually matters. Counting the site
recovers that information without inventing a size for it.

`large_alloc_flag` is still emitted, now derived via `AllocStats.any_large()` (`size_max >
LARGE_ALLOC_THRESHOLD`), so its semantics and existing tests hold unchanged.

**Accepted limitation (and how the new fields address it):** an allocation whose size is given by
a *variable* still cannot be *sized* statically — the extractor only knows the
constant/constant-folded dimensions it can evaluate (see the `const_eval` doc comment). But it is
no longer invisible: the site is counted by `alloc_unknown_sites`. Verified behaviour: for a Java
source containing `new byte[chunks][1024 * 1024]` where `chunks` is a variable, `alloc_size_max`
is `1024 * 1024` (the known inner dimension), `alloc_sites` is `2` (each dimension is a site),
`alloc_unknown_sites` is `1` (the outer variable dimension), and `large_alloc_flag` is `1`,
because the constant inner dimension *does* exceed the threshold; the same source with a literal
outer dimension is also `1`.

**Unit asymmetry (documented intent, not a bug):** the underlying APIs are not commensurable.
C `malloc` / `calloc` sizes are **bytes**; Java / Python `new T[n]` and container capacities are
**element counts**. The field names are therefore neutral — `alloc_size_max` / `alloc_size_total`
rather than `bytes_max` — because calling them bytes would be a false claim.

---

## 3. Dataset Layout

The dataset walker traverses directory trees organized by language and resource tier:

```text
<dataset_root>/
├── C/
│   ├── Light/*.c
│   └── Heavy/*.c
├── C++/
│   ├── Light/*.cpp
│   └── Heavy/*.cpp
├── Java/
│   ├── Light/*.java
│   └── Heavy/*.java
└── Python/
    ├── Light/*.py
    └── Heavy/*.py
```

- **Language** is inferred strictly from top-level directory names (`C`, `C++`, `Java`, `Python`).
- **Resource Tier Label** is inferred from the subfolder name (`Light` or `Heavy`).
- Deduplication is enforced on the `(submission_id, language)` composite key.

Both `model-training/codecontests_subset` and `model-training/codenet_subset` follow this layout.

---

## 4. Building & Running

### Prerequisites
- **Rust Toolchain $\ge$ 1.85** (`edition = "2024"`).

### Building Release Binary
```bash
cargo build --release --bin OJ-feature-extraction-spike
```

### Running Feature Extraction
```bash
# Run feature extraction on the dataset:
./target/release/OJ-feature-extraction-spike <dataset_root> [output_features.csv]

# Example (CodeContests subset produced by ../model-training/extract_codecontests.py):
./target/release/OJ-feature-extraction-spike ../model-training/codecontests_subset features.csv

# Example (CodeNet subset produced by ../model-training/extract_codenet.py):
./target/release/OJ-feature-extraction-spike ../model-training/codenet_subset features.csv
```

### Running Tests
```bash
cargo test
```
Runs **53 tests**, all passing: **48 unit** tests, **2 CLI** tests, and **3 integration** tests,
verifying feature extraction edge cases (range-for, do-while, elif chains, recursive self-calls,
array sizing, and CSV serialization). The new tests cover dynamic (runtime-sized) allocation,
magnitude capture for statically-known sizes, multi-dimensional Java arrays counting each
dimension as a site, and a guard that non-allocating calls such as `printf` are **not** counted
as allocation sites.

The CSV header lives in one place — the exported constant `output::CSV_HEADER`, shared by the
writer and all test files. It used to be duplicated across three files, which silently broke two
tests whenever a column was added.

---

## 5. Model Performance Benchmark (CodeNet 71,218 Dataset)
> ⚠️ **Historical / superseded.** The figures in this section are from an earlier
> CodeNet run and **do not describe the models currently compiled into the judge**.
> See the authoritative table in [`model-training/README.md`](../model-training/README.md) §4.
>
> The accuracy / F1 / precision / recall / ROC-AUC / threshold values below are carried over
> from that earlier run and are **UNVERIFIED - needs measurement** against the current models.

Trained with GPU-accelerated XGBoost using 5-fold **Problem-Grouped Cross-Validation** (`GroupKFold` on `problem_id` to evaluate strictly against unseen competitive programming problems):

| Model Architecture               | Test Accuracy                 | F1-Score      | Precision | Recall    | ROC-AUC      | Optimal Threshold |
| -------------------------------- | ----------------------------- | ------------- | --------- | --------- | ------------ | ----------------- |
| **Specialized C++ Model**        | **$90.01\%$** _(std: 90.01%)_ | **$90.91\%$** | $88.54\%$ | $93.41\%$ | **$0.9612$** | $0.439$           |
| **Specialized C Model**          | **$89.26\%$** _(std: 91.95%)_ | **$68.82\%$** | $55.98\%$ | $89.31\%$ | **$0.9575$** | $0.200$           |
| **Specialized Python Model**     | **$79.45\%$** _(std: 79.13%)_ | **$82.01\%$** | $80.33\%$ | $83.76\%$ | **$0.8740$** | $0.456$           |
| **Specialized Java Model**       | **$78.20\%$** _(std: 77.52%)_ | **$81.54\%$** | $82.03\%$ | $81.06\%$ | **$0.8537$** | $0.482$           |
| **Unified Multi-Language Model** | **$83.71\%$** _(std: 83.76%)_ | **$84.81\%$** | $80.19\%$ | $89.99\%$ | **$0.9149$** | $0.457$           |

All models are compiled via **m2cgen** into Rust source (`server/src/generated/*.rs`) by
[`regenerate_models.sh`](../model-training/regenerate_models.sh) and baked into the judge
binary at build time — inference has no Python/C dependency at runtime. `regenerate_models.sh`
verifies each trained model's own `num_features()` before writing and refuses on mismatch, so a
feature-count drift cannot silently ship a mismatched model.
