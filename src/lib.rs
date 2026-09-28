//! specodelic — the Specodelic format library shared by the `ddl` CLI.
//!
//! Purpose: model the four-layer markdown spec format (Intent /
//! Constraints / Model / Properties) so the CLI commands and tests all
//! speak the same parse. Responsibilities: expose `spec` (the parser),
//! `ears` (the statement grammar), `lint` (the invariants), and `graph`
//! (the derived reference graph). Rationale: keep the domain core in a
//! library so the binary stays thin and the corpus can be dogfooded from
//! integration tests.

pub mod ears;
pub mod graph;
pub mod lint;
pub mod spec;
