---
reviews: 2026-11-27-ro5u-trust-ledger-slices.md
verdict: pass
tags: [pipeline-run:tdd-ro5-2026-11-27-trust-ledger-derivation-slices-1-3, pipeline-step:ro5u-review]
---

# RO5U review: trust-ledger-derivation slices 1-3

Diff reviewed: c1f2e052..45e6db8 (src/verify/trust_base.rs, src/verify/gate.rs, surfaces in src/commands/model_check.rs + src/human.rs, 16 contract tomls, docs).

**Verdict: PASS — 0 Critical, 0 High, 2 Medium, 0 Low.**

## Medium

- **M1** `src/commands/model_check.rs` `cmd_verify` (~line 421) + `src/human.rs` `trust_base_lines` empty message (~line 276): the empty-ledger human line says "the corpus declares no unchecked claim", but the derivation scope is the verify invocation's file set (`derive_trust_base(&specs)`). A single-file verify of a clean file inside a dirty corpus overclaims corpus-wide emptiness.
  Fix: reword to "the checked scope declares no unchecked claim and no machine-uninterpreted guard".
- **M2** `src/verify/trust_base.rs` `corpus_members` (~line 225): a row with multiple `satisfies` links to foreign extension points yields duplicate members pre-dedup; correctness relies on `sort` + `dedup_by`, safe only because the sort is total on (kind, source, id). Verified safe, but the precondition is implicit.
  Fix: dedup via `BTreeSet<TrustMember>` or pin the total-sort precondition in a comment.

## Verified correct (stage B)

- One derivation, three surfaces: `cmd_verify` derives once; model-check persists via `fresh_report_json`; verify recomputes `stored_trust_base_agrees` against the same `derive_trust_base`.
- Absent vs empty: `stored_trust_base_gate` rejects a missing `trust_base` before any empty read.
- Run limits excluded: `run_bound_and_digest_stays_out_of_the_ledger`.
- Guard classification reads the compiled ModelIr, not guard text: `guard_classification_reads_the_model_ir`.
- Human cap = 10 with stated truncation: `trust_base_lines_count_per_kind_and_truncate`.
- Ledger rides verified AND blocked verify entries (`human.rs` render closure pushes `trust_base_lines` outside the blocked branch).

Review only — no fixes applied.
