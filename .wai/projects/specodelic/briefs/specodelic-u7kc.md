# Subagent brief: specodelic-u7kc — verify-runner seams (tool dependency + fixture API + properties-only verdict)

You are a subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-u7kc` and is hands-off: you own the design record + implementation
end-to-end. The orchestrator verifies with real gates — never claim work the
diff doesn't contain.

## Why (measured, 2026-11-13)

The lf3 scoped run proved 0/537 corpus property rows are honestly
expressible: the props scratch crate cannot see the tool (`SCRATCH_MANIFEST`
= proptest only), generators are `Just(name)` constants, fragments cannot
read files or spawn. Full evidence:
`.wai/projects/specodelic/research/2026-11-13-lf3-feasibility-summary.md`
(sections 2, 5) + per-file tables.

## Orientation (do this first)

- `export WAI_PROJECT=specodelic`; `bd show specodelic-u7kc` (three seams
  P1/P2/P3 + acceptance criteria — read them literally).
- Read: `src/verify.rs` (`SCRATCH_MANIFEST`, `CargoRunner`, the
  `PropertiesRunner` seam, `evaluate_properties_gate`, `RunReport` output
  surface, `both_gates_required` semantics and the stage precedence that
  hides `properties_failed` behind `missing_model_run`);
  `src/compile.rs` (`properties_to_proptest`, `spec_gen` emission,
  `FRAGMENT_BANNED`); `src/lib.rs` (public module surface);
  `openspec/specs/verify/spec.md` and `openspec/specs/compile/spec.md`
  (the contracts you may extend — dual-format, via `just archive-change`
  only); `justfile` (recipe conventions); `deny.toml` (dependency rules).
- `wai search "verify runner"`, `wai search "fixture"` — accumulated
  patterns.
- Start and execute the executor pipeline literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-u7kc: verify runner seams"`
  Advance ONLY via `wai pipeline next` — never hand-edit run YAMLs.
- `export TMPDIR=/var/tmp/specodelic-xch1` (mkdir -p it) — MANDATORY.

## What to build

**Spec first** (openspec change `add-verify-runner-seams`: proposal.md,
design.md, tasks.md, deltas on `verify` and (if emission changes) `compile`,
layer sidecars `specs/<cap>/layer.md` — full post-change layer per
`src/archive_companion.rs` + `openspec/changes/archive/2026-10-19-add-delta-layer-sidecar/specs/*/layer.md`).
design.md MUST settle and justify:
- **P1** the tool-dependency seam: scratch `Cargo.toml` gains
  `specodelic = { path = env!(CARGO_MANIFEST_DIR) }` (exact shape your
  call; mind the shared scratch target dir, the lib rebuild cost, deny.toml
  applicability, and offline behavior). Keep the runner contract unchanged
  (same `PropertiesRunner` trait, same report shape).
- **P2** the fixture API: the corpus's generators name fixtures
  (`spec_file_with(transition.to_not_in_states: true)`,
  `row_belonging_to_a_sixth_section(...)`, `pack_pinned_to_older_revision()`,
  `mixed_claim_corpus()`). Choose the MINIMAL honest shape — an in-corpus
  `## Fixtures` section would be a format Revision (out of scope here), so
  the expected shape is a small `specodelic::fixtures` builder module in
  the lib (a public, documented API; keep it minimal and general enough for
  the lf3 files) OR a name→fixture registry the runner materializes. Every
  fixture must produce a concrete `Spec`/corpus the REAL checker consumes —
  no test doubles, no re-implemented checkers.
- **P3** properties-only verdict: a mode/flag (e.g. `--properties-only`)
  or envelope field proving `properties_failed` exactly when a block
  fails, NOT reported for a missing/stale model run, and never weakening
  `both_gates_required` (full verify semantics unchanged).
Then the implementation (RED first):
- RED: a scratch spec whose fragment calls `specodelic::lint::lint_corpus`
  fails today ("unresolved crate") and compiles+executes after P1; a
  fixture-named generator materializes; the properties-only verdict fires
  exactly per P3.
- GREEN: the seams in `src/verify.rs` (+ `src/lib.rs` fixtures module if
  chosen, + `src/compile.rs` only if emission changes).
- **Demonstration (acceptance)**: migrate ONE property row of
  `openspec/specs/compile/spec.md` (no law row there) to a `**rust:**`
  fragment that CALLS the tool, passes, with a REAL strategy — via the
  change delta + `just archive-change` for the corpus edit. Do NOT migrate
  more than the one demonstration row; the bulk migration is lf3's next run.
- Then `spk init --force` (catalog-only diff), archive the change, and
  gates: targeted tests during red/green; `cargo fmt` before commits;
  `pretender check` green.

## Hard rules

- Never re-implement a checker inside a fragment (M6 tautology).
- Never `Just(name)`/`true`/trivial scaffolds as acceptance evidence.
- Never weaken `both_gates_required`; the models gate keeps its semantics.
- Minimal public API: everything you add to `src/lib.rs` is a supported
  commitment — justify each symbol in design.md.
- Law-case rows stay untouched (separate Revision).

## Scope guard

- Allowed files: `src/verify.rs`, `src/lib.rs` (fixtures module only),
  `src/compile.rs` (only if emission changes), `tests/**`,
  `openspec/changes/add-verify-runner-seams/**`,
  `openspec/specs/**` ONLY via `just archive-change`, `justfile` (only if
  the change's contract requires a recipe), `.espectacular/` ONLY new
  contract TOMLs for YOUR scenarios, `AGENTS.md` ONLY via `spk init --force`.
- Never edit: other `src/**` modules, `packs/`, `specs/` (domain corpus),
  `pretender.toml`, `.wai/resources/**`, run YAMLs, `.beads/**`.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene**: `git status --short` before every commit; stage only
  your files (`git add <paths>`); blame only what the diff contains.
- **beads**: do NOT claim/close/update — the orchestrator owns bd state.
- **Do NOT push**; **do NOT run `wai close`** (a prior subagent's
  `wai close` corrupted the orchestrator's run bookkeeping).
- **New source files** need Purpose/Responsibilities/Rationale headers;
  contract-changing edits update Rationale.
- **Non-interactive shells**: `-f`/`-rf`/`-y`; **TMPDIR** as above.
- **Run `cargo fmt` before commits.**

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Seams**
- P1 / P2 / P3: chosen shape + the test names proving each

**Demonstration**
- the migrated compile row + verify result (properties pass, model gate untouched)

**Gates run**
- targeted RED/GREEN → results; `just archive-change` → result

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">