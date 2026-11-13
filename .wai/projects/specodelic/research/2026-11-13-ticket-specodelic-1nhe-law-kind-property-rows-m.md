---
tags: [pipeline-run:epic-orchestrator-2026-11-13-specodelic-1nhe-law-kind-per-case-fragment-bodies, pipeline-step:select-ticket]
---

TICKET: specodelic-1nhe — law-kind property rows may carry per-case fragment bodies (format Revision, M4).

WHY THIS ONE NOW: epic specodelic-ez9a's only open child is specodelic-lf3 (IN_PROGRESS), and lf3's remaining 47 rows (openspec/specs/specodelic/spec.md 30 rows, 1 law row rename_naturality; openspec/specs/kinds/spec.md 17 rows, 1 law row kind_shape_naturality) are hard-blocked by compile.md's `fragment_law_rejected` invariant plus the one-marker-per-cell rule. No other ready issue unblocks the epic. Queue head confirmed by the 2026-11-13 continuation-3 handoff (branch topic resume-epic-ez9a).

STATE AT SELECTION: main == origin/main at 59136eb3, tree clean except WAI bookkeeping. Gates at HEAD (handoff evidence): just ci exit 0, ah check --run-tests --all-tests 641 passed/0 findings, openspec validate 53/53, just lint-specs 48/48, just verify-specs exit 0 over [compile, packs]. 51/51 lawful corpus property rows (compile 35 + packs 16) already executable. lf3 scope status: compile + packs done; specodelic + kinds blocked on 1nhe.

TICKET SCOPE (from bd specodelic-1nhe): a format Revision (specodelic.md/compile.md delta) defining per-case fragment bodies for law-kind rows (case-labelled body map), compile emission (one block per case with that case's body), and the law_cases lint interaction. Runner seams from u7kc already let a fragment call the real tool. Anti-goals: no weakening of law_cases labels; no single-body-per-row semantics.

CONSTRAINTS: scoped-first is the owner's standing authorization (extends per run); espectacular is read-only over openspec except `just archive-change`; subagents never push, never bd close, never wai close, and never hand-edit or advance another run's YAML; TMPDIR=/var/tmp/specodelic-xch1; ah check --run-tests selects by worktree diff (use --all-tests for the full suite); tools.toml no-op churn must be reverted.

NEXT: land the Revision spec-first inside the tdd-ro5 child run (red -> green -> refactor -> ro5u review -> quality ledger), then return to lf3's specodelic + kinds migration and extend verify-specs scope.
