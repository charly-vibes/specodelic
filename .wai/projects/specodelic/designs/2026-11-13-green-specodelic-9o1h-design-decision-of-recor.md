---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-9o1h-compose-kernel-outcomes-into-the-citation-algebra, pipeline-step:green]
---

GREEN (specodelic-9o1h) — DESIGN DECISION OF RECORD (orchestrator, 2026-11-13), implemented as specified.

1. Evidence union: a citation target resolves against same-run EXEC outcomes AND corpus-static KERNEL claim outcomes. Precedence in Resolver::resolve_target: exec → kernel → citation row (dependency order) → property-row unknown → missing. The exec and kernel sets are disjoint (a row carries exactly one evaluator — compile::extract_model_ir puts a row in guard_citations OR guard_kernel, never both), so the order documents intent.
2. Evaluate kernel claims first: kernel::evaluate_corpus_claims now runs BEFORE the citation resolution in orchestrate::run_claim_gated_model_check, and the per-file kernel statuses are passed into apply_corpus_resolution / resolve_corpus_citations as an added evidence map (kernel_evidence: &[BTreeMap<String, ClaimEvidence>], one map per spec, aligned with the specs — the same alignment contract the exec map already had). The kernel claims are evaluated ONCE; the citation algebra only CONSUMES the statuses (hard rule: no evaluator re-implemented in the algebra).
3. TLC backend: kernel claims come back unknown with the labeled unsupported reason; a citation over them stays unknown (Kleene absorb, never pass). No special case.
4. One-way composition: the closed kernel grammar contains no [[..]] references, so kernel→citation cannot cycle. Citation→citation dependency and cycle semantics are unchanged (the resolver's memo/visiting machinery is untouched).
5. No aggregate change: aggregate_required_claims Rule 4 is untouched. The acceptance flip is that [[exec]] ∧ [[kernel]] now verifies and the file reports no_counterexample when every required claim discharges.
6. Status identity unchanged: merge_corpus_statuses still appends each kernel claim exactly once; a citation row citing a kernel claim is its own row; verify consumes the persisted outcome unchanged.
7. Docs: prose in openspec/specs/model_check/spec.md (Notes) and openspec/specs/kernel/spec.md (Notes) — no new table rows, no new #### Scenario: blocks, no contract changes.

Shapes: citation_corpus::ClaimEvidence { status: ThreeValued, reason: Option<String> } — a neutral evidence record so the algebra stays evaluator-agnostic; kernel::citation_evidence(&[CorpusClaimStatus]) -> BTreeMap<String, ClaimEvidence> is the one conversion (kernel owns its own shape). An unknown kernel claim composes the reason 'kernel claim `X` is unknown: <kernel cause>'; a verified or counterexample kernel claim carries no reason on the citing row (mirroring the exec leg). The exec map now also excludes guard_kernel rows, so the two evidence sets are disjoint by construction.

Measured: the F2 fixture (exec row c1 + kernel row ka + citation row kc = [[c1]] ∧ [[ka]]) flips from kc unknown / outcome exploration_only (implementation reverted, proven by stash) to kc verified / outcome no_counterexample end to end through the real binary; verify accepts the clean aggregate.
