# Subagent brief: specodelic-36t — keep shared kernel fixtures stable for the Python follow-up

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-36t` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: `openspec/changes/add-min-expr-kernel/design.md`
  (esp. the D2 interim-agreement row and its deferred-of-record note),
  `tasks.md` §4 (task 4.3 — yours), then
  `openspec/changes/add-py-fragment-emission/design.md` (where the
  promotion path gets documented).
- `wai search "fixture"` and `wai search "agreement"` — check accumulated
  patterns before designing.
- Your ticket maps to a tdd-ro5 run — start it and execute each step's
  prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-36t: §4.3 — shared fixture module + named-case law promotion path"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`.
- Baseline facts: the backend-agreement property (36n, 6cc3159) is wired
  into `just ci` and asserts Rust status equals each fixture's expected
  status; the cross-backend assertion activates with the py emitter
  (change `add-py-fragment-emission`) and must stay uncommented.

## What to build

add-min-expr-kernel **§4.3 (TIDY)**:

1. **Shared fixture module**: move the agreement fixture corpus currently
   duplicated across `tests/kernel_grounding.rs` and
   `tests/kernel_status.rs` into a shared module under `tests/`
   (e.g. `tests/fixtures/` module or `tests/common/` per existing
   conventions — check how other integration tests share code in this
   repo first). All existing assertions keep their meaning; the Rust
   interim gate is NOT weakened.
2. **Document the promotion path**: in
   `openspec/changes/add-py-fragment-emission/design.md`, add the
   documented path for promoting the shared fixtures to a
   `law_requires_cases`-shaped Property row in the l8l change (this is
   the deferred-of-record note tasks.md 4.3 requires). Mark it clearly as
   the promotion contract so the py change can pick it up.
3. Characterization-first: the shared-module move must be
   behavior-preserving — 0 changes to accepted inputs, output payloads,
   exit statuses; `cargo test` green with all existing assertions intact.

Ticket meter (hard gate): `cargo test` green after the move with fixture
meaning preserved; the §4 CI property still runs and passes unchanged.

## Hard scope guard

- Allowed files: `tests/kernel_grounding.rs`, `tests/kernel_status.rs`,
  new shared module under `tests/` (e.g. `tests/fixtures/…` or
  `tests/common/mod.rs` — follow repo conventions),
  `openspec/changes/add-py-fragment-emission/design.md`,
  `openspec/changes/add-min-expr-kernel/tasks.md` (only the §4.3 checkbox).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template,
  `openspec/changes/add-min-expr-kernel/{proposal,design}.md`,
  `openspec/changes/add-py-fragment-emission/` anything other than
  `design.md`, `tests/kernel_agreement.rs` (owned by the 36n property —
  import from it, do not restructure it).
- Respect shrink-only ratchets: run `pretender check` before committing.
  Test files are not ratchet-governed, but do not restructure
  `tests/cli/*`.
- Agent test runner during iteration: `just test-smart` (falls back to
  plain `cargo test`); the `just ci` gate stays the orchestrator's.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; commit the export after
  `bd close` (`bd export` gotcha: run it or the close is invisible to git).
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive — author them in the
  archive commit, never mid-change.
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers skill convention).
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.

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
