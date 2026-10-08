# Subagent brief: specodelic-68m.5 — Users see the same claim blockers in every report

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-68m.5` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: the openspec change's `design.md` (this ticket is governed
  by **D2 — Aggregate rules**, **D3 — Report version and freshness**, and
  **D5 — Explain assurance levels**: capability table states implemented
  behavior, prerequisites, unchecked content, and pending changes; versions
  derive from Cargo metadata rather than independently maintained literals),
  `tasks.md` §3, `proposal.md` (slice scoping), then the delta specs this
  ticket extends.
- `wai search "<topic>"` — check accumulated patterns before designing.
- This ticket **maps to a tdd-ro5 run**. Start it and execute each step's
  prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-68m.5: users see the same claim blockers in every report"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`. Set `export WAI_PROJECT=min-expr-kernel` first.
- In the tdd-ro5 run, the `refactor` step must be a **no-op**: task 3.3
  TIDY is a separate ticket (specodelic-68m.6). Do not bundle cleanup.
- A parallel session is working graph-view tickets (`src/graph.rs`,
  `src/main.rs`, `tests/cli/parse_misc.rs`) — never stage or commit its
  files.

## What to build

tasks.md tasks **3.1 (RED)** and **3.2 (GREEN)**:

**3.1 RED — parity fixtures first.** Add CLI fixtures asserting that JSON
envelope output, persisted report, and human-readable output state the SAME
claim counts and blocker sets on mixed fixtures (e.g. verified Rust +
refuted kernel, verified Rust + unknown citation, prose-only unchecked).
Also add a docs-consistency check that fails when a version literal or
capability status in the docs conflicts with the release (Cargo) metadata —
D5: versions derive from Cargo metadata, never independently maintained
literals. Observe each RED fixture failing for the intended reason at HEAD
before GREEN; record commands and results. A zero-test filtered run is not
evidence.

**3.2 GREEN — smallest implementation.** Human/JSON/persisted views surface
evaluated, unchecked, and blocking claims consistently (identical blockers
and counts in every view on mixed fixtures — the hard gate). Synchronize
README, `docs/src/status.md`, `openspec/project.md`, `specs/STATUS.md`, and
the embedded guide (`src/guide.rs`) with the implemented capability status.
Document the distinction between specodelic's verification (bounded model
checking of opted-in invariant claims) and application-test execution —
verification results are not a substitute for the application's own test
suite. Keep D2/D3 semantics untouched — this ticket is presentation parity
plus docs, not verdict policy.

## Hard scope guard

- Allowed files: `src/model_check.rs`, `src/commands/model_check.rs`,
  `src/orchestrate.rs`, `src/verify.rs`, `src/human.rs`, `src/guide.rs`,
  `tests/cli/model_check.rs`, `tests/cli/orchestrate_verify.rs`,
  `README.md`, `docs/src/status.md`, `openspec/project.md`,
  `specs/STATUS.md`,
  `openspec/changes/define-verification-claim-gates/tasks.md` (the 3.1 and
  3.2 checkboxes only).
- Note: `specs/STATUS.md` is explicitly allowed BY THIS TICKET (it is repo
  documentation, not a format spec). No other file under `specs/` may be
  touched.
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/**` except `specs/STATUS.md`, `.wai/resources/**`, this template,
  `src/graph.rs`, `src/main.rs`, `tests/cli/parse_misc.rs`,
  `openspec/changes/add-graph-views/**` (parallel ticket's files).
- Docs note: `docs/src/` contains generated copies; the tracked source of
  truth for prose lives at repo root / `specs/`. If a doc edit you make
  under `docs/src/status.md` mirrors another doc, keep both consistent.
- Respect shrink-only ratchets: run `pretender check` before committing; if
  a pinned source file breaches, move new tests to `tests/` rather than
  raising entries. `src/model_check.rs` is pinned (see `pretender.toml`) —
  new model-check tests go in `tests/` integration files.
- If the design requires touching files outside the allowed list, STOP and
  report — do not expand scope.

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
