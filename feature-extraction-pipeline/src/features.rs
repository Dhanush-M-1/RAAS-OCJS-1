//! Pure feature-extraction logic (pipeline steps 3-4).
//!
//! Everything in this module takes a source string + a [`Language`] and returns
//! feature values. It performs **no file I/O and no directory walking**, so the
//! same code can be reused later inside a live judge on a single incoming
//! submission, not only in this offline batch context.
//!
//! All tree-sitter node type names used below were verified against real parse
//! trees (see `src/bin/probe.rs`), not guessed.

use tree_sitter::{Node, Parser};

use crate::language::Language;

/// Allocations whose statically-known size is strictly **above** this threshold
/// are flagged as "large". This is a named constant (not a magic number) so it
/// can be tuned in one place. Units are intentionally uniform even though APIs
/// size in bytes (malloc/calloc) vs. element counts (`new T[n]`, `.reserve(n)`,
/// Java collection initial capacities, Python container repetition).
pub const LARGE_ALLOC_THRESHOLD: u64 = 1_000_000;

/// Computed features for a single submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Features {
    /// Deepest nesting of for/while/if constructs. else-if / elif chains do NOT
    /// add extra depth (see `compute_nesting_depth` for the verified rationale).
    pub nesting_depth: u32,
    /// Deepest nesting specifically of loops (for / while / do-while).
    pub max_loop_depth: u32,
    /// Total count of all loops in the source file.
    pub total_loops: u32,
    /// McCabe cyclomatic complexity = 1 + decision points (if, for, while,
    /// case, &&, ||, ternary).
    pub cyclomatic_complexity: u32,
    /// `true` if any function calls itself by name within its own body.
    pub is_recursive: bool,
    /// Total count of recursive self-call sites (branching factor indicator).
    pub recursive_call_count: u32,
    /// `true` if a static allocation or global array has size above [`LARGE_ALLOC_THRESHOLD`].
    pub large_alloc_flag: bool,
    /// Largest statically-known allocation. Unit is bytes for the C allocator
    /// family and element counts for `new T[n]` / `[0] * n` (see [`AllocStats`]).
    pub alloc_size_max: u64,
    /// Sum of every statically-known allocation. Same unit caveat as `alloc_size_max`.
    pub alloc_size_total: u64,
    /// Count of recognised allocation sites (static or not).
    pub alloc_sites: u32,
    /// Recognised allocation sites whose size is NOT statically known — i.e.
    /// input-driven allocations like `malloc(n)` or `new byte[chunks][1 << 20]`.
    pub alloc_unknown_sites: u32,
    /// `true` if Fast I/O / bulk stream reading idioms are detected in source.
    pub has_fast_io: bool,
    /// `true` if heavy standard library structures (hash maps, trees, queues) are detected.
    pub has_heavy_datastructure: bool,
    /// `true` if standard prime modulo arithmetic (10^9+7, 998244353) is detected.
    pub has_modulo_arithmetic: bool,
    /// `true` if bitmask DP or bitwise operators/builtins are detected.
    pub has_bitmask_ops: bool,
    /// `true` if graph adjacency lists/matrices or edge arrays are detected.
    pub has_graph_adjacency: bool,
    /// Total number of function / method declarations.
    pub total_functions: u32,
    /// Total number of function / method call sites.
    pub total_calls: u32,
    /// Total array / collection index access expressions (`a[i]`).
    pub total_subscripts: u32,
    /// Total multi-dimensional array / matrix index expressions (`a[i][j]`).
    pub total_2d_subscripts: u32,
    /// Total binary arithmetic operation count (+, -, *, /, %).
    pub total_arithmetic_ops: u32,
    /// Largest integer literal / constant in source (excluding prime modulos).
    pub max_integer_constant: u64,
    /// Total count of AST nodes.
    pub ast_node_count: u32,
    /// Maximum depth of the AST hierarchy.
    pub ast_depth: u32,
    /// Count of non-empty source lines of code.
    pub source_loc: u32,
    /// Total character count of source code.
    pub source_chars: u32,
    /// `true` if the parse tree contains any ERROR node.
    pub parse_error_flag: bool,
    /// Count of ERROR nodes in the tree (informational / diagnostics).
    pub error_node_count: usize,
}

/// Pure entry point: source code + language in, feature values out.
pub fn compute_features(source: &str, lang: Language) -> Features {
    let mut parser = Parser::new();
    parser
        .set_language(lang.tree_sitter_language())
        .expect("grammar is a compile-time constant; loading cannot fail");
    let tree = parser
        .parse(source, None)
        .expect("tree-sitter parse unexpectedly returned None");

    let root = tree.root_node();
    let error_node_count = count_error_nodes(root);
    let parse_error_flag = error_node_count > 0;

    let nesting_depth = compute_nesting_depth(root, lang);
    let (max_loop_depth, total_loops) = compute_loop_metrics(root, lang);
    let cyclomatic_complexity = 1 + count_decision_points(root, source);
    let (is_recursive, recursive_call_count) = detect_recursion_with_count(root, lang, source);
    let alloc = detect_large_allocation(root, lang, source);
    let large_alloc_flag = alloc.any_large();
    let has_fast_io = detect_fast_io(source, lang);
    let has_heavy_datastructure = detect_heavy_datastructure(source, lang);
    let has_modulo_arithmetic = detect_modulo_arithmetic(source);
    let has_graph_adjacency = detect_graph_adjacency(source);

    let (
        total_functions,
        total_calls,
        total_subscripts,
        total_2d_subscripts,
        total_arithmetic_ops,
        has_bitmask_found,
        max_integer_constant,
        ast_node_count,
        ast_depth,
    ) = compute_ast_metrics(root, lang, source);

    let has_bitmask_ops = has_bitmask_found || detect_bitmask_text(source);

    let source_loc = source.lines().filter(|l| !l.trim().is_empty()).count() as u32;
    let source_chars = source.len() as u32;

    Features {
        nesting_depth,
        max_loop_depth,
        total_loops,
        cyclomatic_complexity,
        is_recursive,
        recursive_call_count,
        large_alloc_flag,
        alloc_size_max: alloc.size_max,
        alloc_size_total: alloc.size_total,
        alloc_sites: alloc.sites,
        alloc_unknown_sites: alloc.unknown_sites,
        has_fast_io,
        has_heavy_datastructure,
        has_modulo_arithmetic,
        has_bitmask_ops,
        has_graph_adjacency,
        total_functions,
        total_calls,
        total_subscripts,
        total_2d_subscripts,
        total_arithmetic_ops,
        max_integer_constant,
        ast_node_count,
        ast_depth,
        source_loc,
        source_chars,
        parse_error_flag,
        error_node_count,
    }
}

