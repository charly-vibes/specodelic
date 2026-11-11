# Subagent brief: specodelic-8ebr — audit slice 2: pack-check rows — empty checked-set yields unchecked (tasks 2.1–2.2)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-8ebr` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=specodelic`
- Read, in order: `openspec/changes/add-evidence-audit/design.md` (esp. the
  **verdict vocabulary is audit-local and closed** decision — `unchecked` is
  an audit class layered on conform's verdicts, an audit record carries BOTH;
  plus the read-only discipline: audit never compiles, never writes
  artifacts, never advances a lifecycle transition; `evidence_scope` labeling
  verbatim), `tasks.md` §Slice 2 (your tasks are 2.1–2.2 ONLY), `proposal.md`
  (slice scoping — "pack checked-set honesty": an empty checked-set must
  never read as a pass), and the delta spec
  `openspec/changes/add-evidence-audit/specs/audit/spec.md` (the scenarios
  your rows implement).
- Supersedes closed `specodelic-va3q` (`pack_activation_honest`) — run
  `wai search "pack activation"` and `wai search "pack-check rows"` for prior
  session records before designing.
- Slice 1 (oracle-row classification + report-diff) is fully landed at
  `3c754b8`: `src/audit.rs` already has the taxonomy layered on conform
  verdicts and the extracted `ensure_diffable`/`overlay_class` diff units.
  Rebase before pickup if the branch moved (PRE-002: `src/audit.rs` is the
  single-verb shared surface; slices land sequentially red-to-green).
- Start your tdd-ro5 pipeline and execute each step's prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-8ebr: pack-check rows"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`.

## What to build

tasks.md 2.1–2.2 (slice 2, feature pair; supersession of va3q):

- **2.1 Red**: pack vocabulary + empty checked-set yields `unchecked`,
  countable, never a pass; non-empty checked-set results carry through.
  Write the failing tests FIRST and watch them fail for the RIGHT reason
  (missing pack-row surface / missing `unchecked` classification — a
  compile-error on a not-yet-existing API is an acceptable red if that is
  the honest first failure, as in slice 1; an assertion-fail on existing
  misbehavior is equally fine. Know and record WHY it's red).
- **2.2 Green**: surface pack checker outcomes as audit rows. A checker
  whose checked-set is empty yields the structured `unchecked` verdict —
  never a silent pass; rows count toward the report's totals; non-empty
  checked-set results flow through as their real verdicts.
- Zero change to slice 1 behavior: the existing 14 tests in
  `tests/audit_oracle_rows.rs` stay green throughout; the oracle-row and
  report-diff surfaces are untouched except where the row-emission path
  gains a new row kind.
- Do NOT start 2.3 (row-emission unification refactor — separate ticket),
  3.x, 4.x, or 5.x.
- tasks.md checkboxes 2.1 and 2.2 only.

## Hard scope guard

- Allowed files: `src/audit.rs` (vocabulary + row emission + unit tests),
  `tests/audit_oracle_rows.rs` (CLI-level pins; prefer in-module unit tests
  where possible), `openspec/changes/add-evidence-audit/tasks.md`
  (checkboxes 2.1–2.2 only).
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
