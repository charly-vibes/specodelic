# Subagent brief: specodelic-s64 — docs: v0.6 release-readiness (D1/D2/F9/U1/U2 + ils B5)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-s64` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain. This ticket gates a v0.6 release: docs must match
implemented capabilities per `verify.assurance_views_agree` →
`release_docs_match_implemented_capabilities` (specs/verify.md).

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: `docs/src/commands.md` (the doc under repair — lines ~165
  and ~191 carry the stale claims), `docs/src/graph-views.md` (graph flags
  documented HERE but not linked from commands.md), `README.md`,
  `specs/USAGE.md`, `specs/model_check.md` + `specs/verify.md` (the claim
  gate semantics you are documenting: required/unchecked classification,
  claim-aggregate priority, scope_sha256 evidence binding, stale-report
  rerun hints, isolated_scope_required), `src/compile.rs:537-538` (the
  emitted no-emitter hint strings), `specs/CHANGELOG.md` #114 (stale
  beads-id hint strings — the fix target).
- `wai search "release docs readiness"` — check accumulated patterns.
- This ticket is docs+changelog-only. A tdd-ro5 run is NOT required (no
  production code change); do the work directly with RED-evidence in the
  form of the meter greps (record before/after). If you start a tdd-ro5
  run anyway, advance ONLY via `wai pipeline next`.

## What to build

**D1** — commands.md:165 "no corpus invariant is executable yet (Decision 3,
Option A)" and :191 native backend "never reports no_counterexample" are
both false since the fragment/kernel/claim-gate landings — the native
backend executes kernel claims and reports no_counterexample (reproduced).
Rewrite both passages to describe implemented behavior.

**D2** — claim gates undocumented: document in commands.md's
model-check/verify sections — required/unchecked classification, the
claim-aggregate priority (refuted → counterexample_found; exhausted →
timed_out; unknown/missing → exploration_only; all-verified over completed
exploration → no_counterexample), scope_sha256 evidence binding, stale-report
rerun hints, isolated_scope_required.

**F9/U2** — `--format edges|dot|mermaid` and `--view` wiring: surface them
in the spk graph section of commands.md, README, and USAGE (they currently
live only in docs/src/graph-views.md — link or summarize, don't duplicate
full content).

**U1** — lowercase bold label edge: `**note:**` in fragment position is a
labeled compile failure (spec-correct, closed tag set); add a warning note
in the compile section.

**B5 (absorbed from ils)** — CHANGELOG #114's no-emitter remediation hint
strings name beads ids (specodelic-l8l/aby) but src/compile.rs:537-538 emits
openspec change names (add-py-fragment-emission / add-ts-fragment-emission);
correct the changelog to match the code.

**Meter (acceptance criteria — verify all four greps):**
- `grep -c 'no corpus invariant is executable' docs/src/commands.md` = 0
- `grep -c scope_sha256 docs/src/commands.md` >= 1
- `grep -c 'format edges' README.md docs/src/commands.md` >= 2 total
- `grep -c 'specodelic-l8l' specs/CHANGELOG.md` = 0

## Hard scope guard

- Allowed files: `docs/src/commands.md`, `README.md`, `specs/USAGE.md`,
  `specs/CHANGELOG.md` (B5 correction ONLY — the #114 hint strings; do not
  add a new entry), `docs/src/graph-views.md` (only if cross-linking
  requires it).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/*.md` other than USAGE.md + CHANGELOG.md, `src/**`, `tests/**`,
  `.wai/resources/**`, this template.
- Respect shrink-only ratchets: run `pretender check` before committing.
- If you genuinely believe a corpus edit beyond USAGE/CHANGELOG is required,
  STOP and report it in Deviations instead of editing.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains. A foreign
  `pretender.toml` modification may be present in the tree — not yours,
  never stage it.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`. **Export gotcha**:
  `bd export` writes to STDOUT — after `bd close`, run
  `bd export > .beads/issues.jsonl` and commit the file, or the close is
  invisible to git.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.
- **/tmp quota gotcha**: use `TMPDIR=/var/tmp/spk-s64` for test runs if
  disk-quceeded errors appear.
- **Run FULL `just ci`** — the docs gates (sync-sections,
  summary-completeness, lint-doc-examples) live outside cargo test and are
  exactly the ones this ticket can trip; do not skip them.

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