---
reviews: 2026-10-08-green-src-kernel-rs-eq-cmp-atomic-eval-scope-no.md
verdict: pass
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-7gh-claim-records-carry-reason-when-not-verified-f4-f6, pipeline-step:ro5u-review]
---

# RO5U review — specodelic-7gh GREEN diff (src/kernel.rs, tests/cli/model_check.rs)

P1 correctness — Kleene composition preserves verdicts exactly: counterexample dominates (reason dropped — the refuted verdict is its own evidence), unknown keeps the first cause, and every Unknown path now carries a Some(reason) so a reason-less unknown is structurally unreachable in eval_scope.

P2 edge cases — empty-domain ∀ still (Verified, None); cyclic_nodes/forward_closure error types match (cargo check clean); And with (Verified, Unknown) propagates b's reason; (Unknown, Unknown) keeps a's.

P3 contracts — claim_report_schema "reason where not verified" now holds for kernel records in BOTH command output entries and persisted QualifiedClaim records; verify's freshness checks compare statuses/ids/evaluator only (reason strings never compared) — no drift risk; 4v1 exit codes untouched (unknown → exploration_only, exit 0; counterexample → exit 1 via existing aggregate).

P4 findings:
- (Medium) Counterexample kernel claims now carry a reason, but citation-counterexample records (a citation resolving to a refuted rust fragment — src/citation_corpus.rs resolve_target returns (status, None)) still do not. citation_corpus.rs is outside this ticket's allowed-files list; flagged for the orchestrator rather than fixed here.
- (Low) Counterexample reason wording says "the aggregate names it as the violated invariant" — with multiple refuted claims only the first is named.
- (Low) Kernel Not() passes the raw reason through while the citation path wraps "composed ¬ over unknown: {r}" — cosmetic phrasing asymmetry only.

No Critical, no High. No refactor warranted: the (status, reason) tuple thread is already the minimal implementation mirroring the deployed citation-path pattern.

