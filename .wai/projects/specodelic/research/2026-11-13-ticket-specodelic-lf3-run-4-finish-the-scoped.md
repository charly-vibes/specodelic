---
tags: [pipeline-run:epic-orchestrator-2026-11-13-specodelic-lf3-migrate-specodelic-kinds-properties-run-4, pipeline-step:select-ticket]
---

TICKET: specodelic-lf3 (run 4) — finish the scoped-first corpus migration: openspec/specs/specodelic/spec.md (30 rows, 1 law row) and openspec/specs/kinds/spec.md (17 rows, 1 law row), then extend the verify-specs gate scope to all four files.

WHY THIS ONE NOW: it is epic specodelic-ez9a's only open child, and the last format blocker is gone — specodelic-1nhe landed Revision 21 (commit f1d5698f, pushed d774ee28): law-kind rows carry one executable body per named case via the shared law.rs grammar. The runner seams (u7kc: tool path-dep, fixtures API, properties-only verdict) are already landed. 51/51 rows are executable in compile (35) + packs (16); the remaining 47 rows are exactly this ticket's scope.

STATE: main == origin/main at b048a048, tree clean. Gates at HEAD: just ci 0, just test 0, ah check --run-tests --all-tests 641/0, openspec validate 53/53, lint 48/0, verify-specs 0 over [compile, packs].

ROW CLASSIFICATION (2026-11-13 feasibility research, pre-seams): all 47 rows are tool-behavior claims (blocker T) needing the tool-dependency seam — now landed. Known residuals to resolve honestly:
- specodelic row 28 silently_shrunk_id_set_rejected needs a revision-diff seam; kinds rows 16/17 are corpus-consistency claims (label strings across three corpus sites) — fixture/corpus access must stay within fragment_hygiene (no std::fs/include_str inside fragments; builder functions in src/lib.rs are allowed).
- specodelic row 27 prose_does_not_affect_lint needs two fixture specs differing only in prose.
- Law rows: specodelic rename_naturality (identity/associativity/naturality) and kinds kind_shape_naturality (identity/associativity) now use per-case bodies (**name::** case key + **rust:** marker in fragment position; labels **name:** remain the enumeration — Revision 21).
If a row cannot be honestly expressed even with the landed seams, STOP on that row, leave todo_predicate!, report it, and do NOT extend the CI scope to that file (a false green is forbidden).

CONSTRAINTS: scoped-first is the owner's standing authorization (extends per run; law rows now unblocked); espectacular read-only over openspec except just archive-change; subagents never push/bd close/wai close and never edit another run's YAML; TMPDIR=/var/tmp/specodelic-xch1; ah check --run-tests selects by worktree diff (use --all-tests); tools.toml no-op churn reverted.

NEXT: brief -> spawn subagent -> verify -> push -> close lf3 (epic ez9a then has no open children).
