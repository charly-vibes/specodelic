# Subagent brief: specodelic-ats — keep external claim guidance on the established binding path

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-ats` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Your ticket maps to a tdd-ro5 run that is ALREADY IN FLIGHT at step 3
  (RED): run id `tdd-ro5-2026-10-07-specodelic-ats-add-min-expr-kernel-ss5-3-external-claim-guidance-binding-path-coverage-lint-turn-ons-no-contract-tomls`.
  The `.last-run` pointer already targets it — advance ONLY via
  `wai pipeline next`; never hand-edit `.wai/pipeline-runs/*.yml`.
- Read, in order: `openspec/changes/add-min-expr-kernel/design.md` (D4 —
  opaque binding, no registry), `tasks.md` §5 (task 5.3 — yours),
  `proposal.md` Tier C scoping, and the landed §5.1/§5.2 work
  (`src/compile.rs` `kernel_binding_of_row`, `tests/kernel_binding.rs`,
  commit 6cb509d) — you document what already landed.
- The plan artifact for your run already exists:
  `.wai/projects/min-expr-kernel/plans/2026-10-07-plan-specodelic-ats-ss5-3-tidy-desired-behavior.md`.
  Follow it; do not re-plan.

## What to build

add-min-expr-kernel **§5.3 (TIDY)** — document the claim path next to the
existing espectacular binding docs (language-tag change precedent):

- The `kernel.binding` claim path (extracted verbatim by compile, surfaced
  through contract-TOML `flags`, never interpreted — design D4) must be
  documented beside the existing two-binding-layers passage in
  `docs/src/commands.md` (the "Two binding layers, never cross-wired"
  section) and `specs/USAGE.md` where external-checker guidance lives.
- Turn coverage lint ON for the affected guidance files (turn-on rows in
  the established checklist mechanism — the language-tag change precedent:
  a language-tagged change turns coverage on for the guidance files it
  touches).
- NO `.espectacular/` edits, NO contract TOMLs (archive-phase concern,
  specodelic-bxk — authoring them mid-change is the known orphan-toml gate
  red). The negative guard (no `*.toml` under `.espectacular/contracts`
  referencing claim guidance) must stay green.
- Update only the §5.3 checkbox in
  `openspec/changes/add-min-expr-kernel/tasks.md`.

**Hard gate (meter):** `just ci` green — 0 changes to accepted inputs,
output payloads, or exit statuses. `just lint-specs` green with the
turn-on rows added and no new advisory class.

## Hard scope guard

- Allowed files: `docs/src/commands.md`, `specs/USAGE.md`,
  `openspec/changes/add-min-expr-kernel/tasks.md` (only the §5.3 checkbox),
  the coverage checklist turn-on rows (find the checklist the
  language-tag precedent used — grep `specs/` and `scripts/` for it).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `src/`, `tests/`, `.wai/resources/**`, this template,
  `openspec/changes/add-min-expr-kernel/{proposal,design}.md`,
  `openspec/changes/*/` other than the tasks.md checkbox.
- `specs/` corpus edits: ONLY the turn-on rows + USAGE.md claim-path
  prose. If a turn-on would require a new advisory class, STOP and
  retag DESIGN rather than improvising (specodelic-04c is the known
  vocabulary-enablement question — do not solve it here, note it).

## RED convention for a TIDY ticket

RED = the failing-state evidence, not a new test: run `just lint-specs`
and the grep guard BEFORE editing, record their output in a
`wai add research "RED: ..."` artifact (coverage not turned on for the
guidance files; claim path undocumented in commands.md/USAGE.md), then
GREEN with the docs + turn-on rows and re-run.

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