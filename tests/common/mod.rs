//! Shared integration-test helpers (add-min-expr-kernel phase 4,
//! specodelic-36n tasks.md §4.3): the kernel agreement fixture corpus
//! lives in `kernel_fixtures` so the rust property and l8l's future py
//! parity tests consume the exact same corpus.
//!
//! Each test target compiles this module into its own crate, so the
//! dead-code lint fires on corpus items only some targets consume —
//! that partial consumption is the module's design, not dead code
//! (clippy 1.99 baseline drift, same class as commit f75041d).
#[allow(dead_code)]
pub mod kernel_corpora;
#[allow(dead_code)]
pub mod kernel_fixtures;

/// The corpus-backed capability dirs — the files that were the specs/
/// corpus before the single-tree migration (specodelic-jf1). compile and
/// model-check are per-file gates that refuse stateless specs, so the
/// dogfood corpus gates target exactly this set (mirrors the justfile's
/// CORPUS_DIRS; kept in sync by the model-check-specs gate).
#[allow(dead_code)]
pub fn corpus_dirs() -> Vec<String> {
    [
        "specodelic",
        "kinds",
        "packs",
        "graph",
        "compile",
        "model_check",
        "verify",
        "rename",
        "merge",
        "refactor",
        "orchestrate",
        "errors",
        "linter-frontmatter",
        "linter-schema_shape",
        "linter-ears_syntax",
        "linter-coverage",
        "linter-graph_shape",
        "linter-model_shape",
        "linter-referential_integrity",
        "linter-external_completeness",
        "linter-failure_shape",
        "linter-observability",
    ]
    .iter()
    .map(|d| format!("openspec/specs/{d}"))
    .collect()
}
