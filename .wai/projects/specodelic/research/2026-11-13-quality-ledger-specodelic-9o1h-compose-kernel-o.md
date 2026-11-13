---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-9o1h-compose-kernel-outcomes-into-the-citation-algebra, pipeline-step:quality-ledger]
---

QUALITY LEDGER (specodelic-9o1h — compose kernel outcomes into the citation algebra, E2 finding F2)

Changed:
- src/citation_corpus.rs — the citation algebra's evidence union: a new neutral ClaimEvidence record (status + labeled reason), an added kernel_evidence map on apply_corpus_resolution / resolve_corpus_citations / Resolver, the exec→kernel→citation-row→property-row→missing precedence in resolve_target, the composed reason for an unknown kernel claim, and the exec map's guard_kernel filter (disjointness by construction).
- src/kernel.rs — citation_evidence(): the ONE conversion from the kernel evaluator's CorpusClaimStatus into the algebra's neutral evidence record (kernel owns its own shape).
- src/orchestrate.rs — run_claim_gated_model_check now evaluates the kernel corpus pass BEFORE the citation resolution and hands the per-file statuses to the resolver; the claims are evaluated ONCE and the merge/aggregate are untouched.
- tests/citation_algebra.rs — resolver-level tests: composition verified, empty evidence map == pre-composition behavior, unknown composes the kernel reason, counterexample propagates, a short evidence slice degrades instead of panicking.
- tests/cli/citation_resolution.rs — CLI-level tests: the F2 acceptance ([[exec]] ∧ [[kernel]] → verified → no_counterexample → verify passes), the unknown leg (composed reason naming the kernel cause), the counterexample leg (Rule 2 names the citation row, no invented trace), the TLC unsupported leg, the qualified cross-file leg, the both-views parity (CLI model-check --json vs the orchestrate model_check stage), and the regression pins (exec-only citation, citation-of-citation order, cycle/missing/property reasons unchanged, kernel claim exactly once).
- openspec/specs/model_check/spec.md, openspec/specs/kernel/spec.md — prose Notes stating the cross-evaluator composition (no new table rows, no new scenarios, no contract changes).

Verified:
- cargo test --test cli → 332 passed, 0 failed (1 ignored, pre-existing); cargo test --test citation_algebra → 13 passed.
- just ci (fmt-check, clippy -D warnings, full test suite, release build, openspec validate, lint-deltas, model-check-specs, verify-specs, sync-sections, doc examples, pack doc counts, lint baseline, pretender check, kernel-agreement, sibling-blockers, gate-drill-lock, guard-espectacular) → exit 0.
- ah check --run-tests --all-tests → 641 passed, 0 findings. openspec validate --all --strict → 53 passed, 0 failed. just lint-specs → ok, issues [] (advisories only). pretender check → exit 0.
- Non-vacuity: with the implementation reverted (git stash of the three src files) the six composition tests FAIL for the expected reason and the real binary reports kc unknown / outcome exploration_only; with the fix the same fixture reports kc verified / outcome no_counterexample end to end (verified through the real binary, both ways).
- Both views: the CLI model-check envelope and the orchestrate model_check stage publish identical statuses and outcome for one fixture (test in tests/cli/citation_resolution.rs).

Review:
- RO5U self-review recorded (artifact 2026-11-13-ro5u-review-specodelic-9o1h-cross-evaluator-c.md): 0 critical, 0 high, 0 medium, 2 low — both fixed in 44d7267 (total evidence lookup + documented alignment contract + a regression test). No findings remain.

Risks:
- The composition is one-way by grammar (the closed kernel grammar has no [[..]] references), so no kernel→citation cycle can arise; if a future atomic ever admitted a reference, the resolver's existing cycle machinery would need a look.
- A citation row over a COUNTEREXAMPLE kernel claim carries no reason of its own (mirroring the exec leg); the kernel claim's own row carries the reason.
- The new CLI tests live in tests/cli/citation_resolution.rs, not tests/cli/model_check.rs (pretender file_lines ratchet: 2300 max, HEAD 2217) — a deviation from the brief's suggestion, recorded.

Next:
- Orchestrator: close specodelic-9o1h (bd close), push the branch (this subagent does not push), and record the ship note. Commits: ec2b5ef (feat), 239297e (docs), 42d352a (refactor), 44d7267 (review fix).
