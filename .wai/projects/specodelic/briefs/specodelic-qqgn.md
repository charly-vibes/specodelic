# Subagent brief: specodelic-qqgn — audit slice 2 refactor: unify row-emission code across row kinds (tasks 2.3)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-qqgn` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=specodelic`
- Read, in order: `openspec/changes/add-evidence-audit/design.md` (esp. the
  **verdict vocabulary is audit-local and closed** decision — your
  unification must preserve BOTH-layer semantics exactly: every audit row
  carries the conform verdict AND the audit class, class always
  `AuditClass::of`-derived), `tasks.md` §Slice 2 (your task is 2.3 ONLY),
  and `src/audit.rs` as it stands at HEAD — slice 1 (oracle rows +
  extracted `ensure_diffable`/`overlay_class` diff units) and slice 2
  (`PackCheck`/`PackViolation`, `pack_check_row(s)`, `with_pack_rows`) are
  both landed; the row-emission paths for oracle rows and pack rows are
  what 2.3 unifies.
- `wai search "row-emission audit"` and
  `wai search "pack-check rows"` — prior session records for both slices.
- This is a REFACTOR ticket: the pin is ALL EXISTING tests staying green
  throughout — `tests/audit_oracle_rows.rs` (14), the in-module
  `pack_row_tests` (5) and `diff_tests`, plus the full suite. Your "red"
  is a new focused unit test pinning the UNIFIED emission surface's
  behavior BEFORE the unification (write it against the current internal
  API locations, watch it pass, unify, watch everything still pass) —
  slice-1's gheu refactor precedent (`f6aeabd`/`964d8a5`).
- Start your tdd-ro5 pipeline and execute each step's prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-qqgn: unify row-emission"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`.

## What to build

tasks.md 2.3 (slice 2 refactor): unify row-emission code across row kinds.

- Extract/unify the per-row-kind emission paths (oracle rows vs pack rows)
  into a shared, unit-testable surface — e.g. a single row-construction
  helper parameterized by row key + verdict/class inputs — so adding a
  future row kind (binds_nothing slice 3, rung aggregate slice 4) means
  one call, not a copy of the walk-and-collect shape.
- Preserve semantics EXACTLY: row keys (`pack:<pack_id>:<subject_id>` and
  the oracle-row keys), both-layer verdict+class payloads, count-map
  recomputation (`with_pack_rows`), violation message formatting
  (`rule (message)` verbatim), never-a-pass reason on empty checked-sets.
- ZERO behavior change: audit's envelope output must be byte-identical
  before/after on fixed fixtures (prove with before/after invocations
  compared with `diff`, recorded in your notes — gheu precedent artifacts
  lived under `/var/tmp/gheu-proof`; use your own dir).
- Do NOT change the exit contract, verdict vocabulary, schema version, or
  any rendering. This ticket is unification only.
- Do NOT start 3.x (binds_nothing), 4.x, or 5.x.
- tasks.md checkbox 2.3 only.

## Hard scope guard

- Allowed files: `src/audit.rs` (unification + unit tests),
  `tests/audit_oracle_rows.rs` (only if a CLI-level pin needs adding —
  prefer in-module unit tests),
  `openspec/changes/add-evidence-audit/tasks.md` (checkbox 2.3 only).
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
