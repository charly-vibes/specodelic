# Subagent brief: specodelic-lf3 (run 2) — migrate the compile file's property cells + wire verify-specs

You are a subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-lf3` and is hands-off: you own the scoped migration end-to-end.
The orchestrator verifies with real gates — never claim work the diff
doesn't contain.

## Authorized scope (owner-decided scoped-first)

ONLY `openspec/specs/compile/spec.md` (35 property rows, ZERO law rows —
the only core file not blocked by the law-case Revision; `specodelic` and
`kinds` each carry one law row and stay out; `packs` follows in a later
run). Do NOT touch other corpus files.

## Prerequisite is landed (specodelic-u7kc, commit 449aa5b0)

- P1: the props scratch crate takes a path dependency on the tool when the
  artifact references `specodelic::` (`SPECODELIC_TOOL_PATH` override).
- P2: `pub mod fixtures` (lib) — builder API materializing generator-named
  fixtures as concrete `Spec`/text inputs.
- P3: `pub mod scope_verdict` — properties-only verdict (never weakens
  `both_gates_required`).
- Demonstration of record: compile row `proptest_block_per_property` already
  uses a `**rust:**` fragment calling
  `specodelic::verify::blocks_from_source` /
  `specodelic::compile::properties_to_proptest` /
  `specodelic::spec::parse_str` over a materialized fixture.

## Orientation (do this first)

- `export WAI_PROJECT=specodelic`; `bd show specodelic-lf3` (notes carry the
  run-1 evidence and the scoped-first authorization).
- Read: `openspec/specs/compile/spec.md` property table + Notes;
  `.wai/projects/specodelic/research/2026-11-13-lf3-feasibility-compile.md`
  (the 35-row blocker table: which rows name fixtures, which need tool
  calls); `src/lib.rs` `fixtures` + `scope_verdict` modules;
  `src/verify.rs` (properties gate, scope verdict, scratch seam);
  `src/compile.rs` (`properties_to_proptest`, `spec_gen` emission,
  `FRAGMENT_BANNED`); `openspec/specs/verify/spec.md` (`both_gates_required`
  and the properties-gate contract); `justfile` (recipe conventions).
- `wai search "fixture"`, `wai search "verify"` — accumulated patterns.
- Start and execute the executor pipeline literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-lf3: migrate compile properties"`
  Advance ONLY via `wai pipeline next` — and ALWAYS with
  `WAI_PIPELINE_RUN=tdd-ro5-2026-11-13-specodelic-lf3-migrate-compile-properties`
  exported, so the pointer never moves the orchestrator's epic run.
- `export TMPDIR=/var/tmp/specodelic-xch1` (mkdir -p it) — MANDATORY.

## What to do

1. **Fixture builders first (RED)**: for every generator name the 35 rows
   use (e.g. `spec_file_with(transition_missing_guard: true)`,
   `property_row_with_law_case_floor()`, …), add a minimal builder to
   `src/lib.rs`'s `fixtures` module that produces the REAL input the
   property is about (a `Spec` or spec text the tool consumes). No
   re-implemented checkers; no test doubles. Keep the module documented
   (each public fn: one line).
2. **Migrate the 35 property rows** to `**rust:**` predicates that call the
   real tool (`specodelic::lint::…`, `specodelic::compile::…`,
   `specodelic::spec::…` as the row's claim requires) and assert the row's
   claim (e.g. the expected rule id is reported / the expected label
   appears). Use `## Fragments` shared helpers where a body recurs. Do not
   change requirement text, scenarios, or the typing tables.
3. **Regenerate artifacts + prove**: `spk compile` the file, regenerate
   `specodelic/compile_props.rs` + `compile.check.json`, and run
   `spk verify openspec/specs/compile/spec.md` → expect properties pass and
   the model gate current (`no_counterexample`); investigate any failing
   row honestly rather than weakening the predicate.
4. **Wire the CI gate**: add a `verify-specs` recipe to `justfile`
   scoped to exactly `openspec/specs/compile/spec.md`, using the
   properties-only verdict so the stage fails ONLY on
   `properties.state == "failed"` (never on a skipped/stale model stage —
   but the model gate for this file must also be current since its
   check.json is committed), and add it to `just ci`. Never weaken
   `both_gates_required`.
5. `cargo fmt` before commits; `pretender check` green; targeted tests
   during red/green; the orchestrator owns the full gate battery.

## Hard rules

- No checker re-implementation inside fragments; no `true`/trivial
  predicates; no `Just(name)` scaffolds as acceptance evidence.
- The 35 predicates must each genuinely assert their row's claim.
- Corpus edits are layer-table only (no requirement/scenario changes); if a
  row's claim cannot be honestly expressed even with the landed seams, STOP
  on that row, leave it as `todo_predicate!`, and report it (do not fake
  it) — the file then cannot be scoped into CI until resolved.

## Scope guard

- Allowed files: `openspec/specs/compile/spec.md` (property cells only),
  `src/lib.rs` (fixtures module additions), `justfile` (verify-specs recipe
  + ci stage), `tests/**`, regenerated `specodelic/compile_props.rs` +
  `specodelic/compile.check.json`.
- Never edit: other corpus files, other `src/**` modules (if the seams need
  a change, report it — do not widen), `packs/`, `specs/`,
  `pretender.toml`, `.wai/resources/**`, run YAMLs, `.beads/**`.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene**: `git status --short` before every commit; stage only
  your files (`git add <paths>`); blame only what the diff contains.
- **beads**: do NOT claim/close/update — the orchestrator owns bd state.
- **Do NOT push**; **do NOT run `wai close`**; **do NOT hand-edit any run
  YAML** (a prior subagent did and it broke the orchestrator's state).
- **Non-interactive shells**: `-f`/`-rf`/`-y`; **TMPDIR** as above.
- **Run `cargo fmt` before commits.**

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Migration**
- rows migrated / 35, fixtures added, verify result (properties + model)

**Gate wiring**
- recipe line + the scoped command's result

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">