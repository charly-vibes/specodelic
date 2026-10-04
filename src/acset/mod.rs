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
pub mod query;
pub mod schema;

use self::instance::Instance;
use crate::graph::Edge;
use crate::spec::Spec;

/// The typed instance over a parsed corpus (tasks 3.x): the acset — one
/// interned id-set per schema object, one partial morphism vector per
/// Reference Typing row. The parity property (`adapter_graph_equivalent`,
/// task 3.3) gates it edge for edge against `graph::build`; the old path
/// stays authoritative until parity holds over the whole corpus.
pub struct Acset {
    instance: Instance,
}

impl Acset {
    /// Build the acset from a corpus of parsed specs — the constructor the
    /// parity property exercises (`edges(from_specs(c)) ==
    /// graph::build(c).edges`).
    pub fn from_specs(specs: &[Spec]) -> Self {
        Self {
            instance: Instance::from_specs(specs),
        }
    }

    /// The corpus's edge set, derived from the instance — the parity
    /// property compares it, edge for edge, with `graph::build`'s.
    pub fn edges(&self) -> Vec<Edge> {
        self.instance.edges()
    }
}
