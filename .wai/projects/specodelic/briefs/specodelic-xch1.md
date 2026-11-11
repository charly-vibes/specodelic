# Subagent brief: specodelic-xch1 — audit slice 1: oracle-row classification + --before diff

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-xch1` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=specodelic`
- Read, in order: `openspec/changes/add-evidence-audit/design.md` (esp. all
  four Decision rows — report-diff-not-files, warnings-channel
  `binds_nothing`, rung-derived-not-stored, audit-local closed vocabulary
  layered on conform's), `tasks.md` (Slice 1: tasks **1.1, 1.2, 1.3, 1.4**
  — this ticket's slice; do NOT do 1.5, that is separate ticket
  specodelic-gheu), `proposal.md` (slice scoping + exit-code contract).
- `wai search "conform walk verdict"` and `wai search "evidence_scope"` —
  check accumulated patterns before designing.
- Start your tdd-ro5 pipeline and execute each step's prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-xch1: oracle-row classification + --before diff"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`.

## What to build

New verb `spk audit <file> --oracle <corpus.jsonl>` — Slice 1 only
(oracle/example-row classification + prior-report diff). Reuse
`src/conform.rs` walk machinery unchanged.

- **1.1 RED**: fixture corpus + current model — audit classifies each
  record `{current, unchecked, contradicted}` carrying the underlying
  conform verdict; the report carries `evidence_scope` verbatim +
  `scope_sha256`. Tests must FAIL for the intended reason (verb does not
  exist yet); a zero-test filtered run is not evidence.
- **1.2 RED**: `--before <report.json>` — a record whose class
  dropped/worsened classifies `stale`; records absent from the prior
  report classify `new`; a `scope_sha256` mismatch between reports fails
  labeled (design's digest discipline — never silently diff across corpus
  versions).
- **1.3 RED**: exit contract — 1 iff `contradicted` (or `stale` when the
  prior report declared an expectation); 2 on gate refusal or zero
  records; NEVER nonzero on `unchecked`/`new`.
- **1.4 GREEN**: `src/audit.rs` (walk reuse from conform, unchanged),
  CLI wiring, envelope output via `genesis::guide::Output::emit`.
- Verdict vocabulary is audit-local and closed:
  `{current, stale, unchecked, contradicted, binds_nothing}` — Slice 1
  implements the subset its tests exercise; an audit record carries BOTH
  the conform verdict and the audit class.
- Read-only like conform: never compiles, never writes artifacts, never
  advances a lifecycle transition.
- `src/audit.rs` is a new source file: it MUST carry a
  Purpose/Responsibilities/Rationale header.
- Check the boxes for tasks 1.1–1.4 in
  `openspec/changes/add-evidence-audit/tasks.md` as you complete them.

## Hard scope guard

- Allowed files: `src/audit.rs` (new), `src/main.rs` (CLI wiring only),
  `src/conform.rs` (reuse only — at most visibility-only edits; no
  behavior change), `tests/audit_oracle_rows.rs` (new),
  `openspec/changes/add-evidence-audit/tasks.md` (checkboxes 1.1–1.4 only).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template.
- Respect shrink-only ratchets: run `pretender check` before committing; if a
  pinned source file breaches, move new tests to `tests/` rather than raising
  entries. `src/model_check.rs` is pinned (see `pretender.toml`) — new
  model-check tests go in `tests/` integration files.

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
  (create it; prior sessions needed this).

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
