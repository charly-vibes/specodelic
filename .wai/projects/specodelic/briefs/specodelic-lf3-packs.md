# Subagent brief: specodelic-lf3 (run 3) — migrate the packs file's property cells

You are a subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-lf3` and is hands-off: you own this scoped migration end-to-end.
The orchestrator verifies with real gates — never claim work the diff
doesn't contain.

## Authorized scope (owner-decided scoped-first, run 3)

ONLY `openspec/specs/packs/spec.md` (16 property rows, ZERO law rows).
The compile file is already done (run 2, commit 861491e1 — read it as the
worked example). Do NOT touch other corpus files.

## Precedent to follow (run 2)

- All 35 compile rows were migrated with fixtures-registry builders
  (`src/lib.rs` `fixtures` module) + `**rust:**` predicates calling the
  real tool; artifacts regenerated; `verify-specs` recipe added scoped to
  compile with the properties-only verdict (`scope_verdict`), wired into
  `just ci`; the recipe proven to exit 1 on a deliberately broken fragment.
- Harness seams: P1 tool path-dep (`SPECODELIC_TOOL_PATH`), P2 fixtures
  builders, P3 properties-only verdict (both_gates_required untouched).

## Orientation (do this first)

- `export WAI_PROJECT=specodelic`; `bd show specodelic-lf3` (notes carry
  run 1/2 evidence).
- Read: `openspec/specs/packs/spec.md` property table + Notes;
  `.wai/projects/specodelic/research/2026-11-13-lf3-feasibility-packs.md`
  (row-by-row blocker table); `src/lib.rs` `fixtures`/`scope_verdict`;
  `src/verify.rs`; `src/compile.rs` emission; `justfile` `verify-specs`;
  the run-2 commit `861491e1` (fixtures + predicates + recipe shape).
- Start the executor pipeline literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-lf3: migrate packs properties"`
  Advance ONLY via `wai pipeline next` with
  `WAI_PIPELINE_RUN=tdd-ro5-2026-11-13-specodelic-lf3-migrate-packs-properties`
  exported — never hand-edit run YAMLs, never let the pointer move the
  orchestrator's epic run.
- `export TMPDIR=/var/tmp/specodelic-xch1` (mkdir -p it).

## What to do

1. **Fixture builders** for every generator name the 16 rows use (pack
   corpora, pinned/older-revision packs, consumer specs with `uses`
   columns, mixed claim corpora…): add minimal builders to `src/lib.rs`'s
   `fixtures` module producing the REAL inputs the pack scanner/linter
   consumes. Verify each builder actually exercises the row's claim
   (pack discovery by scan, lifecycle filtering, etc.).
2. **Migrate all 16 property rows** to `**rust:**` predicates calling the
   real tool (`specodelic::lint::…`, `specodelic::packs::…` as needed) and
   asserting their claim. Shared helpers via `## Fragments` where bodies
   recur. Layer-table edits only — no requirement/scenario changes.
3. **Regenerate + prove**: `spk compile` → `specodelic/packs_props.rs` +
   `packs.check.json`; run the properties gate and the model gate for the
   file; `spk lint` stays 0 issues.
4. **Extend the CI gate**: update the `verify-specs` recipe scope to
   `[openspec/specs/compile/spec.md, openspec/specs/packs/spec.md]`
   (properties-only verdict; never weaken `both_gates_required`), and
   re-prove the gate fails on a deliberately broken fragment.
5. `cargo fmt` before commits; `pretender check` green; targeted tests
   during red/green only.

## Hard rules

- No checker re-implementation; no `true`/`Just(name)`/trivial scaffolds.
- Every predicate genuinely asserts its row's claim.
- If a row cannot be honestly expressed even with the landed seams, STOP
  on it, leave `todo_predicate!`, report it — and then do NOT extend the
  CI scope to packs until it is resolved (a false green is forbidden).

## Scope guard

- Allowed: `openspec/specs/packs/spec.md` (property cells only),
  `src/lib.rs` (fixtures additions), `justfile` (verify-specs scope),
  `tests/**`, regenerated `specodelic/packs_props.rs` +
  `specodelic/packs.check.json`.
- Never edit: other corpus files, other `src/**` modules, `packs/`,
  `specs/`, `pretender.toml`, `.wai/resources/**`, run YAMLs, `.beads/**`.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene**: `git status --short` before every commit; stage only
  your files; blame only what the diff contains.
- **beads**: do NOT claim/close/update — the orchestrator owns bd state.
- **Do NOT push**; **do NOT run `wai close`**; **do NOT hand-edit run
  YAMLs**; do not run `wai pipeline next` without your own
  `WAI_PIPELINE_RUN` exported.
- **Non-interactive shells**: `-f`/`-rf`/`-y`; **TMPDIR** as above.
- **Run `cargo fmt` before commits.**

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Migration**
- rows migrated / 16, fixtures added, properties + model gate results

**Gate wiring**
- recipe scope + the scoped command's result + the broken-fragment proof

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">