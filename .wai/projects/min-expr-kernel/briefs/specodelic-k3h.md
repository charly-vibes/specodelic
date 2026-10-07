# Subagent brief: specodelic-k3h — keep claim identity and command results consistent during cleanup

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-k3h` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: `openspec/changes/add-min-expr-kernel/design.md`
  (esp. D9's identical-statuses clause), `tasks.md`
  §"Corrective integration amendment" (task 3.8 — yours), then the
  delta specs this ticket extends.
- `wai search "kernel"` and `wai search "model-check"` — check accumulated
  patterns before designing.
- Your ticket maps to a tdd-ro5 run — start it and execute each step's
  prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-k3h: §3.8 — claim identity and command parity during cleanup"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`.
- Baseline facts: §3.6 (hb4, aa414e9) and §3.7 (mui, 84c4310) are landed and
  pushed. mui's RO5U review deferred two low findings to THIS ticket: the
  corpus pass lives in `src/kernel.rs` (citation_corpus.rs was outside that
  ticket's allowed list) and two ~20-line merge blocks are duplicated across
  the `src/commands/model_check.rs` and `src/orchestrate.rs` call sites.
- This is a **cleanup-only ticket**: establish green characterization
  fixtures FIRST and keep them unchanged; the extraction must be
  behavior-preserving (0 changes to accepted inputs, output payloads, or
  exit statuses). RED here means: characterization/parity tests that pin
  current behavior and pass before the refactor (plus, if the ticket's RED
  convention requires an observed failing state, a parity test must be
  demonstrated to fail against a deliberately broken intermediate — record
  it, then restore).

## What to build

add-min-expr-kernel **§3.8 (TIDY)** — after the §3.7 command fixtures pass,
extract shared identity/evaluation helpers in a separate refactoring work
item and commit. Pin command parity (CLI vs orchestrate vs persisted report).
Specifically:

- Extract the duplicated corpus-claim merge blocks (mui deviation) into one
  shared helper both call sites use.
- Extract shared claim-identity helpers (canonical qualified invariant id
  resolution from hb4 + corpus claim status shapes from mui) so the
  identity rules live in exactly one place.
- Finish before phases 4/5/6 proceed. Proposed claim gates govern aggregate
  acceptance and migration readiness — do not implement them here.
- Pin command parity with characterization tests: same input through the CLI
  command and orchestrate path yields identical statuses, payloads, and exit
  statuses.

Ticket meter (hard gate): 0 changes to accepted inputs, output payloads or
exit statuses — `cargo test`/`just test` must stay green with all existing
assertions intact; the new characterization fixtures pin parity.

## Hard scope guard

- Allowed files: `src/kernel.rs`, `src/commands/model_check.rs`,
  `src/orchestrate.rs`, `tests/cli/model_check.rs`, new files under `tests/`
  (e.g. `tests/claim_parity.rs`), `src/model_check.rs` ONLY if an extraction
  moves code OUT of it (net line shrink — it is pinned at 2300; do not add),
  `openspec/changes/add-min-expr-kernel/tasks.md` (only the §3.8 checkbox).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template,
  `openspec/changes/add-min-expr-kernel/{proposal,design}.md`.
- Respect shrink-only ratchets: run `pretender check` before committing; if a
  pinned source file breaches, move new tests to `tests/` rather than raising
  entries. New parity tests go in `tests/` integration files.
- Agent test runner during iteration: `just test-smart` (falls back to plain
  `cargo test`); the `just ci` gate stays the orchestrator's.

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
