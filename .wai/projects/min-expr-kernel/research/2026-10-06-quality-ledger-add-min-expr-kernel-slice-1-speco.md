---
tags: [pipeline-run:tdd-ro5-2026-10-06-add-min-expr-kernel-slice-1-specodelic-txo-guard-citation-semantics, pipeline-step:quality-ledger]
---

QUALITY LEDGER: add-min-expr-kernel slice 1 (specodelic-txo) — guard citation semantics.

Changed:
- src/compile.rs — citation algebra (ThreeValued, CitationExpr, parse_citation_expr, evaluate_citation), ModelIr.guard_citations extraction (pure widening); refactor: strip_guard_cell() helper dedupes cell normalization, dropped unused parse binding; RO5U F-1: reject ']' in citation ids (malformed stays prose)
- src/model_check.rs — InvariantStatus + RunReport.invariant_statuses wired across native/TLC/exec backends; citation_statuses evaluator; RO5U F-2: honest unknown on failed re-parse instead of silent drop (file at 2282/2300, ratchet intact)
- src/verify.rs, tests/citation_algebra.rs — fmt + test updates (8 integration tests, out-of-module for the ratchet)
- Commits: 1649fbf (GREEN, orchestrator), 444ad1c (TIDY refactor), beff19d (fmt), c2bc2fc (RO5U fixes)

Verified:
- just test: 23/23 suites ok, 0 failures (incl. citation_algebra 8/8, pretender gate in-hook green)
- cargo fmt --check: clean; cargo clippy -- -D warnings (lib): clean
- pretender check: gate passes; model_check.rs 2282 < 2300 (shrink-only ratchet preserved)
- ah check (deployed): 0 issues per pre-commit hook
- ah check --changes add-min-expr-kernel: 'add-min-expr-kernel' is valid (openspec); 24 structural findings — KNOWN DEBTS, not introduced by txo work: (a) no-toml ×23 for change-phase delta scenarios (ah binds contract TOMLs to deployed scenarios only; the 5 slice-1 model-check TOMLs are stashed at /tmp/txo-contracts/*.toml and MUST be copied into the archive commit per the dual-format recipe); (b) overlay-conflict ×1: compile/mid-span-occurrence-is-a-mention, pre-existing from the scaffold commit 18f4b8b
- openspec validate add-min-expr-kernel: valid

Review:
- RO5U: PASS, CONVERGED — 0 critical / 0 high / 1 medium (F-1 bracket-in-id, FIXED) / 3 low (F-2 FIXED; F-3 tracked: report-field redundancy by serde design; F-4 tracked for slice-2: double-negation prose fallback becomes labeled in kernel-grammar increment)

Risks:
- clippy --all-targets has 8 PRE-EXISTING errors in tests/gate_drill_lock.rs, tests/cli/feedback_init.rs, tests/cli/parse_misc.rs (present at GREEN HEAD with changes stashed; outside txo scope — blocks just ci, not just test)
- no-toml/overlay-conflict debts above are owned by the archive task, not slice 1
- clippy --all-targets, pretender hotspot displays (compile.rs cyclomatic 17, model_check.rs cognitive 29) are at-ceiling grandfathered, not txo regressions

Next:
- ship-close: close bd specodelic-txo, bd export to .beads/issues.jsonl, mark tasks.md §2 (2.1-2.4) done with commit evidence, export commit. No push (orchestrator).
