# Subagent brief: specodelic-eczv.1 — conform phase 1: verdict engine core (pure classification, no CLI)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-eczv.1` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=domain-specific-extensions`
- Read, in order: `openspec/changes/add-conform/design.md` (decisions D1–D4,
  D6 — the five-valued taxonomy, evidence-class separation, reuse of
  model_check claim types, mechanical name identity D3), `tasks.md` phase 1
  (tasks 1.1–1.5), `proposal.md` (slice scoping), and the delta spec under
  `openspec/changes/add-conform/specs/`.
- `wai search "conform"` and `wai search "model_check claim"` — check
  accumulated patterns before designing.
- Read `src/model_check.rs` ONLY as needed to reuse claim classification
  types (it is pretender-pinned — do NOT edit it; new model-check tests go in
  `tests/` integration files).
- Start and execute the tdd-ro5 pipeline literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-eczv.1: conform phase 1 verdict engine core"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`.

## What to build

Pure verdict engine in `src/conform.rs` (NO CLI wiring — that is phase 4):

- Closed five-valued taxonomy: `permitted | forbidden | underspecified |
  unknown | unsupported` (D1 — distinct values, never collapsed to pass/fail).
- Evidence-class separation (D2): a trace contradicting a declared executable
  claim is `forbidden` in ANY invocation mode (`closed_world: false`, reason
  `contradiction_forbidden_any_mode`); an uncovered trace is `forbidden` only
  with closed-world declared (`closed_world: true`, reason
  `closed_world_forbidden_recorded`), otherwise `underspecified`
  (`uncovered_trace_never_forbidden`).
- Trace classification against the compiled Model's transition relation +
  invariant claims, reusing model_check's claim classification types
  (`required_claims_classified`) — no new status vocabulary (D4; OQ2 stands
  as approved: invariant claims + Model only, laws excluded).
- Mechanical name identity after trimming (D3).
- Taxonomy totality: every corpus trace receives exactly one verdict; fixture
  corpus reaches all five values.

TDD discipline per tasks.md phase 1:

- 1.1 **RED**: unit tests in `tests/conform_classification.rs` for the closed
  taxonomy over a fixture spec + fixture JSONL corpus: executable-claim
  contradiction → `forbidden` with claim id in reason; prose-only covering
  claim → `unknown` naming the claim; no covering claim → `underspecified`;
  unsupported evaluator kind → `unsupported` naming the kind. Run
  `just test-smart` — all new tests must FAIL first (module does not exist).
- 1.2 **GREEN**: implement claim classification consumption in
  `src/conform.rs` reusing model_check's claim types (D4); classify traces
  against the compiled Model's transition relation.
- 1.3 **RED→GREEN**: evidence-class separation cases per D2 (above).
- 1.4 **RED→GREEN**: taxonomy totality — `taxonomy_is_total_and_distinct`
  over a five-value fixture corpus.
- 1.5 **TIDY**: extract verdict-reason formatting into a testable unit; dead
  flag and clippy sweep — as a SEPARATE tidy commit.

Check off tasks 1.1–1.5 in `openspec/changes/add-conform/tasks.md` as you
complete them (checkbox edits only — no other tasks.md changes).

## Hard scope guard

- Allowed files: `src/conform.rs` (new), `tests/conform_classification.rs`
  (new), `openspec/changes/add-conform/tasks.md` (task 1.x checkoffs only),
  `Cargo.toml` ONLY if a module registration genuinely requires it (it
  should not — lib modules are auto-declared).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template,
  `src/model_check.rs`, `src/lib.rs` beyond a single conform module
  declaration if registration is manual.
- Respect shrink-only ratchets: run `just pretender-check` before committing;
  if a pinned source file breaches, move new tests to `tests/` rather than
  raising entries.

## Repo facts (boilerplate — applies to every ticket)

- **TMPDIR gotcha**: `export TMPDIR=/var/tmp/specodelic-eczv` (directory
  already created) before any cargo test run — /tmp quota exhaustion
  masquerades as hundreds of test failures.
- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint. (Phase 1
  is library-only — no CLI surface yet.)
- **beads**: issues live in `.beads/issues.jsonl`; the orchestrator handles
  close/export — you do not touch bd.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive — do NOT author any.
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers skill convention) on `src/conform.rs`.
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
