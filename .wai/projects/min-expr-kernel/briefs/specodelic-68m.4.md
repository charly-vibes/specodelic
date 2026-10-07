# Subagent brief: specodelic-68m.4 — Keep freshness decisions identical across commands

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-68m.4` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: the openspec change's `design.md` (esp. **D3 — Report
  version and freshness** and **D4 — One aggregation implementation**),
  `tasks.md` §2, `proposal.md` (slice scoping), then the delta specs this
  ticket extends.
- `wai search "<topic>"` — check accumulated patterns before designing.
- This is a **TIDY (cleanup-only) ticket**. There is no tdd-ro5 run for it;
  do NOT start one. Do NOT advance any pipeline — the orchestrator owns the
  epic-orchestrator run.

## What to build

tasks.md task **2.3**: "TIDY: separate ticket/commit shares scope encoding
across commands."

Context from the 68m.3 handoff: freshness machinery (canonical scope
encoder, digest computation, D3 report-field injection via
`verify::fresh_report_json`) lives in `src/verify.rs`; model_check and
orchestrate consume it through per-command paths. Design D4's principle —
one implementation per concern — should now extend to the scope encoder.

Concretely:

1. Establish green characterization first: the stale-evidence fixtures from
   68m.3 (old/unrecognized schema, missing/duplicate/extra claims, evaluator
   mismatch, dropped/changed files, reversed order/prose-only digest
   stability, `isolated_scope_required` preflight, swapped-report rejection)
   all pass at HEAD. Record the exact commands and results.
2. Consolidate the canonical scope encoding so model-check, verify, and
   orchestrate share ONE encoder/digest implementation rather than
   duplicated or parallel helpers; consolidate the D3 report-field
   injection helpers where feasible.
3. **Zero behavior change**: accepted inputs, output payloads (including
   digest values), and exit statuses must be byte-identical for all existing
   fixtures. If consolidation would change any payload or digest, STOP —
   that is a semantic change; retag DESIGN and report back instead of
   guessing.
4. Keep the change in a single separate commit (TIDY commit, distinct from
   any feature work).
5. After landing, update ONLY the `2.3` checkbox in
   `openspec/changes/define-verification-claim-gates/tasks.md` (leave every
   other checkbox untouched).

Known constraint from 68m.3: typing the D3 fields as `RunReport` struct
fields would require editing `tests/citation_algebra.rs` and
`tests/differential_reach.rs`, which are **outside this ticket's allowed
files** — do NOT do it; leave the injected-JSON approach in place and note
it for the orchestrator if consolidation still feels incomplete without it.

## Hard scope guard

- Allowed files: `src/model_check.rs`, `src/commands/model_check.rs`,
  `src/orchestrate.rs`, `src/verify.rs`, `tests/cli/model_check.rs`,
  `tests/cli/orchestrate_verify.rs`,
  `openspec/changes/define-verification-claim-gates/tasks.md` (the 2.3
  checkbox only).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template,
  `tests/citation_algebra.rs`, `tests/differential_reach.rs`, `src/compile.rs`,
  `src/graph.rs`, `src/main.rs`, `tests/cli/parse_misc.rs` (last four are
  claimed by a parallel ticket — do not touch).
- Respect shrink-only ratchets: run `pretender check` before committing; if
  a pinned source file breaches, move new tests to `tests/` rather than
  raising entries. `src/model_check.rs` is pinned (see `pretender.toml`) —
  new model-check tests go in `tests/` integration files.
- If consolidation requires touching files outside the allowed list, STOP
  and report — do not expand scope.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files — a
  parallel session may leave residue in the tree; never stage it.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; commit the export after
  `bd close` (`bd export` gotcha: run it or the close is invisible to git).
  — **you may NOT close this ticket**; the orchestrator owns close.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive — author them in the
  archive commit, never mid-change.
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers skill convention). If consolidation extracts a new module,
  it needs a header — but prefer staying inside the allowed file list.
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
