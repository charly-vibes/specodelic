# Subagent brief: specodelic-gch — Quick-start kernel examples produce expected claim evidence

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-gch` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: the openspec change's `design.md`, `tasks.md` §6 (task
  6.1), `proposal.md` (slice scoping), `specs/USAGE.md` (the examples you
  will migrate), and `specs/claim-guidance.checklist.md` (the binding-path
  guidance from ats).
- `wai search "<topic>"` — check accumulated patterns before designing.
- This ticket **maps to a tdd-ro5 run**. Start it and execute each step's
  prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-gch: quick-start kernel examples produce expected claim evidence"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`. Set `export WAI_PROJECT=min-expr-kernel` first.
- In the tdd-ro5 run, the `refactor` step must be a **no-op**: task 6.3
  TIDY is a separate ticket (specodelic-5c4). Do not bundle cleanup.
- A parallel session is working graph-view tickets (`src/graph.rs`,
  `src/main.rs`, `tests/cli/parse_misc.rs`) — never stage or commit its
  files.

## What to build

tasks.md task **6.1**: "**RED→GREEN**: migrate `specs/USAGE.md` examples to
kernel invariants one at a time; extracted fixtures must execute through
compile/model-check with expected claims."

Concretely:

1. Establish green characterization first: record the current state of
   `just lint-doc-examples && just lint-specs && cargo test --test cli` at
   HEAD.
2. Migrate the `specs/USAGE.md` fenced examples to kernel invariants **one
   at a time**. For each migrated example: extract it as a CLI fixture
   (in `tests/cli/model_check.rs` or the shared fixture corpus per repo
   convention), run it through compile → model-check, and assert the
   expected claim statuses. Missing or wrong claim evidence must FAIL
   before the migration is accepted (RED before GREEN per example).
3. **0 lint baseline additions**: migrations must lint clean without new
   exemptions (`just lint-specs`, `just lint-doc-examples`).
4. Hard gate: 100% of migrated examples lint and produce fresh expected
   claim statuses. Meter: `just lint-doc-examples && just lint-specs &&
   cargo test --test cli`.
5. After landing, update ONLY the `6.1` checkbox in
   `openspec/changes/add-min-expr-kernel/tasks.md` (leave every other
   checkbox untouched).

## Hard scope guard

- Allowed files: `specs/USAGE.md`, `scripts/check_doc_examples.py`,
  `tests/cli/model_check.rs`,
  `openspec/changes/add-min-expr-kernel/tasks.md` (the 6.1 checkbox only).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/**` except `specs/USAGE.md`, `.wai/resources/**`, this template,
  `src/graph.rs`, `src/main.rs`, `tests/cli/parse_misc.rs`,
  `openspec/changes/add-graph-views/**` (parallel ticket's files),
  `src/` generally (this is a corpus+tests ticket; if implementation seems
  required, STOP and report — that's a semantic gap).
- Respect shrink-only ratchets: run `pretender check` before committing;
  `src/model_check.rs` is pinned — new tests go in `tests/` integration
  files.
- If the design requires touching files outside the allowed list, STOP and
  report — do not expand scope.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`. — **you may NOT close
  this ticket**; the orchestrator owns close.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive — author them in the
  archive commit, never mid-change.
- **New source files** need Purpose/Responsibilities/Rationale headers.
- **FORMAT_REVISION note**: migrating corpus content does NOT bump the
  format revision (that's task 7.1, specodelic-bxk) — do not touch
  `src/guide.rs` revision literals.
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
