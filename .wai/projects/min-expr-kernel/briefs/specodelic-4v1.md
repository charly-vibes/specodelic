# Subagent brief: specodelic-4v1 — p1 errors: failures ride success-shaped envelopes — refuted claim exits 0 (F1)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-4v1` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: `specs/errors.md` (the exit-code mapping and
  envelope_error_kind rows — these ARE the normative truth for this ticket),
  `specs/CHANGELOG.md` #116 (counterexample_found declared a failing
  aggregate outcome), `specs/model_check.md` and `specs/verify.md`
  (failure terminal Models), `specs/orchestrate.md`.
- `wai search "exit code envelope"` — check accumulated patterns.
- This ticket maps to a tdd-ro5 run. Start it and execute each step's
  prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-4v1: failures ride success-shaped envelopes (F1)"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`. If the ephemeral spawn dies, re-spawn with a named
  persisting session (known gotcha).

## What to build

Every case in the ticket's symptom table must exit per the
`specs/errors.md` mapping (`exit_code_mapping`, `envelope_error_kind` rows):
failures carry `ok:false` / `envelope_kind:error` (or a findings-shaped
envelope the spec sanctions), and a refuted kernel claim exits 1. The corpus
already mandates this — the code is wrong, not the spec. **Design direction
is decided: ok means "run succeeded" — fix exit/envelope emission.** Do NOT
re-open the body's open question as a corpus reconciliation.

Symptom cases (re-verified at bd6585b on the quick-start spec from
specs/USAGE.md §1):

1. `spk model-check` with a **refuted** kernel claim: exit **0**, JSON
   ok:true, outcome: counterexample_found. The sharpest case — CHANGELOG
   #116 already declares counterexample_found a failing aggregate outcome.
2. `spk compile` with a kernel-grammar failure: exit 1 but ok:true /
   envelope_kind:ok, files_failed: 1.
3. model-check stale, verify blocked, orchestrate verify-stage failure:
   same shape — non-zero exit riding a success envelope.

Fold in F7 documentation only as prose in the docs you are allowed to touch
(see scope): verify's scope digest binds inputs, not result statuses —
forged check.json statuses yield model gate state: clean. Document this in
the command docs if it fits the touched files; do NOT edit `specs/`.

RED first: add failing CLI fixtures covering each symptom case before
fixing emission. Meter: re-run the body's F1 repro sequence — any case
still printing exit 0 with ok:true = not done.

ANTI-GOALS (from the ticket): do not weaken findings emission or the
envelope warnings/hints fields; do not make batch commands exit 0 to hide
partial failure; exit codes may only move toward the specs/errors.md
mapping, never away from it.

## Hard scope guard

- Allowed files: `src/commands/corpus.rs`, `src/commands/model_check.rs`,
  `src/verify.rs`, `src/orchestrate.rs`, new/updated tests under `tests/cli/`
  (e.g. `tests/cli/model_check.rs`, `tests/cli/compile.rs`,
  `tests/cli/orchestrate_verify.rs`), `docs/src/commands.md` (F7 prose only),
  `specs/CHANGELOG.md` entry at wrap if warranted.
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus — including `specs/errors.md`; it is already
  normative-correct), `.wai/resources/**`, this template.
- Respect shrink-only ratchets: run `pretender check` before committing; if
  a pinned source file breaches, move new tests to `tests/` rather than
  raising entries. `src/model_check.rs` is pinned (see `pretender.toml`) —
  new model-check tests go in `tests/` integration files.
- If you genuinely believe a corpus edit is required to make the fix land,
  STOP and report it in Deviations instead of editing.

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
- **/tmp quota gotcha**: use `TMPDIR=/var/tmp/spk-4v1` for test runs if
  disk-quceeded errors appear.

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