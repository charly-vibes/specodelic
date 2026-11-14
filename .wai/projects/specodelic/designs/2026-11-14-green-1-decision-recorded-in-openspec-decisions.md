---
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-cbh-kernel-grammar-gate-placement-derive-typed-reference-columns-from-schema, pipeline-step:green]
---

GREEN: (1) decision recorded in openspec/decisions/2026-11-14-kernel-grammar-gate-placement.md — compile is the kernel-grammar gate, lint stays grammar-agnostic; kernel.md Notes + openspec/specs/AGENTS.md carry the contributor guidance. (2) src/graph.rs replaces the hardcoded 7-entry TYPED_REFERENCE_COLUMNS const with LazyLock<BTreeSet> derived from acset::schema::canonical().morphisms minus NON_GRAPH_COLUMNS [from,to,uses] (documented rationale); typed_reference_columns() is the accessor; resolve_link uses it. Added gate test typed_reference_columns_derive_from_schema: every canonical column is extracted or exempt, and the derived set is exactly the previous seven. Verified: cargo test --lib graph 20 passed; cargo test --test acset_parity 18 passed; graph over the corpus 1765 edges, 0 dangling, 0 typing violations (unchanged).
