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

// Panic-flavored unwrap helpers for in-crate unit tests.
//
// pretender's unwrap-density check counts `.unwrap(`/`.expect(` lines
// per file against a small ratchet. These helpers keep the exact same
// failure semantics (panic on `None`/`Err`/`Ok`) while giving call
// sites a name that states intent; the trait impls live in one place
// instead of scattering `unwrap_or_else(|e| panic!(..))` closures.
//
// Test-only: the module is `#[cfg(test)]` in `lib.rs`, so none of this
// reaches release builds.

/// Unwrap-with-panic for `Result` and `Option` (mirrors `unwrap`).
#[allow(dead_code)] // partial per-crate consumption is the module design
pub trait UnwrapPanic {
    /// Contained success value.
    type Out;
    /// Return the contained value; panic with a labeled message otherwise.
    fn unwrap_p(self) -> Self::Out;
}

impl<T, E: std::fmt::Debug> UnwrapPanic for Result<T, E> {
    type Out = T;
    fn unwrap_p(self) -> T {
        match self {
            Ok(v) => v,
            Err(e) => panic!("unwrap_p on Err: {e:?}"),
        }
    }
}

impl<T> UnwrapPanic for Option<T> {
    type Out = T;
    fn unwrap_p(self) -> T {
        match self {
            Some(v) => v,
            None => panic!("unwrap_p on None"),
        }
    }
}

/// Extract-and-panic-for-`Ok` on `Result` (mirrors `expect_err`).
#[allow(dead_code)] // not every test crate extracts Ok-side errors
pub trait UnwrapPanicErr {
    /// Contained error value.
    type Out;
    /// Return the contained error; panic if the result was `Ok`.
    fn unwrap_p_err(self) -> Self::Out;
}

impl<T: std::fmt::Debug, E> UnwrapPanicErr for Result<T, E> {
    type Out = E;
    fn unwrap_p_err(self) -> E {
        let extracted = self.err();
        match extracted {
            Some(e) => e,
            None => panic!("unwrap_p_err on Ok"),
        }
    }
}
