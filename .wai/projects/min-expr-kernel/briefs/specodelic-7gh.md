# Subagent brief: specodelic-7gh — model-check: claim records omit reason when not verified — dangling reachable yields unlabeled unknown (F4/F6)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-7gh` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: `specs/model_check.md` line ~35 (`claim_report_schema` —
  records carry "id, evaluator kind, status, reason where not verified") and
  ~76 (`unknown_or_missing_claim_blocks_clean` — reasons name the claim ids),
  `src/kernel.rs` (native evaluation returns `ThreeValued::Unknown` with
  `reason: None` for evaluation errors ~872-892 and ~1090-1115; reasons
  attach only on the TLC path and the re-parse-failure fallback).
- `wai search "claim reason unknown"` — check accumulated patterns. Note the
  4v1 landing (as_failure helper, refuted claims exit 1) and m6k landing
  (kernel_grammar pre-check) — related surface, do not regress either.
- This ticket maps to a tdd-ro5 run. Start it and execute each step's
  prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-7gh: claim records carry reason when not verified (F4/F6)"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`. If the ephemeral spawn dies, re-spawn with a named
  persisting session (known gotcha).

## What to build

**Bug** (re-verified at bd6585b): a kernel claim with status unknown
persists exactly `{"evaluator":"kernel","id":"refund_traces_resolve",
"status":"unknown"}` — no reason. Human output reads
`refund_traces_resolve (kernel, unknown)` with no explanation. Repro:
compile the quick-start spec from specs/USAGE.md §1 (file must be named
`order-cancel.md` per the dash⇔dot id rule), sed the kernel constraint cell
to `reachable(order.cancel.nope, order.cancel.refund_bounded, guard)` — a
dangling endpoint lints and compiles, then evaluates to unknown, downgrading
the run to `exploration_only` with nothing pointing at the bad id.

**Expected** (spec, already normative): records carry a reason wherever
status is not verified, and reasons name the claim ids.

**Cause**: native evaluation returns Unknown with reason None for
evaluation errors; reasons attach only on TLC path and re-parse-failure
fallback.

**The unknown itself is deliberate** ("never a fabricated verdict") — the
fix is attaching the reason, NOT changing the verdict semantics. Keep
status semantics identical; keep exit codes identical to post-4v1 state.

**Also verify while here** (flagged UNKNOWN in the ticket): whether
counterexample records carry reason — if they do, leave alone; if not, fix
consistently and say so in the report.

Diagnosability asymmetry to preserve as-is: mistyped morphism in
resolves(...) gets a labeled compile error (m6k pre-check); the dangling
endpoint in reachable(...) gets unknown — now with a reason naming the bad
id.

RED first: the ticket's repro must show a reason-less unknown record before
the fix.

## Hard scope guard

- Allowed files: `src/kernel.rs` (reason plumbing on native evaluation
  paths), `src/commands/model_check.rs` / `src/verify.rs` only if report
  assembly needs plumbing, new/updated tests under `tests/cli/` (model_check
  tests — `src/model_check.rs` is pretender-pinned, tests go in `tests/`),
  `specs/CHANGELOG.md` entry at wrap if warranted, `docs/src/commands.md`
  only if user-facing semantics change.
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template.
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
- **beads**: issues live in `.beads/issues.jsonl`. **Export gotcha**:
  `bd export` writes to STDOUT — after `bd close`, run
  `bd export > .beads/issues.jsonl` and commit the file, or the close is
  invisible to git.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive — author them in the
  archive commit, never mid-change.
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers skill convention).
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.
- **/tmp quota gotcha**: use `TMPDIR=/var/tmp/spk-7gh` for test runs if
  disk-quceeded errors appear.
- **Run FULL `just ci` (not just `just test`)** — the views-purity python
  suite (`just sync-sections-test`) lives outside cargo test and caught a
  0zk integration miss; do not skip it.

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