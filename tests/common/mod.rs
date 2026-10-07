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
