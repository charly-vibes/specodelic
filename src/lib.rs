//! specodelic — the Specodelic format library shared by the `specodelic` CLI (binary `specodelic`, alias `spk`).
//!
//! Purpose: model the four-layer markdown spec format (Intent /
//! Constraints / Model / Properties) so the CLI commands and tests all
//! speak the same parse. Responsibilities: expose `spec` (the parser),
//! `ears` (the statement grammar), `lint` (the invariants), `graph` (the
//! derived reference graph), and `compile` (the Constraints→TOML /
//! Model→IR / Properties→proptest! functor). Rationale: keep the domain core in a
//! library so the binary stays thin and the corpus can be dogfooded from
//! integration tests.

pub mod compile;
pub mod ears;
pub mod graph;
pub mod lint;
pub mod spec;
