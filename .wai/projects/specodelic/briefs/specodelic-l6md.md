# Subagent brief: specodelic-l6md — audit slice 3 refactor: share the declared-ids index with lint (task 3.3)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-l6md` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=specodelic`
- This is a REFACTOR ticket: the pin is ALL EXISTING tests staying green
  throughout — full suite (985 at HEAD), plus `lint` findings and `audit`
  envelopes staying byte-identical on fixed fixtures (prove with before/
  after invocations compared with `diff`, artifacts in your own
  `/var/tmp/l6md-proof/` dir — qqgn precedent). Your "red" is a new focused
  unit test pinning the SHARED surface's behavior BEFORE the unification,
  written against the current internal API locations (slice-1's gheu
  precedent `f6aeabd`/`964d8a5`, slice-2's qqgn precedent `1a8788d`/`ea17b6b`).
- Read, in order: `openspec/changes/add-evidence-audit/design.md` (the
  slices decision — refactor as its own red→green cycle), `tasks.md`
  §Slice 3 (your task is 3.3 ONLY), then the two duplication sites:
  - `src/audit.rs`: `declared_model_ids` (states + transitions +
    constraints + properties ids; intent id deliberately NOT a model
    element — a fragment citing only the intent id binds nothing) and
    `cites_declared_id` (whole-token substring-free match), landed by edrv
    (19f6589/e337e10, tasks 3.1-3.2).
  - `src/lint/graph.rs`: per-file state-id sets (`states.iter().map(...)`,
    ~:123) feeding the transition→state connectivity edges (the
    every_state_used / every_transition_valid machinery), and `rows()`
    (~:452) collecting constraints/properties/states rows for graph
    projections.
- `wai search "declared-ids"`, `wai search "binds_nothing"` — the edrv run
  records (its RO5U findings M1 and L8 were explicitly DEFERRED to this
  ticket's scope — see What to build).
- Start your tdd-ro5 pipeline and execute each step's prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-l6md: shared declared-ids index"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`.

## What to build

tasks.md 3.3 (slice 3 refactor): share the declared-ids index with lint.

- Extract ONE shared declared-ids/citation surface so audit's
  `declared_model_ids` and lint's per-file state-id/row-id collections
  derive from a single implementation instead of parallel walks. Placement
  is your call (a `pub(crate)` helper in an existing shared module — the
  crate is flat; audit already calls `crate::compile::*`) — but both
  consumers must call the one surface.
- Mind the semantic DIFFERENCES you are unifying around, never flattening:
  audit's set includes transition ids and deliberately excludes the intent
  id; lint's state checks use states only and its `rows()` covers
  constraints/properties/states for graph projections. The shared surface
  should expose the pieces (or one full set consumers filter), with each
  consumer's behavior byte-identical before/after.
- RO5U deferrals from edrv, now IN SCOPE: M1 (the bind fold in `run()`
  reuses the `with_pack_rows` name — rename/restructure so the shared fold
  reads honestly) and L8 (CLI helper dedup flagged in the edrv run). Both
  are `src/audit.rs`-internal tidy items; zero behavior change still pins.
- ZERO behavior change: `spk lint` findings and `spk audit` envelopes
  byte-identical before/after on fixed fixtures (record the diff in your
  notes; use your own proof dir).
- Do NOT touch the vacuity check semantics (`cites_declared_id` stays
  whole-token substring-free), the class closed set, exit contracts,
  schema versions, or any rendering.
- Do NOT start 4.x (rung aggregate = 37hl/6hou) or 5.x (x0wy).
- tasks.md checkbox 3.3 only.

## Hard scope guard

- Allowed files: `src/audit.rs`, `src/lint/graph.rs`, `src/lint/mod.rs`
  (only if the shared surface's home lands there or a test pins it),
  a new shared-module file if you justify it (file-headers convention),
  `openspec/changes/add-evidence-audit/tasks.md` (checkbox 3.3 only).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template,
  `src/conform.rs`, `src/main.rs`, `src/lib.rs`, `src/kernel.rs`,
  `src/verify.rs`, `src/parse.rs`.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; the orchestrator closes
  the ticket — you do NOT run `bd close`.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive — author them in the
  archive commit, never mid-change.
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers skill convention).
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.
- Test runner during iteration: `just test-smart` (falls back to
  `cargo test`); full `just ci` gate before you report done.
  **`export TMPDIR=/var/tmp/specodelic-xch1` (create it; it exists) is
  MANDATORY for every cargo run** — /tmp tmpfs OOM-killed cargo twice in
  the 8ebr run; don't wait for it to bite you.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `just test` → <result>
- `ah check` → <result>
- `openspec validate --all --strict` → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">
