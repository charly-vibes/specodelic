# Subagent brief: specodelic-m6k — model-check: stale compile demotes a malformed kernel claim to unchecked — silent clean (F2)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-m6k` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: `specs/compile.md` (kernel_nonmember_labeled row — kernel
  claims never fall through unlabeled; labeled kernel-grammar findings are
  "never prose, never silence"), `specs/model_check.md` (staleness/freshness
  semantics — note the recent claim-gate work: scope digest, claim
  classification), `specs/verify.md` (model gate only accepts
  no_counterexample).
- `wai search "stale compile kernel claim"` — check accumulated patterns.
- This ticket maps to a tdd-ro5 run. Start it and execute each step's
  prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-m6k: stale compile demotes malformed kernel claim to unchecked (F2)"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`. If the ephemeral spawn dies, re-spawn with a named
  persisting session (known gotcha).

## What to build

**Bug** (re-verified at bd6585b): after a successful `spk compile`, editing a
constraint cell so it breaks the kernel grammar (e.g. sed the cell to a
non-kernel-grammar expression) and running standalone `spk model-check`
silently demotes the claim to `unchecked` and reports
outcome `no_counterexample` / clean — exit 0. The labeled kernel-grammar
failure path (`src/kernel.rs` ~455-463, `kernel_nonmember_labeled` at
specs/compile.md:93) exists but is never reached because
`src/verify.rs` claim_classification (~line 695) falls through to the
unchecked branch when `extract_model_ir` has no guard_kernel entry for the
cell. model-check's staleness check catches edited *transitions* (exit 1)
but not edited *constraint cells*.

**Fix direction** (from the ticket, confirm against the code): extend
model-check's staleness check to cover constraint cells (e.g. include
structured constraint content in the digest compared pre-check), or fail
labeled when a previously-compiled required claim's cell no longer parses.

**Mitigations already in place** (do not duplicate): verify catches the
edit via scope digest; orchestrate recompiles first. Only standalone
model-check after a stale compile is affected.

RED first: the body's repro — successful compile on the quick-start spec,
sed a kernel-grammar constraint cell to garbage, standalone model-check
must exit 1 with a labeled kernel-grammar finding; exit 0 with
outcome no_counterexample = not done. Write that failing fixture before
fixing.

ANTI-GOALS: do not weaken the freshness gate for *legitimately* stale
artifacts that model-check already catches (transitions); do not make
verify/orchestrate recompile; do not change claim classification for
claims that were never required (unchecked is correct there).

## Hard scope guard

- Allowed files: `src/verify.rs`, `src/kernel.rs`, `src/commands/model_check.rs`,
  new/updated tests under `tests/cli/` (e.g. `tests/cli/model_check.rs`),
  `docs/src/commands.md` only if user-facing semantics change,
  `specs/CHANGELOG.md` entry at wrap if warranted.
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus — `src/model_check.rs` is pretender-pinned, so new
  model-check tests go in `tests/` integration files), `.wai/resources/**`,
  this template.
- Respect shrink-only ratchets: run `pretender check` before committing; if
  a pinned source file breaches, move new tests to `tests/` rather than
  raising entries.
- If you genuinely believe a corpus edit is required to make the fix land,
  STOP and report it in Deviations instead of editing.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains. A foreign
  `pretender.toml` modification and gitignored `vendor/mermaid.min.js` may
  be present in the tree — neither is yours; if the pretender gate fails on
  the vendored mermaid asset, move `vendor/mermaid.min.js` aside for the
  gate run and restore it after (regenerable via `just docs-mermaid`).
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
- **/tmp quota gotcha**: use `TMPDIR=/var/tmp/spk-m6k` for test runs if
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