# Subagent brief: specodelic-hb4 — Citations resolve the intended invariant in every command

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-hb4` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: `openspec/changes/add-min-expr-kernel/design.md` (esp. the
  **D9 — Complete command-path evaluation** amendment: bare names local to
  citing file, qualified names exact, never suffix resolution, no implicit
  filesystem discovery, duplicate_corpus_identity invalid, repeated local row
  IDs across distinct ordinary intents valid), `tasks.md` §3.6, then the
  delta specs `specs/model-check/spec.md` and `specs/compile/spec.md`
  (the amendment's additions define the scenarios you pin).
- `bd show specodelic-hb4` — the acceptance criteria and meter are binding.
- `wai search "citation resolution"` — accumulated patterns before designing.
- Study the substrate: `src/model_check.rs` (citation algebra: `parse_citation_expr`,
  `evaluate_citation`, `ThreeValued`, `RunReport.invariant_statuses`),
  `src/commands/model_check.rs`, `src/orchestrate.rs`, `src/compile.rs`
  (`ModelIr.guard_citations`).
- Run the pipeline and execute each step's prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-hb4 (add-min-expr-kernel §3.6): citation resolution in commands"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`.

## What to build

tasks.md §3.6 (RED→GREEN, one cycle):
- CLI fixtures covering: bare and qualified **same-file** citations; two
  ordinary files sharing a local ID; chains; cycles; a missing target; a
  property-row citation. Resolve canonical qualified invariant IDs from
  same-run evidence; **no property execution during model-check**.
- Assert exact statuses in both JSON output and persisted reports.
- Two `id:` spec files sharing a local claim ID with **opposite outcomes**:
  combined command scope fails `isolated_scope_required` **before any
  writes**; independent runs in separate output directories keep their
  outcomes.
- Pin rejection of mixed dual-format/ordinary inputs; rejection of ordinary
  duplicate intent IDs; multi-file lint continues to accept them.
- Ratchet: `src/model_check.rs` is pinned (see `pretender.toml`) — do NOT
  grow it; new model-check tests go in `tests/` integration files.

Binding acceptance criteria (bd specodelic-hb4):
- ≥8 fixtures covering local/qualified parity, same local IDs, chains,
  cycles, missing/property targets, opposite dual-format claims
- Combined `id:` spec inputs fail isolated_scope_required before writes;
  independent runs preserve outcomes; multi-file lint succeeds
- 0 cross-file suffix matches or implicit filesystem discovery
- RED test observed failing for the intended reason before GREEN — record
  commands and results; a zero-test filtered run is not evidence
- Bind new deployed scenarios to genuine tests as each slice lands; do not
  invent bindings or postpone contracts to closure

## Hard scope guard

- Allowed files: `src/model_check.rs` (read-only preferred — ratchet),
  `src/commands/model_check.rs`, `src/orchestrate.rs`, `src/compile.rs`,
  new files under `src/` if you split modules (PREFERRED over growing pinned
  files), `tests/cli/model_check.rs`, `tests/cli/lint.rs`, new `tests/` files,
  `openspec/changes/add-min-expr-kernel/tasks.md` (§3.6 checkbox only),
  `.wai/projects/min-expr-kernel/**`, `.beads/issues.jsonl` via `bd export` only.
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, other changes' directories.
- Respect shrink-only ratchets: run `pretender check` before committing; if a
  pinned source file breaches, move new tests to `tests/` rather than raising
  entries.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; commit the export after
  `bd close` (`bd export` gotcha: run it or the close is invisible to git).
  Note: bd runs embedded — `bd close` writes the jsonl directly.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive — author them in the
  archive commit, never mid-change.
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers skill convention).
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.
- Baseline fact (bd ticket): at 29f81ef a qualified citation resolves unknown
  while the corresponding bare citation verifies — your RED tests should
  reproduce this class of divergence.

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