# Subagent brief: specodelic-7c7 — model_present: make the wording match behavior (no differential promised)

You are a subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-7c7` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=specodelic`
- Read: `bd show specodelic-7c7`;
  `openspec/specs/linter-model_shape/spec.md` (owning corpus spec — its
  `model_sections_paired` constraint is symmetric and promises NO
  differential); `src/lint/mod.rs` RULE_TABLE entry `model_present`
  (line ~90) plus the fixture_corpus model_present fixture (line ~644);
  `src/lint/rules.rs` firing site (lines ~170-177); `tests/cli/lint.rs`
  (existing model_present assertions, if any).
- `wai search "model_present"` and `wai search "model_shape"` — accumulated
  patterns first.
- Start and execute the executor pipeline literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-7c7: model_present differential"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`.
- Set `export TMPDIR=/var/tmp/specodelic-xch1` (mkdir -p it) — MANDATORY
  for all cargo invocations in this repo.
- Run `grep -rn "empty-but-present" src/ tests/ AGENTS.md docs/ 2>/dev/null`
  once to enumerate every site carrying the phrase before editing.

## What to build

The `model_present` finding message ("…both sections must exist
(empty-but-present beats absent)") and the RULE_TABLE semantics string
promise a differential between an absent Model section and an
empty-but-present one. No such differential exists: both parse to empty
`states`/`transitions` vectors and produce the identical finding,
message, and exit 1. Acceptance: **message text matches actual behavior;
fixtures for both cases pinned in lint tests.**

1. **RED first** (tests in `tests/cli/lint.rs` following its existing
   pattern): fixture A — spec file with NO Model section at all; fixture
   B — spec file with a Model section present but empty (States heading
   with no bullets, Transitions table present but empty — or section
   shells with no rows). Assert BOTH produce exactly one `model_present`
   finding with the SAME label and a message that makes no
   differential claim, plus exit-code behavior matching the harness
   pattern used by neighboring tests. The label `model_present` is
   UNCHANGED — downstream consumers (audit rung
   `src/audit/rung.rs`, embedded guide) key on it.
2. **GREEN**: reword so no differential is promised, e.g. semantics:
   `the Model section must contain both a States list and a Transitions
   table`; message: `Model section incomplete: {n} states, {m}
   transitions — both a States list and a Transitions table must exist`.
   You may refine wording; the invariant is: no claim that
   empty-but-present is treated differently from absent, counts still
   reported, remediation value ("both must exist") retained.
   Update BOTH the firing message (`src/lint/rules.rs`) and the RULE_TABLE
   semantics string (`src/lint/mod.rs`) — same claim, one fix. Keep the
   edit surgical: the RULE_TABLE row and its adjacent fixture comment
   only.
3. **Generated catalog**: after the code change is final, refresh the
   AGENTS.md managed rule catalog with `spk init --force` and verify
   `git diff AGENTS.md` is catalog-only (the `linter.model_present`
   bullet tracks the new semantics string). Update any other hit your
   grep found (tests, guide text) — corpus `openspec/specs/**` carries
   NO copy of the phrase, so NO openspec change is expected.
   Exception: if you discover corpus text that genuinely must sync,
   stop and scaffold an openspec change + `just archive-change` route
   instead of hand-editing specs — but expect none.
4. **Iteration testing**: run TARGETED tests only during red-green
   cycles: `TMPDIR=/var/tmp/specodelic-xch1 cargo test --test cli lint_
   -- --nocapture` style filters (and `cargo test --lib lint` if you
   touch mod.rs fixtures). Do NOT run full `just ci`, `ah check`,
   `just lint-specs`, or `openspec validate` — the orchestrator owns
   full gates after your report.

## Hard scope guard

- Allowed files: `src/lint/rules.rs`, `src/lint/mod.rs` (RULE_TABLE
  `model_present` row + its fixture comment only), `tests/cli/lint.rs`,
  new fixture tests under `tests/**`, `AGENTS.md` ONLY via
  `spk init --force`.
- Never edit: `openspec/**` (no spec delta in scope), `specs/` (domain
  corpus), `.espectacular/**`, `pretender.toml`, `.wai/resources/**`,
  `.beads/hooks/**`, run YAMLs, `src/guide.rs` catalog wording beyond
  what regeneration requires.
- Respect shrink-only ratchets: run `pretender check` before
  committing; if a pinned source file breaches, move new tests to
  `tests/` rather than raising entries.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`,
  run `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`. Do NOT claim/close/
  update the ticket — the orchestrator owns bd state for this run.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`.
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers skill convention).
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may
  be aliased with `-i` and will hang on prompts.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Tests run**
- targeted RED/GREEN cycles → <results>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">
