---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-9o1h-compose-kernel-outcomes-into-the-citation-algebra, pipeline-step:plan]
---

TDD plan — specodelic-9o1h (E2 F2: compose kernel outcomes into the citation algebra)

DESIRED BEHAVIOR
A citation target resolves against same-run EXEC outcomes AND corpus-static KERNEL claim outcomes. Precedence in Resolver::resolve_target: exec → kernel → citation row (dependency order) → property-row unknown → missing. A file whose required claims are one verified exec row, one verified kernel row and one citation row [[exec]] ∧ [[kernel]] reports every required claim verified and the file outcome no_counterexample (today: unknown → exploration_only).

OUT OF SCOPE
- aggregate_required_claims Rule 4 (untouched); merge_corpus_statuses identity (untouched).
- Re-implementing any evaluator: the citation algebra only CONSUMES statuses.
- Kernel→citation cycles (impossible: the closed kernel grammar has no [[..]] references) — no new cycle rule.
- TLC backend semantics (kernel claims stay labeled unsupported → citing citation unknown).
- Any new table row / Scenario block in the corpus (prose clarification only).

TEST CASES (RED first)
Library level (tests/citation_algebra.rs, resolve_corpus_citations called directly):
 1. kernel evidence composed into a citation row → verified (exec ∨ kernel evidence union).
 2. empty kernel evidence map behaves exactly as today (missing-target reason unchanged).
 3. kernel unknown → citing citation unknown with a composed reason naming the kernel cause.
 4. kernel counterexample → citing citation counterexample.
CLI level (tests/cli/citation_resolution.rs):
 5. F2 acceptance: [[exec]] ∧ [[kernel]] → citation verified, all required claims verified, outcome no_counterexample, exit 0.
 6. kernel unknown → citation unknown (composed reason), outcome exploration_only.
 7. kernel counterexample → citation counterexample, Rule 2 names the citation row id, no invented trace.
 8. qualified cross-file [[otherfile.kernelrow]] composes the same way.
 9. TLC leg: kernel unsupported → citing citation unknown with the unsupported reason, never verified.
 10. regression pins: exec-only citation unchanged; citation-of-citation order/cycle reason unchanged; missing-target and property-row reasons unchanged; kernel claim appears exactly once in invariant_statuses.
CLI level (tests/cli/model_check.rs):
 11. both views agree: model-check --json and the orchestrate model_check stage publish the same statuses and outcome for one fixture.

NARROW TEST COMMAND
cargo test --test citation_algebra && cargo test --test cli citation_resolution && cargo test --test cli model_check

FULL VERIFICATION COMMAND
just ci (fmt-check, clippy -D warnings, tests, release build) + just lint-specs + openspec validate --all --strict + ah check --run-tests --all-tests

FILES LIKELY TOUCHED
src/citation_corpus.rs (ClaimEvidence + kernel evidence map + resolve_target precedence + apply_corpus_resolution/resolve_corpus_citations signature), src/kernel.rs (evidence-map helper over CorpusClaimStatus), src/orchestrate.rs (evaluate kernel claims BEFORE citation resolution, pass the evidence map), tests/citation_algebra.rs, tests/cli/citation_resolution.rs, tests/cli/model_check.rs, openspec/specs/model_check/spec.md + openspec/specs/kernel/spec.md (prose only).
