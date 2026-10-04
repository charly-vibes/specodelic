//! The acset core — the format's schema and one way to walk it (openspec
//! change `add-acset-core`, beads specodelic-84i).
//!
//! Purpose: hold the format's fixed finite schema (five objects, typed
//! reference morphisms) as one data value, build parsed corpora into typed
//! instances whose dangling references are representable values, and answer
//! reachability questions through a single closure primitive shared by
//! graph, merge, and refactor. Responsibilities: internal-only — no new
//! CLI surface; every migration onto these modules is gated by parity
//! against the existing walks (design.md decision 2). Rationale: the
//! Reference Typing table as per-field match arms, private adjacency maps
//! in merge.rs, and duplicated edge loops in graph.rs are three renderings
//! of one schema — one data value retires all three.

pub mod doc;
pub mod instance;
pub mod schema;

/// The typed instance over a parsed corpus (tasks 3.x). A bare skeleton so
/// the 1.2 parity property compiles and fails at runtime (todo!-panic red,
/// the documented 1.2 convention) while the snapshot oracle (task 1.1)
/// runs and pins `graph::build`'s bytes.
pub struct Acset;

impl Acset {
    /// The corpus's edge set, derived from the instance — the parity
    /// property compares it, edge for edge, with `graph::build`'s.
    pub fn edges(&self) -> Vec<crate::graph::Edge> {
        todo!("instance builder (tasks 3.2/3.3)")
    }
}