/// Visit every node in the subtree exactly once (iterative DFS; no recursion so
/// it is safe even on deep trees).
fn for_each_node<'tree>(root: Node<'tree>, mut f: impl FnMut(Node<'tree>)) {
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        f(node);
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            stack.push(child);
        }
    }
}

fn count_error_nodes(root: Node) -> usize {
    let mut count = 0;
    for_each_node(root, |n| {
        if n.is_error() {
            count += 1;
        }
    });
    count
}

// ---------------------------------------------------------------------------
// nesting_depth
// ---------------------------------------------------------------------------

/// Deepest nesting of loop/conditional constructs (for/while/if).
///
/// else-if chain structure, **verified from real parse trees** (probe output):
/// - C/C++: `if ... else if ...` nests the next `if_statement` inside an
///   `else_clause`. Naively descending would overcount depth, so those
///   continuation `if_statement`s are collapsed (kept at the same depth).
/// - Java: the continuation `if_statement` is the `alternative` field of the
///   parent `if_statement`, directly after `else`. Same collapsing applies.
/// - Python: `elif` clauses are FLAT siblings of the same `if_statement`
///   (multiple `elif_clause` nodes at one level), so there is nothing to
///   collapse there.
fn compute_nesting_depth(root: Node, lang: Language) -> u32 {
    let mut max = 0u32;
    walk_nesting(root, 0, lang, &mut max);
    max
}

fn walk_nesting(node: Node, depth: u32, lang: Language, max: &mut u32) {
    let is_control =
        matches!(node.kind(), "if_statement" | "for_statement" | "while_statement");
    let is_continuation = is_else_if_continuation(node, lang);

    let child_depth = if is_control && !is_continuation {
        *max = (*max).max(depth + 1);
        depth + 1
    } else {
        depth
    };

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_nesting(child, child_depth, lang, max);
    }
}

/// True when `node` is an `if_statement` that continues an `else if` chain and
/// therefore must NOT add extra nesting depth.
fn is_else_if_continuation(node: Node, lang: Language) -> bool {
    if node.kind() != "if_statement" {
        return false;
    }
    match lang {
        // C/C++: an else-if continuation lives directly inside an `else_clause`.
        Language::C | Language::Cpp => match node.parent() {
            Some(p) => p.kind() == "else_clause",
            None => false,
        },
        // Java: the continuation is the `alternative` field of a parent
        // `if_statement`.
        Language::Java => match node.parent() {
            Some(p) => {
                p.kind() == "if_statement"
                    && p.child_by_field_name("alternative")
                        .map(|a| a == node)
                        .unwrap_or(false)
            }
            None => false,
        },
        // Python elif is a flat sibling; no nested if continuation exists.
        Language::Python => false,
    }
}

// ---------------------------------------------------------------------------
// loop metrics (max_loop_depth and total_loops)
// ---------------------------------------------------------------------------

fn is_loop_node(node: Node, lang: Language) -> bool {
    match lang {
        Language::C | Language::Cpp => matches!(
            node.kind(),
            "for_statement" | "while_statement" | "do_statement" | "for_range_loop"
        ),
        Language::Java => matches!(
            node.kind(),
            "for_statement" | "enhanced_for_statement" | "while_statement" | "do_statement"
        ),
        Language::Python => matches!(node.kind(), "for_statement" | "while_statement"),
    }
}

fn compute_loop_metrics(root: Node, lang: Language) -> (u32, u32) {
    let mut max_depth = 0u32;
    let mut total_loops = 0u32;
    walk_loop_nesting(root, 0, lang, &mut max_depth, &mut total_loops);
    (max_depth, total_loops)
}

fn walk_loop_nesting(node: Node, depth: u32, lang: Language, max: &mut u32, total: &mut u32) {
    let is_loop = is_loop_node(node, lang);
    let child_depth = if is_loop {
        *total += 1;
        *max = (*max).max(depth + 1);
        depth + 1
    } else {
        depth
    };

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_loop_nesting(child, child_depth, lang, max, total);
    }
}

// ---------------------------------------------------------------------------
// AST structure metrics (functions, calls, subscripts, 2D access, arithmetic, constants)
// ---------------------------------------------------------------------------

