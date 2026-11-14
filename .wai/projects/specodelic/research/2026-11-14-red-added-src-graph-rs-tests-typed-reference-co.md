---
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-cbh-kernel-grammar-gate-placement-derive-typed-reference-columns-from-schema, pipeline-step:red]
---

RED: added src/graph.rs::tests::typed_reference_columns_derive_from_schema calling typed_reference_columns() and NON_GRAPH_COLUMNS (neither exists). Command cargo test --lib graph::tests::typed_reference_columns_derive_from_schema fails with E0425 (unresolved names), as expected for the missing derivation. No production behavior changed. Next green: derive the set from acset::schema::canonical() minus the explicit {from,to,uses} exemptions.
