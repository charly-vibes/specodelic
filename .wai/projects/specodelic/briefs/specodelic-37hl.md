# Subagent brief: specodelic-37hl — audit slice 4: ladder-rung aggregate (tasks 4.1-4.2)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-37hl` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=specodelic`
- Read, in order: `openspec/changes/add-evidence-audit/design.md` (esp. the
  **rung aggregate is derived, not stored** decision — highest rung with
  `current` evidence, computed at report time from audit/conform/verify
  outputs ALREADY PRESENT; no new state, no staleness machinery), `tasks.md`
  §Slice 4 (your tasks are 4.1 + 4.2 ONLY), the delta spec
  `openspec/changes/add-evidence-audit/specs/audit/spec.md` §"Evidence rung
  aggregate" (both scenarios are normative), and the delta layer rows
  `rung_derived` + `honesty_channels` plus property `p_rung`
  (`arbitrary_evidence_mix()`: rung == highest evidence-backed rung ∧ never
  exceeds it).
- Ladder semantics (docs/src/verification-boundaries.md, the five-rung
  ladder this makes machine-readable): 1=lint-clean, 2=well-modeled,
  3=checked-within-bounds (model-check), 4=matches-oracle, 5=better-design.
  The delta pins: a file lint-clean but only model-checked, no verify pass
  ⇒ rung 3, NEVER 4 or 5; no audit/conform/verify evidence at all ⇒ rung 1,
  stated as such. Honesty intersects here: unchecked/binds_nothing evidence
  is countable, NEVER verified — it must not lift the rung.
- Ticket-context note (from bd): slice 4 was made INDEPENDENT of slice 1
  (xch1 DEP fix) — "if red test 4.1 proves audit rows are required, re-add
  the dep". Let the red test decide the wiring surface; the delta spec's
  requirement subject is what audit summaries SHALL report.
- Supersession context: supersedes CLOSED specodelic-4m3y — do not reopen
  or edit it; read it only for design history if useful.
- `wai search "rung"`, `wai search "ladder"` — prior records (the ladder
  docs page landed via lf4b.6; the aggregate supersedes 4m3y's approach).
- `src/audit.rs` at HEAD: slices 1-3 landed (oracle rows, pack rows,
  binds_nothing vacuity via `bind_rows`/`fold_rows`, shared declared-ids
  index in `src/lint/graph.rs`). Report plumbing: `AuditReport`,
  `human_view`, `emit_report_with`, `build_report`, `diff_with_prior`.
- Start your tdd-ro5 pipeline and execute each step's prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-37hl: rung aggregate"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`.

## What to build

tasks.md 4.1 (red) + 4.2 (green): per-file ladder-rung aggregate, 1-5,
derived at report time from evidence actually present — never exceeding it.

RED (4.1) — failing tests first, pinning:
- Mixed-evidence corpus: summaries report per-file rung matching the
  highest evidence-backed rung (model-checked-only file ⇒ 3, never 4/5;
  evidence-free file ⇒ 1, stated as such).
- Never exceeds evidence: a rung without its backing evidence is
  impossible — including unchecked/binds_nothing rows NOT lifting a rung.
- Existing summaries unchanged otherwise: lint and orchestrate outputs stay
  byte-identical on fixed fixtures (this ticket adds a field to audit's
  summary, not a rewrite of other commands' summaries).

GREEN (4.2) — derive rungs from existing outputs at report time:
- Derivation only — NO new stored state, NO staleness machinery for the
  rung itself (design decision). Read the evidence that exists (audit/
  conform/verify outputs already present in the report/walk inputs).
- Schema: `AUDIT_SCHEMA_VERSION` is 1 — adding a derived summary field
  within the delta's declared shape needs no bump, but check
  `parse_prior_report` round-trips and `--before` diffing still behaves
  (the rung is derived, so a prior report's rung should never drive the
  current one's — if prior-parsed rungs complicate the diff, document how).
- Both views: the rung surfaces in the JSON envelope AND `human_view`
  (single rendering path discipline — one formatter rendered from the same
  report values, so the views cannot drift).
- Do NOT start 4.3 (the `just ci` green-gate pass — that is ticket 6hou,
  a separate refactor), or 5.x (x0wy).
- tasks.md checkboxes 4.1 and 4.2 only.

## Hard scope guard

- Allowed files: `src/audit.rs` (derivation + summary field + unit tests),
  `tests/audit_*.rs` (only if a CLI-level pin needs adding — prefer
  in-module unit tests),
  `openspec/changes/add-evidence-audit/tasks.md` (checkboxes 4.1 + 4.2 only).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template,
  `src/conform.rs`, `src/main.rs`, `src/lib.rs`, `src/lint/**`,
  `src/orchestrate.rs`, `src/kernel.rs`, `src/verify.rs`, `src/parse.rs`.
  (lint/orchestrate must stay byte-identical — if a pin seems to require
  touching them, STOP and treat it as a deviation to report, not an edit.)

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