fn compute_ast_metrics(
    root: Node,
    lang: Language,
    source: &str,
) -> (u32, u32, u32, u32, u32, bool, u64, u32, u32) {
    let bytes = source.as_bytes();
    let mut total_functions = 0u32;
    let mut total_calls = 0u32;
    let mut total_subscripts = 0u32;
    let mut total_2d_subscripts = 0u32;
    let mut total_arithmetic_ops = 0u32;
    let mut has_bitmask_found = false;
    let mut max_integer_constant = 0u64;
    let mut ast_node_count = 0u32;
    let mut max_ast_depth = 0u32;

    let mut stack = vec![(root, 1u32)];
    while let Some((node, depth)) = stack.pop() {
        ast_node_count += 1;
        max_ast_depth = max_ast_depth.max(depth);

        let kind = node.kind();

        // Track maximum numeric constants
        match kind {
            "number_literal"
            | "decimal_integer_literal"
            | "integer"
            | "decimal_float_literal"
            | "hex_integer_literal" => {
                if let Ok(text) = node.utf8_text(bytes) {
                    if let Some(val) = parse_num(text) {
                        if val != 1_000_000_007
                            && val != 998_244_353
                            && val != 1_000_000_009
                            && val != 0x3f3f3f3f
                            && val != 0x7fffffff
                            && val < (1u64 << 62)
                        {
                            max_integer_constant = max_integer_constant.max(val);
                        }
                    }
                }
            }
            "binary_expression" | "binary_operator" => {
                if let Some(op_node) = node.child_by_field_name("operator") {
                    if let Ok(op) = op_node.utf8_text(bytes) {
                        match op {
                            "+" | "-" | "*" | "/" | "%" => total_arithmetic_ops += 1,
                            "<<" | ">>" | "&" | "|" | "^" => has_bitmask_found = true,
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }

        match lang {
            Language::C | Language::Cpp => match kind {
                "function_definition" => total_functions += 1,
                "call_expression" => total_calls += 1,
                "subscript_expression" => {
                    total_subscripts += 1;
                    if let Some(arg) = node.child_by_field_name("argument") {
                        if arg.kind() == "subscript_expression" {
                            total_2d_subscripts += 1;
                        }
                    }
                }
                _ => {}
            },
            Language::Java => match kind {
                "method_declaration" | "constructor_declaration" => total_functions += 1,
                "method_invocation" | "constructor_invocation" => total_calls += 1,
                "array_access" => {
                    total_subscripts += 1;
                    if let Some(arr) = node.child_by_field_name("array") {
                        if arr.kind() == "array_access" {
                            total_2d_subscripts += 1;
                        }
                    }
                }
                _ => {}
            },
            Language::Python => match kind {
                "function_definition" => total_functions += 1,
                "call" => total_calls += 1,
                "subscript" => {
                    total_subscripts += 1;
                    if let Some(val) = node.child_by_field_name("value") {
                        if val.kind() == "subscript" {
                            total_2d_subscripts += 1;
                        }
                    }
                }
                _ => {}
            },
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            stack.push((child, depth + 1));
        }
    }

    (
        total_functions,
        total_calls,
        total_subscripts,
        total_2d_subscripts,
        total_arithmetic_ops,
        has_bitmask_found,
        max_integer_constant,
        ast_node_count,
        max_ast_depth,
    )
}

fn detect_modulo_arithmetic(source: &str) -> bool {
    source.contains("1000000007")
        || source.contains("998244353")
        || source.contains("1000000009")
        || source.contains("1e9 + 7")
        || source.contains("1e9+7")
        || source.contains("% MOD")
        || source.contains("% mod")
        || source.contains("pow_mod")
}

fn detect_graph_adjacency(source: &str) -> bool {
    source.contains("adj[")
        || source.contains("head[")
        || source.contains("visited[")
        || source.contains("edges[")
        || source.contains("vector<int> adj")
        || source.contains("vector<edge>")
        || source.contains("vector<pair")
        || source.contains("List<Integer>[]")
        || source.contains("ArrayList[]")
}

fn detect_bitmask_text(source: &str) -> bool {
    source.contains("1 <<")
        || source.contains("1<<")
        || source.contains("__builtin_popcount")
        || source.contains("__builtin_clz")
        || source.contains(".bit_count()")
}

fn detect_fast_io(source: &str, lang: Language) -> bool {
    match lang {
        Language::C => source.contains("getchar_unlocked") || source.contains("fread") || source.contains("fwrite"),
        Language::Cpp => {
            source.contains("sync_with_stdio")
                || source.contains("cin.tie")
                || source.contains("getchar_unlocked")
                || source.contains("fread")
        }
        Language::Java => {
            source.contains("BufferedReader")
                || source.contains("StringTokenizer")
                || source.contains("FastScanner")
                || source.contains("FastReader")
                || source.contains("CustomScanner")
        }
        Language::Python => {
            source.contains("stdin.readline")
                || source.contains("stdin.read")
                || source.contains("stdout.write")
        }
    }
}

fn detect_heavy_datastructure(source: &str, lang: Language) -> bool {
    match lang {
        Language::C => source.contains("qsort") || source.contains("bsearch"),
        Language::Cpp => {
            source.contains("unordered_map")
                || source.contains("unordered_set")
                || source.contains("priority_queue")
                || source.contains("bitset")
                || source.contains("multiset")
                || source.contains("map<")
                || source.contains("set<")
                || source.contains("vector<vector")
        }
        Language::Java => {
            source.contains("BigInteger")
                || source.contains("BigDecimal")
                || source.contains("PriorityQueue")
                || source.contains("HashMap")
                || source.contains("HashSet")
                || source.contains("TreeMap")
                || source.contains("TreeSet")
                || source.contains("ArrayDeque")
        }
        Language::Python => {
            source.contains("defaultdict")
                || source.contains("Counter")
                || source.contains("heapq")
                || source.contains("permutations")
                || source.contains("combinations")
                || source.contains("bisect")
                || source.contains("deque")
                || source.contains("math.comb")
        }
    }
}

// ---------------------------------------------------------------------------
// cyclomatic_complexity
// ---------------------------------------------------------------------------

/// Count decision points: if, for, while, case, &&, ||, ternary.
/// Cyclomatic complexity = 1 + this count (the +1 is applied by the caller).
fn count_decision_points(root: Node, source: &str) -> u32 {
    let bytes = source.as_bytes();
    let mut count = 0u32;
    for_each_node(root, |n| {
        count += match n.kind() {
            // if / for / while are decisions in every language. Note that
            // else-if continuations ARE counted here (each `else if` is its own
            // decision point per McCabe), unlike nesting_depth where they do
            // not add depth. Python `elif_clause`s are separate node kinds that
            // each represent one more decision in the chain.
            "if_statement" | "elif_clause" | "for_statement" | "while_statement" => 1,
            // case labels: C/C++ `case_statement`, Java `switch_label`,
            // Python `case_clause` (all verified).
            "case_statement" | "case_clause" | "switch_label" => 1,
            // ternary: C/C++/Python `conditional_expression`, Java
            // `ternary_expression` (all verified).
            "conditional_expression" | "ternary_expression" => 1,
            // Python boolean operators (and/or).
            "boolean_operator" => 1,
            // C/C++/Java: binary expressions whose operator is && or ||.
            "binary_expression" => {
                let is_bool_op = n
                    .child_by_field_name("operator")
                    .and_then(|op| op.utf8_text(bytes).ok())
                    .map(|op| op == "&&" || op == "||")
                    .unwrap_or(false);
                if is_bool_op {
                    1
                } else {
                    0
                }
            }
            _ => 0,
        };
    });
    count
}

// ---------------------------------------------------------------------------
// is_recursive & recursive_call_count
// ---------------------------------------------------------------------------

/// Detect direct recursion and count self-call sites across function bodies.
fn detect_recursion_with_count(root: Node, lang: Language, source: &str) -> (bool, u32) {
    let count = match lang {
        Language::C | Language::Cpp => detect_recursion_c_like(root, source),
        Language::Java => detect_recursion_java(root, source),
        Language::Python => detect_recursion_python(root, source),
    };
    (count > 0, count)
}

fn detect_recursion_c_like(root: Node, source: &str) -> u32 {
    let bytes = source.as_bytes();
    let mut functions: Vec<(String, Node)> = Vec::new();
    for_each_node(root, |n| {
        if n.kind() == "function_definition" {
            if let Some(name) = c_like_function_name(n, bytes) {
                functions.push((name, n));
            }
        }
    });

    let mut total_calls = 0u32;
    for (name, fn_node) in functions {
        let Some(body) = fn_node.child_by_field_name("body") else {
            continue;
        };
        for_each_node(body, |m| {
            if m.kind() == "call_expression" {
                let callee = m.child_by_field_name("function");
                let is_direct = callee.map(|c| c.kind() == "identifier").unwrap_or(false);
                if is_direct {
                    let callee_text = callee.and_then(|c| c.utf8_text(bytes).ok()).unwrap_or("");
                    if callee_text == name {
                        total_calls += 1;
                    }
                }
            }
        });
    }
    total_calls
}

/// Extract the name of a C/C++ function definition. The name is the base
/// identifier of the `declarator` field of the `function_declarator`, possibly
/// wrapped in pointer/parenthesized declarators.
fn c_like_function_name(fn_node: Node, bytes: &[u8]) -> Option<String> {
    let declarator = fn_node.child_by_field_name("declarator")?;
    if declarator.kind() != "function_declarator" {
        return None;
    }
    let inner = declarator.child_by_field_name("declarator")?;
    let ident = base_identifier(inner)?;
    ident.utf8_text(bytes).ok().map(|s| s.to_string())
}

/// Walk through declarator wrappers down to the base identifier.
fn base_identifier(mut node: Node) -> Option<Node> {
    loop {
        match node.kind() {
            "identifier" | "field_identifier" => return Some(node),
            "pointer_declarator" | "parenthesized_declarator" | "reference_declarator" => {
                node = node.named_child(0)?;
            }
            _ => return None,
        }
    }
}

fn detect_recursion_java(root: Node, source: &str) -> u32 {
    let bytes = source.as_bytes();
    let mut methods: Vec<(String, Node)> = Vec::new();
    for_each_node(root, |n| {
        if n.kind() == "method_declaration" {
            if let Some(name) = n
                .child_by_field_name("name")
                .and_then(|c| c.utf8_text(bytes).ok())
                .map(|s| s.to_string())
            {
                methods.push((name, n));
            }
        }
    });

    let mut total_calls = 0u32;
    for (name, method) in methods {
        let Some(body) = method.child_by_field_name("body") else {
            continue;
        };
        for_each_node(body, |m| {
            if m.kind() == "method_invocation" {
                let invoked = m
                    .child_by_field_name("name")
                    .and_then(|c| c.utf8_text(bytes).ok())
                    .unwrap_or("");
                if invoked == name {
                    let external_receiver = m
                        .child_by_field_name("object")
                        .map(|o| o.utf8_text(bytes).map(|t| t != "this").unwrap_or(true))
                        .unwrap_or(false);
                    if !external_receiver {
                        total_calls += 1;
                    }
                }
            }
        });
    }
    total_calls
}

fn detect_recursion_python(root: Node, source: &str) -> u32 {
    let bytes = source.as_bytes();
    let mut functions: Vec<(String, Node)> = Vec::new();
    for_each_node(root, |n| {
        if n.kind() == "function_definition" {
            if let Some(name) = n
                .child_by_field_name("name")
                .and_then(|c| c.utf8_text(bytes).ok())
                .map(|s| s.to_string())
            {
                functions.push((name, n));
            }
        }
    });

    let mut total_calls = 0u32;
    for (name, fn_node) in functions {
        let Some(body) = fn_node.child_by_field_name("body") else {
            continue;
        };
        for_each_node(body, |m| {
            if m.kind() == "call" {
                let callee = m.child_by_field_name("function");
                let is_direct = callee.map(|c| c.kind() == "identifier").unwrap_or(false);
                if is_direct {
                    let callee_text = callee.and_then(|c| c.utf8_text(bytes).ok()).unwrap_or("");
                    if callee_text == name {
                        total_calls += 1;
                    }
                }
            }
        });
    }
    total_calls
}

// ---------------------------------------------------------------------------
// large_alloc_flag
// ---------------------------------------------------------------------------

/// Aggregated allocation statistics for one source file.
///
/// `large_alloc_flag` is a single boolean against one fixed threshold, and a
/// boolean cannot express *how big* an allocation is. Measured feature
/// importance showed the models ignore it almost entirely — it is literally
/// unused (importance 0.0000) by the specialised C++ model. These aggregates
/// carry the magnitude instead:
///
/// - `size_max`      largest statically-known allocation
/// - `size_total`    sum of every statically-known allocation
/// - `sites`         count of recognised allocation sites (static or not)
/// - `unknown_sites` recognised sites whose size is NOT statically known
///
/// Unit caveat, inherited from the existing detectors and deliberate: a size is
/// in BYTES for the C allocator family (`malloc(n)` is bytes) but in ELEMENT
/// COUNTS for C++ `new T[n]`, Java `new int[n]` and Python `[0] * n`, which
/// cannot be converted without resolving the element type. The fields are
/// therefore named `size_*` rather than `bytes_*` to avoid claiming a unit the
/// extractor does not actually produce.
///
/// `unknown_sites` is the load-bearing one. `new byte[chunks][1 << 20]` and
/// `malloc(n)` have no static size, yet an input-driven allocation is exactly
/// the shape that blows a memory budget — and it is invisible to
/// `large_alloc_flag` by construction.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllocStats {
    pub size_max: u64,
    pub size_total: u64,
    pub sites: u32,
    pub unknown_sites: u32,
}

impl AllocStats {
    /// Record one recognised allocation site; `None` means "size not statically known".
    fn record(&mut self, size: Option<u64>) {
        self.sites = self.sites.saturating_add(1);
        match size {
            Some(s) => {
                self.size_max = self.size_max.max(s);
                self.size_total = self.size_total.saturating_add(s);
            }
            None => self.unknown_sites = self.unknown_sites.saturating_add(1),
        }
    }

    /// The historical boolean: any static allocation above [`LARGE_ALLOC_THRESHOLD`].
    fn any_large(&self) -> bool {
        self.size_max > LARGE_ALLOC_THRESHOLD
    }
}

fn detect_large_allocation(root: Node, lang: Language, source: &str) -> AllocStats {
    match lang {
        Language::C => alloc_stats_c(root, source),
        Language::Cpp => alloc_stats_cpp(root, source),
        Language::Java => alloc_stats_java(root, source),
        Language::Python => alloc_stats_python(root, source),
    }
}

/// Is this C function name one of the allocators we recognise?
///
/// This matters for `sites`/`unknown_sites`: without it every ordinary call
/// (`printf`, `scanf`, ...) would be counted as an allocation site with an
/// unknown size, which would make the feature meaningless noise.
fn is_c_allocator(name: &str) -> bool {
    matches!(name, "malloc" | "calloc" | "realloc" | "aligned_alloc")
}

/// Read the `index`-th argument of an `argument_list` and evaluate it as a
/// constant. Returns `None` when the argument is not a compile-time constant
/// (e.g. a variable name).
fn arg_const(args: Node, index: usize, bytes: &[u8]) -> Option<u64> {
    let arg = args.named_child(index)?;
    const_eval(arg, bytes)
}

fn alloc_stats_c(root: Node, source: &str) -> AllocStats {
    let bytes = source.as_bytes();
    let mut stats = AllocStats::default();
    for_each_node(root, |n| {
        if n.kind() != "call_expression" {
            return;
        }
        let callee = match n.child_by_field_name("function") {
            Some(c) if c.kind() == "identifier" => c,
            _ => return,
        };
        let name = callee.utf8_text(bytes).unwrap_or("");
        if !is_c_allocator(name) {
            return;
        }
        let args = match n.child_by_field_name("arguments") {
            Some(a) => a,
            None => return,
        };
        // `None` here now means "recognised allocator, size not statically known"
        // (e.g. `malloc(n)`), which is exactly what `unknown_sites` counts.
        stats.record(c_alloc_size(name, args, bytes));
    });
    stats
}

/// Total allocation size for a C allocator call, when fully static:
/// - malloc / realloc : first argument (bytes)
/// - calloc           : count * element size (both must be literal)
/// - aligned_alloc    : second argument (bytes; first is alignment)
fn c_alloc_size(name: &str, args: Node, bytes: &[u8]) -> Option<u64> {
    match name {
        "malloc" | "realloc" => arg_const(args, 0, bytes),
        "calloc" => {
            let count = arg_const(args, 0, bytes)?;
            let elem = arg_const(args, 1, bytes)?;
            count.checked_mul(elem)
        }
        "aligned_alloc" => arg_const(args, 1, bytes),
        _ => None,
    }
}

fn alloc_stats_cpp(root: Node, source: &str) -> AllocStats {
    let bytes = source.as_bytes();
    let mut stats = AllocStats::default();
    for_each_node(root, |n| {
        match n.kind() {
            // v.reserve(n) / v.resize(n)
            "call_expression" => {
                let callee = match n.child_by_field_name("function") {
                    Some(c) if c.kind() == "field_expression" => c,
                    _ => return,
                };
                // field_expression children: object, '.', field_identifier
                let member = callee.named_child(1);
                let Some(member) = member else {
                    return;
                };
                if member.kind() != "field_identifier" {
                    return;
                }
                let member_name = member.utf8_text(bytes).unwrap_or("");
                if member_name != "reserve" && member_name != "resize" {
                    return;
                }
                let args = match n.child_by_field_name("arguments") {
                    Some(a) => a,
                    None => return,
                };
                stats.record(arg_const(args, 0, bytes));
            }
            // new T[n] — the size expression is the named child inside brackets
            "new_expression" => {
                let mut cursor = n.walk();
                for child in n.children(&mut cursor) {
                    if child.kind() == "new_declarator" {
                        if let Some(size_node) = child.named_child(0) {
                            // non-constant size (e.g. `new byte[chunks]`) -> unknown site
                            stats.record(const_eval(size_node, bytes));
                        }
                    }
                }
            }
            _ => {}
        }
    });
    stats
}

fn alloc_stats_java(root: Node, source: &str) -> AllocStats {
    let bytes = source.as_bytes();
    let mut stats = AllocStats::default();
    for_each_node(root, |n| {
        match n.kind() {
            // new int[10000], and each dimension of `new byte[chunks][1 << 20]`
            "array_creation_expression" => {
                let mut cursor = n.walk();
                for child in n.children(&mut cursor) {
                    if child.kind() == "dimensions_expr" {
                        if let Some(size_node) = child.named_child(0) {
                            // a variable outer dimension -> unknown site, which is
                            // precisely the shape `large_alloc_flag` cannot see
                            stats.record(const_eval(size_node, bytes));
                        }
                    }
                }
            }
            // new ArrayList<>(5000) / new HashMap<>(n) / ...
            "object_creation_expression" => {
                let Some(type_name) = object_creation_type(n, bytes) else {
                    return;
                };
                if !is_capacity_collection(&type_name) {
                    return;
                }
                let args = match n.child_by_field_name("arguments") {
                    Some(a) => a,
                    None => return,
                };
                // first constructor argument is the initial capacity
                stats.record(arg_const(args, 0, bytes));
            }
            _ => {}
        }
    });
    stats
}

/// The created type name inside an `object_creation_expression` (e.g. "ArrayList").
fn object_creation_type(node: Node, bytes: &[u8]) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "type_identifier" => return child.utf8_text(bytes).ok().map(|s| s.to_string()),
            "generic_type" => {
                let mut c = child.walk();
                for gc in child.children(&mut c) {
                    if gc.kind() == "type_identifier" {
                        return gc.utf8_text(bytes).ok().map(|s| s.to_string());
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// Standard-library collection types whose constructor takes an initial capacity.
fn is_capacity_collection(name: &str) -> bool {
    matches!(
        name,
        "ArrayList"
            | "Vector"
            | "HashMap"
            | "HashSet"
            | "LinkedHashMap"
            | "LinkedHashSet"
            | "TreeMap"
            | "TreeSet"
            | "ArrayDeque"
            | "PriorityQueue"
            | "StringBuilder"
            | "StringBuffer"
            | "ArrayBlockingQueue"
            | "LinkedBlockingQueue"
    )
}

fn alloc_stats_python(root: Node, source: &str) -> AllocStats {
    let bytes = source.as_bytes();
    let mut stats = AllocStats::default();
    for_each_node(root, |n| {
        match n.kind() {
            // [0] * 1000000  (container repetition with a literal count)
            "binary_operator" => {
                let op_is_star = n
                    .child_by_field_name("operator")
                    .and_then(|o| o.utf8_text(bytes).ok())
                    .map(|op| op == "*")
                    .unwrap_or(false);
                if !op_is_star {
                    return;
                }
                let left = n.child_by_field_name("left");
                let left_is_container = left
                    .map(|l| {
                        matches!(l.kind(), "list" | "tuple" | "set" | "dictionary" | "string")
                    })
                    .unwrap_or(false);
                if !left_is_container {
                    return;
                }
                if let Some(right) = n.child_by_field_name("right") {
                    // `[0] * n` with a variable n -> unknown site
                    stats.record(const_eval(right, bytes));
                }
            }
            // bytearray(2000000)
            "call" => {
                let callee = match n.child_by_field_name("function") {
                    Some(c) if c.kind() == "identifier" => c,
                    _ => return,
                };
                let name = callee.utf8_text(bytes).unwrap_or("");
                if name != "bytearray" {
                    return;
                }
                let args = match n.child_by_field_name("arguments") {
                    Some(a) => a,
                    None => return,
                };
                stats.record(arg_const(args, 0, bytes));
            }
            _ => {}
        }
    });
    stats
}

// ---------------------------------------------------------------------------
// constant evaluation (literal-only arithmetic)
// ---------------------------------------------------------------------------

/// Statically evaluate a numeric expression built ONLY from number literals and
/// arithmetic operators. Returns `None` when the expression is not a
/// compile-time constant — e.g. `malloc(n * 4)` — which is intentional:
/// allocations sized by a variable are undetectable statically and are never
/// flagged (known, accepted limitation).
fn const_eval(node: Node, bytes: &[u8]) -> Option<u64> {
    match node.kind() {
        // number literals per language (verified node names)
        "number_literal" | "decimal_integer_literal" | "integer" | "decimal_float_literal"
        | "hex_integer_literal" => parse_num(node.utf8_text(bytes).ok()?),
        "parenthesized_expression" => const_eval(node.named_child(0)?, bytes),
        "binary_expression" | "binary_operator" => {
            let left = node.child_by_field_name("left")?;
            let right = node.child_by_field_name("right")?;
            let op = node.child_by_field_name("operator")?;
            let op_text = op.utf8_text(bytes).ok()?;
            let l = const_eval(left, bytes)?;
            let r = const_eval(right, bytes)?;
            match op_text {
                "*" => l.checked_mul(r),
                "+" => l.checked_add(r),
                "-" => l.checked_sub(r),
                "<<" => l.checked_shl(r as u32),
                ">>" => l.checked_shr(r as u32),
                "/" => (r != 0).then(|| l / r),
                "%" => (r != 0).then(|| l % r),
                "&" => Some(l & r),
                "|" => Some(l | r),
                "^" => Some(l ^ r),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Parse a decimal or hex integer literal, ignoring C/C++/Java numeric suffixes
/// (`L`, `U`, `F`, ...) and Python underscore separators (`1_000_000`).
fn parse_num(text: &str) -> Option<u64> {
    let t = text
        .trim()
        .trim_end_matches(|c: char| matches!(c, 'u' | 'U' | 'l' | 'L' | 'f' | 'F'));
    if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        u64::from_str_radix(&hex.replace('_', ""), 16).ok()
    } else {
        t.replace('_', "").parse::<u64>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feats(src: &str, lang: Language) -> Features {
        compute_features(src, lang)
    }

    // ---- C -----------------------------------------------------------------

    #[test]
    fn c_else_if_chain_does_not_inflate_depth() {
        let f = feats(
            r#"
            int f(int x) {
                if (x < 0) return -1;
                else if (x == 0) return 0;
                else if (x < 100) return 1;
                else return 2;
            }
            "#,
            Language::C,
        );
        assert_eq!(f.nesting_depth, 1, "else-if chain must not add depth");
        assert_eq!(f.cyclomatic_complexity, 4, "3 ifs => 1 + 3");
        assert!(!f.is_recursive);
        assert!(!f.large_alloc_flag);
        assert!(!f.parse_error_flag);
    }

    #[test]
    fn c_nested_for_while_if() {
        let f = feats(
            r#"
            int f(int a, int b) {
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
            Language::C,
        );
        assert_eq!(f.nesting_depth, 3);
        assert_eq!(f.cyclomatic_complexity, 4, "for + while + if => 1 + 3");
    }

    #[test]
    fn alloc_stats_capture_dynamic_allocation() {
        // `malloc(n)` is the case the boolean feature structurally cannot see: the
        // size is a runtime value, so `const_eval` fails and `large_alloc_flag`
        // stays false even though the allocation may be enormous.
        // `alloc_unknown_sites` is what carries that signal.
        let f = feats(
            r#"#include <stdlib.h>
               int main(void) { int n; scanf("%d", &n); char *p = malloc(n); return p != 0; }"#,
            Language::C,
        );
        assert!(!f.large_alloc_flag, "no statically-known large size");
        assert_eq!(f.alloc_sites, 1, "one recognised allocation site");
        assert_eq!(f.alloc_unknown_sites, 1, "sized by a variable");
        assert_eq!(f.alloc_size_max, 0);
    }

    #[test]
    fn alloc_stats_capture_magnitude_not_just_a_threshold() {
        let f = feats(
            r#"#include <stdlib.h>
               int main(void) { char *a = malloc(5 * 1024 * 1024); char *b = malloc(2048); return a != 0 && b != 0; }"#,
            Language::C,
        );
        assert!(f.large_alloc_flag, "5 MiB exceeds the 1_000_000 threshold");
        assert_eq!(f.alloc_sites, 2);
        assert_eq!(f.alloc_unknown_sites, 0);
        assert_eq!(
            f.alloc_size_max,
            5 * 1024 * 1024,
            "the boolean discards this magnitude"
        );
        assert_eq!(f.alloc_size_total, 5 * 1024 * 1024 + 2048);
    }

    #[test]
    fn alloc_stats_java_multidim_counts_each_dimension() {
        // The shape behind the misrouted heavy Java submission: a variable outer
        // dimension times a constant inner one. The constant inner dimension is
        // what makes the boolean fire; the variable outer dimension is visible
        // only through `alloc_unknown_sites`.
        let f = feats(
            "import java.util.*; class S { void m(int chunks) { byte[][] a = new byte[chunks][1024 * 1024]; } }",
            Language::Java,
        );
        assert_eq!(f.alloc_sites, 2, "two dimensions = two sites");
        assert_eq!(f.alloc_unknown_sites, 1, "the outer dimension is a variable");
        assert_eq!(f.alloc_size_max, 1024 * 1024);
        assert!(f.large_alloc_flag, "inner dimension alone exceeds the threshold");
    }

    #[test]
    fn alloc_stats_ignore_non_allocating_calls() {
        // Guards the `is_c_allocator` filter. Without it every ordinary call would
        // be counted as an unknown-size allocation site, and the feature would be
        // noise dressed up as signal.
        let f = feats(
            r#"#include <stdio.h>
               int main(void) { printf("hi\n"); return 0; }"#,
            Language::C,
        );
        assert_eq!(f.alloc_sites, 0);
        assert_eq!(f.alloc_unknown_sites, 0);
        assert_eq!(f.alloc_size_total, 0);
    }

    #[test]
    fn c_recursive_with_large_literal_alloc() {
        let f = feats(
            r#"
            #include <stdlib.h>
            long fact(long n) {
                if (n <= 1) return 1;
                char *tmp = malloc(5 * 1024 * 1024);
                return n * fact(n - 1);
            }
            "#,
            Language::C,
        );
        assert_eq!(f.nesting_depth, 1);
        assert_eq!(f.cyclomatic_complexity, 2, "1 if => 1 + 1");
        assert!(f.is_recursive, "fact() calls fact()");
        assert!(f.large_alloc_flag, "malloc(5*1024*1024) = 5MB > threshold");
    }

    #[test]
    fn c_variable_sized_alloc_not_flagged() {
        let f = feats(
            r#"
            #include <stdlib.h>
            void f(int n) {
                int *a = malloc(n * 4);
                int *b = malloc(1000);
            }
            "#,
            Language::C,
        );
        assert!(
            !f.large_alloc_flag,
            "variable size not statically detectable; 1000 < threshold"
        );
    }

    #[test]
    fn c_boolean_ops_and_ternary_count_as_decisions() {
        let f = feats(
            r#"
            int f(int a, int b, int c) {
                int r = (a && b) || (a > c ? 1 : 2);
                return r;
            }
            "#,
            Language::C,
        );
        assert_eq!(f.cyclomatic_complexity, 4, "&& + || + ternary => 1 + 3");
        assert_eq!(f.nesting_depth, 0, "no if/for/while");
    }

    #[test]
    fn c_switch_cases_count_as_decisions() {
        let f = feats(
            r#"
            int f(int x) {
                switch (x) {
                    case 1: return 10;
                    case 2: return 20;
                    default: return 0;
                }
            }
            "#,
            Language::C,
        );
        assert_eq!(f.cyclomatic_complexity, 4, "3 case labels => 1 + 3");
        assert_eq!(f.nesting_depth, 0);
    }

    // ---- C++ ---------------------------------------------------------------

    #[test]
    fn cpp_else_if_chain() {
        let f = feats(
            r#"
            int f(int x) {
                if (x < 0) return -1;
                else if (x == 0) return 0;
                else return 2;
            }
            "#,
            Language::Cpp,
        );
        assert_eq!(f.nesting_depth, 1, "else-if chain must not add depth");
        assert_eq!(f.cyclomatic_complexity, 3, "2 ifs => 1 + 2");
    }

    #[test]
    fn cpp_reserve_and_new_array_large() {
        let f = feats(
            r#"
            #include <vector>
            int main() {
                std::vector<int> v;
                v.reserve(2000000);
                int *arr = new int[5000];
                int *big = new int[3000000];
                int m = 5;
                int *var = new int[m];
                return 0;
            }
            "#,
            Language::Cpp,
        );
        assert!(f.large_alloc_flag, "reserve(2M) and new int[3M] both large");
    }

    #[test]
    fn cpp_in_class_method_bare_self_call_is_detected() {
        // In-class C++ methods are named with a `field_identifier` in the
        // tree-sitter-cpp tree, which `base_identifier` previously ignored —
        // so a method calling ITSELF by bare name was never flagged. With the
        // fix, the method name is extracted and the bare self-call is seen.
        let f = feats(
            r#"
            class Node {
            public:
                void rec(int n) {
                    if (n <= 0) return;
                    rec(n - 1);
                }
            };
            "#,
            Language::Cpp,
        );
        assert!(f.is_recursive, "bare self-call inside an in-class method");
        assert_eq!(f.nesting_depth, 1);
        assert_eq!(f.cyclomatic_complexity, 2);
    }

    // ---- Java --------------------------------------------------------------

    #[test]
    fn java_else_if_chain() {
        let f = feats(
            r#"
            class F {
                int f(int x) {
                    if (x < 0) return -1;
                    else if (x == 0) return 0;
                    else if (x < 100) return 1;
                    else return 2;
                }
            }
            "#,
            Language::Java,
        );
        assert_eq!(f.nesting_depth, 1, "else-if chain must not add depth");
        assert_eq!(f.cyclomatic_complexity, 4, "3 ifs => 1 + 3");
        assert!(!f.is_recursive);
    }

    #[test]
    fn java_recursive_with_this_and_large_collection() {
        let f = feats(
            r#"
            import java.util.*;
            class F {
                long fact(int n) {
                    if (n <= 1) return 1;
                    ArrayList<Integer> list = new ArrayList<>(2000000);
                    return n * this.fact(n - 1);
                }
            }
            "#,
            Language::Java,
        );
        assert_eq!(f.nesting_depth, 1);
        assert_eq!(f.cyclomatic_complexity, 2);
        assert!(f.is_recursive, "this.fact(...) is a direct self-call");
        assert!(f.large_alloc_flag, "ArrayList<>(2M) is large");
    }

    #[test]
    fn java_array_alloc_literal_but_variable_not() {
        let f = feats(
            r#"
            class F {
                void g(int n) {
                    int[] a = new int[10000];
                    int[] b = new int[n];
                    int[] c = new int[5000000];
                }
            }
            "#,
            Language::Java,
        );
        assert!(f.large_alloc_flag, "new int[5M] is large");
    }

    #[test]
    fn java_object_call_is_not_self_recursion() {
        let f = feats(
            r#"
            class F {
                int fact(int n) {
                    if (n <= 1) return 1;
                    F other = new F();
                    return other.fact(n - 1);
                }
            }
            "#,
            Language::Java,
        );
        assert!(
            !f.is_recursive,
            "other.fact() has an external receiver => not self-recursion"
        );
    }

    #[test]
    fn java_boolean_and_ternary_count_as_decisions() {
        let f = feats(
            r#"
            class F {
                int f(int a, int b, int c) {
                    int r = (a && b) || (a > c ? 1 : 2);
                    return r;
                }
            }
            "#,
            Language::Java,
        );
        assert_eq!(f.cyclomatic_complexity, 4, "&& + || + ternary => 1 + 3");
    }

    #[test]
    fn java_switch_cases_count_as_decisions() {
        let f = feats(
            r#"
            class F {
                int f(int x) {
                    switch (x) {
                        case 1: return 10;
                        case 2: return 20;
                        default: return 0;
                    }
                }
            }
            "#,
            Language::Java,
        );
        assert_eq!(f.cyclomatic_complexity, 4, "3 switch labels => 1 + 3");
        assert_eq!(f.nesting_depth, 0);
    }

    // ---- Python ------------------------------------------------------------

    #[test]
    fn py_elif_chain_is_flat() {
        let f = feats(
            r#"
            def f(x):
                if x < 0:
                    return -1
                elif x == 0:
                    return 0
                elif x < 100:
                    return 1
                else:
                    return 2
            "#,
            Language::Python,
        );
        // KEY edge case: Python elif clauses are flat siblings of the same
        // if_statement, so the whole chain stays at depth 1.
        assert_eq!(f.nesting_depth, 1, "elif must not add depth");
        assert_eq!(f.cyclomatic_complexity, 4, "if + 2 elif => 1 + 3");
        assert!(!f.is_recursive);
        assert!(!f.large_alloc_flag);
    }

    #[test]
    fn py_nested_for_while_if() {
        let f = feats(
            r#"
            def f(a, b):
                for i in range(a):
                    while b > 0:
                        if i % 2 == 0:
                            b -= 1
                return b
            "#,
            Language::Python,
        );
        assert_eq!(f.nesting_depth, 3);
        assert_eq!(f.cyclomatic_complexity, 4);
    }

    #[test]
    fn py_recursive_with_large_list_repetition() {
        let f = feats(
            r#"
            def fact(n):
                if n <= 1:
                    return 1
                a = [0] * 5000000
                return n * fact(n - 1)
            "#,
            Language::Python,
        );
        assert_eq!(f.nesting_depth, 1);
        assert_eq!(f.cyclomatic_complexity, 2);
        assert!(f.is_recursive, "fact() calls fact()");
        assert!(f.large_alloc_flag, "[0] * 5M is large");
    }

    #[test]
    fn py_variable_sized_alloc_not_flagged() {
        let f = feats(
            r#"
            def f(n):
                a = [0] * n
                b = bytearray(1000)
                return a
            "#,
            Language::Python,
        );
        assert!(
            !f.large_alloc_flag,
            "variable-size repetition undetectable; bytearray(1000) < threshold"
        );
    }

    #[test]
    fn py_mutual_recursion_is_out_of_scope() {
        let f = feats(
            r#"
            def is_even(n):
                if n == 0:
                    return True
                return is_odd(n - 1)

            def is_odd(n):
                if n == 0:
                    return False
                return is_even(n - 1)
            "#,
            Language::Python,
        );
        assert!(
            !f.is_recursive,
            "mutual recursion is intentionally out of scope; each fn calls the other, not itself"
        );
    }

    #[test]
    fn py_boolean_and_ternary_count_as_decisions() {
        let f = feats(
            r#"
            def f(a, b, c):
                r = (a and b) or (c if a else b)
                return r
            "#,
            Language::Python,
        );
        assert_eq!(f.cyclomatic_complexity, 4, "and + or + ternary => 1 + 3");
    }

    #[test]
    fn py_match_cases_count_as_decisions() {
        let f = feats(
            r#"
            def f(x):
                match x:
                    case 1:
                        return 10
                    case 2:
                        return 20
                    case _:
                        return 0
            "#,
            Language::Python,
        );
        assert_eq!(f.cyclomatic_complexity, 4, "3 case clauses => 1 + 3");
        assert_eq!(f.nesting_depth, 0);
    }

    // ---- parse errors ------------------------------------------------------

    #[test]
    fn parse_error_is_flagged_but_features_still_computed() {
        let f = feats("def f(:\n    x = (\n    return x", Language::Python);
        assert!(f.parse_error_flag, "broken Python must set the flag");
        assert!(f.error_node_count >= 1);
        // best-effort features are still produced (here: nothing decision-like)
        assert_eq!(f.cyclomatic_complexity, 1);
    }

    #[test]
    fn clean_c_file_has_no_parse_error() {
        let f = feats("int main(void) { return 0; }", Language::C);
        assert!(!f.parse_error_flag);
        assert_eq!(f.error_node_count, 0);
    }

    // ---- additional edge cases (documented metric definitions) -------------

    #[test]
    fn c_do_while_adds_no_depth_or_complexity() {
        // `do { } while (...)` parses as `do_statement`, which is not in the
        // counted set (only if/for/while are). Lock in that behavior.
        let f = feats(
            r#"
            int f(int x) {
                do { x--; } while (x > 0);
                return x;
            }
            "#,
            Language::C,
        );
        assert_eq!(f.nesting_depth, 0, "do/while is not a counted construct");
        assert_eq!(f.cyclomatic_complexity, 1);
    }

    #[test]
    fn c_switch_inside_loop_adds_complexity_not_depth() {
        let f = feats(
            r#"
            int f(int n) {
                for (int i = 0; i < n; i++) {
                    switch (i) {
                        case 0: return 1;
                        case 1: return 2;
                        default: break;
                    }
                }
                return 0;
            }
            "#,
            Language::C,
        );
        assert_eq!(f.nesting_depth, 1, "switch is not a nesting construct");
        assert_eq!(f.cyclomatic_complexity, 1 + 1 + 3, "for + 3 case labels");
    }

    #[test]
    fn cpp_range_for_is_not_counted_as_nesting() {
        // Range-based for parses as `range_based_for_statement`, which is not
        // in the counted node set. This documents current behavior.
        let f = feats(
            r#"
            #include <vector>
            int f() {
                std::vector<int> v;
                int total = 0;
                for (int x : v) {
                    total += x;
                }
                return total;
            }
            "#,
            Language::Cpp,
        );
        assert_eq!(f.nesting_depth, 0);
        assert_eq!(f.cyclomatic_complexity, 1);
    }

    #[test]
    fn java_deep_nesting() {
        let f = feats(
            r#"
            class F {
                int f(int a, int b, int c) {
                    for (int i = 0; i < a; i++) {
                        for (int j = 0; j < b; j++) {
                            if (i + j < c) {
                                c--;
                            }
                        }
                    }
                    return c;
                }
            }
            "#,
            Language::Java,
        );
        assert_eq!(f.nesting_depth, 3);
        assert_eq!(f.cyclomatic_complexity, 4, "for + for + if => 1 + 3");
    }

    #[test]
    fn py_try_except_adds_no_depth() {
        let f = feats(
            r#"
            def f(x):
                try:
                    if x > 0:
                        return 1
                except Exception:
                    return 2
            "#,
            Language::Python,
        );
        assert_eq!(f.nesting_depth, 1, "try/except is not counted");
        assert_eq!(f.cyclomatic_complexity, 2, "only the inner if is a decision");
    }

    #[test]
    fn c_variable_sized_new_array_not_flagged() {
        let f = feats(
            r#"
            #include <stdlib.h>
            int main() {
                int n = 5;
                int *a = malloc(2000000 * sizeof(int));
                return 0;
            }
            "#,
            Language::C,
        );
        // `2000000 * sizeof(int)` is not fully static (sizeof is not handled),
        // so it is never flagged. Documented limitation.
        assert!(!f.large_alloc_flag);
    }
}
