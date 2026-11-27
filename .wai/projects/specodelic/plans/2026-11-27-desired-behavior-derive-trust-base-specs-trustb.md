---
tags: [pipeline-run:tdd-ro5-2026-11-27-trust-ledger-derivation-slices-1-3, pipeline-step:plan]
---

Desired behavior: derive_trust_base(specs)->TrustBase{schema_version 1, unverified_claims, assumed}; kinds prose_only_invariant(=claim_classification().unchecked), foreign_extension_point(satisfies link -> foreign extension_point constraint), advisory_constraint(kind cell advisory), prose_only_guard(transition guard citing no evaluable constraint/state per corpus ModelIr sets). Sorted (kind,source,id); arrays always serialized. Out of scope: stored-report gates, human view, format changes (slices 4-5). Tests (red first) in src/verify.rs tests mod: partition+disjointness, unchecked-set agreement, foreign ext point, advisory, run-limits excluded, corpus-scoped source file, guard citing evaluable/state absent, prose guard present, sort order, empty serialized. Narrow: cargo test --lib verify::tests::trust. Full: cargo test --lib. Files: src/verify.rs only (+tasks.md checkboxes).
