---
tags: [pipeline-run:tdd-ro5-2026-11-27-trust-ledger-derivation-slices-1-3, pipeline-step:green]
---

RED (slice 4 add-verification-trust-ledger): red tests appended — tests/cli/model_check.rs (ledger rides all three views; absent/unrecognized/mismatched stored ledger refused stale with the rerun hint; report unmodified; reordered corpus byte-identical reports carrying the ledger; >10-member human truncation; empty ledger serialized never omitted), src/human.rs render tests, src/verify/trust_base.rs kind-table test. Production code untouched — RED confirmed by runtime failures (missing trust_base fields) and missing kind-table consts.
