//! specodelic — the Specodelic format library shared by the `specodelic` CLI (binary `specodelic`, alias `spk`).
//!
//! Purpose: model the four-layer markdown spec format (Intent /
//! Constraints / Model / Properties) so the CLI commands and tests all
//! speak the same parse. Responsibilities: expose `spec` (the parser),
//! `ears` (the statement grammar), `lint` (the invariants), `graph` (the
//! derived reference graph), `compile` (the Constraints→TOML /
//! Model→IR / Properties→proptest! functor), and `guide` (the closed
//! value sets + format revision + embedded primer). Rationale: keep the domain core in a
//! library so the binary stays thin and the corpus can be dogfooded from
//! integration tests.

pub mod archive_companion;
pub mod blocks;
pub mod checklist;
pub mod compile;
pub mod doctor;
pub mod ears;
pub mod graph;
pub mod guide;
pub mod hooks;
pub mod human;
pub mod lint;
pub mod merge;
pub mod migrate;
pub mod model_check;
pub mod orchestrate;
pub mod refactor;
pub mod rename;
pub mod spec;
pub mod update_notice;
pub mod verify;
