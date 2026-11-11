# Subagent brief: specodelic-gheu — audit slice 1 refactor: extract report-diff into testable unit

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-gheu` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=specodelic`
- Read, in order: `openspec/changes/add-evidence-audit/design.md` (esp. the
  **report-diff-not-files** decision — your extraction must preserve its
  semantics exactly: same class ⇒ `current`, dropped/worsened ⇒ `stale`,
  no prior row ⇒ `new`, `scope_sha256` mismatch ⇒ labeled refusal), and
  `src/audit.rs` (the diff logic this ticket extracts; slice 1 landed as
  commit `a435157`).
- `wai search "report-diff audit"` — prior session records for this slice.
- Start your tdd-ro5 pipeline and execute each step's prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-gheu: extract report-diff into testable unit"`
  This is a REFACTOR ticket: the pin is the EXISTING 14 tests in
  `tests/audit_oracle_rows.rs` staying green throughout; your "red" is a
  new focused unit test for the extracted diff unit that pins its behavior
  BEFORE the move (write it against the current internal API location,
  watch it pass, move the code, watch it still pass).
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML.

## What to build

tasks.md 1.5 (slice 1 refactor): extract the `--before` report-diff into a
testable unit.

- Pull the diff logic out of the CLI/audit path into a standalone function
  (module-level in `src/audit.rs`; `pub(crate)` visibility is fine) with a
  signature decoupled from the envelope/rendering path.
- Add a focused unit-test module for the diff unit: same-class ⇒ current,
  worsened ⇒ stale, new row ⇒ new, digest mismatch ⇒ labeled refusal,
  empty prior report ⇒ all new. Test the UNIT, not the CLI.
- ZERO behavior change: audit's envelope output must be byte-identical
  before/after on a fixed fixture (prove with one before/after invocation
  comparison recorded in your notes).
- Do NOT change the exit contract, verdict vocabulary, schema version, or
  any rendering. This ticket is extraction only.
- Do NOT start tasks 2.1+ (slice 2 is a separate ticket).
- tasks.md checkbox 1.5 only.

## Hard scope guard

- Allowed files: `src/audit.rs` (extraction + unit tests),
  `tests/audit_oracle_rows.rs` (only if a CLI-level pin needs adding —
  prefer in-module unit tests),
  `openspec/changes/add-evidence-audit/tasks.md` (checkbox 1.5 only).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template,
  `src/conform.rs`, `src/main.rs`, `src/lib.rs`.

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
  If cargo hits tmpdir space errors, `export TMPDIR=/var/tmp/specodelic-xch1`
  (create it; it exists).

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
