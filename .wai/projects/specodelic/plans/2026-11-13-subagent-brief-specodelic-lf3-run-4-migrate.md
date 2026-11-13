---
tags: [pipeline-run:epic-orchestrator-2026-11-13-specodelic-lf3-migrate-specodelic-kinds-properties-run-4, pipeline-step:write-brief]
---

# Subagent brief: specodelic-lf3 (run 4) — migrate the specodelic + kinds property cells

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-lf3` and is hands-off: you own this scoped migration end-to-end.
The orchestrator verifies with real gates — never claim work the diff doesn't
contain.

## Authorized scope (owner's standing scoped-first authorization, run 4)

ONLY `openspec/specs/specodelic/spec.md` (30 property rows, 1 law row
`rename_naturality`) and `openspec/specs/kinds/spec.md` (17 property rows,
1 law row `kind_shape_naturality`). The compile file (35 rows) and packs file
(16 rows) are already done — read them as the worked examples
(commits `861491e1`, `c34a864c`, `55f058c0`). Do NOT touch other corpus files.

## Precedent to follow

- Fixtures registry builders live in `src/lib.rs`'s `fixtures` module; per-file
  test files (`tests/compile_fixtures.rs`, `tests/packs_fixtures.rs`) pin the
  corpus↔registry correspondence, lint/compile the fixtures, and prove the
  fixtures are real inputs (not `Just(name)`/`true` scaffolds).
- Property cells carry `**rust:**` predicates calling the REAL tool
  (`specodelic::lint::lint_corpus`, `specodelic::compile::*`,
  `specodelic::kinds::*`, `specodelic::verify::*`, …) over materialized
  fixtures. The scratch crate may depend on the tool when the artifact
  references `specodelic::` (`SPECODELIC_TOOL_PATH` override — landed in u7kc).
- The `verify-specs` recipe (justfile) runs the properties-only verdict over a
  file list; it never weakens `both_gates_required`.
- **Revision 21 grammar for law rows** (landed `f1d5698f`): the `**name:**`
  labels stay the enumeration of record; a case body is written under its case
  key — `**name::**` immediately followed by the fragment marker in fragment
  position: ``**identity::** `**rust:** <expr>` ``. One body per case; a case
  without a body keeps `todo_predicate!`. Invalid shapes (unassociated marker,
  duplicate body, unenumerated case, empty body, keyless body) fail labeled.

## Known residuals to resolve honestly (from the 2026-11-13 classification)

- `specodelic` row `silently_shrunk_id_set_rejected` needs a revision-diff
  seam — find one; if the tool exposes no honest seam, STOP on that row.
- `specodelic` row `prose_does_not_affect_lint` — two fixture specs differing
  only in rationale prose must lint identically.
- `kinds` rows `kind_assignment_failure_label_asserted` /
  `shape_check_failure_label_asserted` are three-site corpus-consistency
  claims: express them against the corpus text through a fixture builder or a
  public tool seam (fragments must not use `std::fs`/`include_str!` — builder
  code in `src/lib.rs` may).
- Every other row is a tool-behavior claim over a fixture; the seam exists.

## What to do

1. **Fixture builders** for every generator name the 47 rows use (spec files
   with one defect each, repos/clusters, revision diffs, schema shapes, rename
   inputs, arbitrary well-formed rows…). Each builder must produce the REAL
   input the corresponding checker consumes, and the fixture tests must prove
   it (lint/compile it; assert the defect is what the row claims).
2. **Migrate the 47 property rows** to `**rust:**` predicates calling the real
   tool and asserting their claim. Law rows use per-case bodies (Revision 21).
   Layer-table edits only — no requirement/scenario prose changes.
3. **Regenerate + prove per file**: `spk compile` and `spk model-check` over the
   two files; `spk lint` stays 0 issues; run the properties gate for each file.
4. **Extend the CI gate** to `[openspec/specs/compile/spec.md,
   openspec/specs/packs/spec.md, openspec/specs/specodelic/spec.md,
   openspec/specs/kinds/spec.md]` ONLY if every row in both new files is
   honestly executable (properties-only verdict; never weaken
   `both_gates_required`). Prove the gate fails on a deliberately broken
   fragment in each new file and that a mutation of a strengthened predicate
   fails (non-vacuity), then restore green.
5. `cargo fmt` before commits; `pretender check` green; targeted tests during
   red/green only.

## Hard rules

- No checker re-implementation in fragments; no `true`/`Just(name)`/trivial
  scaffolds; every predicate genuinely asserts its row's claim.
- If a row cannot be honestly expressed, STOP on it, leave `todo_predicate!`,
  report it — and do NOT extend the CI scope to that file (a false green is
  forbidden). lf3 then stays open for that row.
- Do NOT modify `openspec/specs/specodelic/spec.md` / `compile/spec.md` format
  rules, `src/law.rs`, or any other ticket's files. A genuine format gap is a
  STOP-and-report, not an inline Revision.

## Scope guard

Allowed: `openspec/specs/specodelic/spec.md` and
`openspec/specs/kinds/spec.md` (property cells + Notes only),
`src/lib.rs` (fixtures additions), `justfile` (verify-specs scope),
`tests/**`, regenerated `specodelic/specodelic_props.rs` /
`specodelic/kinds_props.rs` and their `*.check.json`.

Never edit: other corpus files, other `src/**` modules, `packs/`, `specs/`,
`pretender.toml`, `.wai/resources/**`, `.wai/pipeline-runs/**`, `.beads/**`,
`.genesis/tools.toml` churn.

## Orientation

- `export WAI_PROJECT=specodelic`; `export TMPDIR=/var/tmp/specodelic-xch1`
  (`mkdir -p` it); `bd show specodelic-lf3`; `bd heartbeat specodelic-lf3`
  before long phases.
- Read: both target files' property tables + Notes; the feasibility research
  (`2026-11-13-lf3-feasibility-specodelic.md`, `-kinds.md`); `src/lib.rs`
  `fixtures`; `src/lint/mod.rs`, `src/compile.rs`, `src/kinds.rs`,
  `src/verify.rs`; `justfile` `verify-specs`; the run-2/run-3 commits above.
- Start your executor pipeline:
  `wai pipeline start tdd-ro5 --topic="specodelic-lf3: migrate specodelic + kinds properties"`
  Export the printed `WAI_PIPELINE_RUN=<tdd-ro5 run id>` and advance ONLY via
  `wai pipeline next`. Never hand-edit run YAMLs; never advance the
  orchestrator's epic run
  (`epic-orchestrator-2026-11-13-specodelic-lf3-migrate-specodelic-kinds-properties-run-4`).

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Rows**
- specodelic: <n>/30 executable (<ids of any left as todo_predicate! and why>)
- kinds: <n>/17 executable (<same>)

**Gates run**
- `just verify-specs` → <result, incl. broken-fragment + mutation proofs>
- `just ci` → <result>
- `ah check --run-tests --all-tests` → <result>
- `just lint-specs` / `openspec validate --all --strict` → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">
