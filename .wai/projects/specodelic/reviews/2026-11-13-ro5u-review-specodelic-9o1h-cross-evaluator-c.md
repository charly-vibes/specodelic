---
reviews: 2026-11-13-green-specodelic-9o1h-design-decision-of-recor.md
verdict: pass
severity: {critical: 0, high: 0, medium: 0, low: 2}
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-9o1h-compose-kernel-outcomes-into-the-citation-algebra, pipeline-step:fix-review]
---

# RO5U review — specodelic-9o1h (cross-evaluator citation composition)

Diff reviewed: 72f8ede8..44d7267 (src/citation_corpus.rs, src/kernel.rs,
src/orchestrate.rs, tests/citation_algebra.rs, tests/cli/citation_resolution.rs,
openspec/specs/model_check/spec.md, openspec/specs/kernel/spec.md).

## CRITICAL
None.

## HIGH
None.

## MEDIUM
None.

## LOW (2 — both FIXED in 44d7267)

1. **src/citation_corpus.rs — `Resolver::resolve_target`**
   The new kernel-evidence lookup indexed `kernel_evidence[idx]` directly,
   so a caller passing a slice shorter than the spec list would panic with
   an index-out-of-bounds instead of degrading to "no kernel evidence for
   that file".
   *Evidence*: my own first draft of the library test passed `&[]` and
   panicked at `src/citation_corpus.rs:491`.
   *Fix*: the lookup is now total (`.get(idx).and_then(|file| file.get(&row))`);
   the alignment contract (one map per spec, in specs order) is documented
   on `apply_corpus_resolution`; `tests/citation_algebra.rs` pins the
   degradation (`a_missing_kernel_evidence_map_degrades_to_no_evidence_never_a_panic`).

2. **src/citation_corpus.rs — `apply_corpus_resolution`**
   The new parameter's alignment contract was undocumented (the exec map's
   contract was documented in the function body).
   *Fix*: the doc comment states the invariant and the degradation rule.

## Notes (no action — deliberate)

- A citation row over a **counterexample** kernel claim carries no reason of
  its own (the kernel claim's own row carries it). This mirrors the exec leg
  exactly (a refuted exec leg carries no reason); the tests assert statuses,
  not reasons.
- The exec map now also filters `guard_kernel` rows. A no-op in practice (a
  kernel claim is never a compiled executable invariant — `extract_model_ir`
  puts a row in `guard_citations` OR `guard_kernel`, and kernel rows carry no
  fragment), but it makes the exec/kernel disjointness explicit by construction.
- The new CLI tests live in `tests/cli/citation_resolution.rs` rather than
  `tests/cli/model_check.rs`: the pretender `file_lines` ratchet caps the
  latter at 2300 lines (HEAD is 2217), and adding the block there turned the
  gate red (`pretender check` exit 1). Recorded as a deviation, not a finding.

## Review method

Every changed line re-read against the design decision of record (orchestrator,
2026-11-13); the evidence union re-derived from `compile::extract_model_ir`
(`guard_citations` ∩ `guard_kernel` = ∅); each Kleene leg checked against the
algebra's existing semantics; every claim verified against a real run:
`cargo test --test cli` (332 passed), `cargo test --test citation_algebra`
(13 passed), `cargo clippy -- -D warnings` (clean), `pretender check` (exit 0),
`ah check --run-tests --all-tests` (641 passed, 0 findings),
`openspec validate --all --strict` (53 passed).

