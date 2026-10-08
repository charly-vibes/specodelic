# Subagent brief: specodelic-7yk — Core format invariants execute as declared kernel claims

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-7yk` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: the openspec change's `design.md`, `tasks.md` §6 (task
  6.2), `proposal.md`, `specs/specodelic.md` (the file you will migrate),
  `specs/claim-guidance.checklist.md`, and the gch migration commits
  (`aa21795`, `ddf80a1`) for the established migration shape.
- `wai search "<topic>"` — check accumulated patterns before designing.
- This ticket **maps to a tdd-ro5 run**. Start it and execute each step's
  prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-7yk: core format invariants execute as declared kernel claims"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML.
  Set `export WAI_PROJECT=min-expr-kernel` first.
- In the tdd-ro5 run, the `refactor` step must be a **no-op**: task 6.3
  TIDY is a separate ticket (specodelic-5c4). Do not bundle cleanup.
- A parallel session is working graph-view tickets (`src/graph.rs`,
  `src/main.rs`, `tests/cli/parse_misc.rs`) — never stage or commit its
  files.

## What to build

tasks.md task **6.2**: "**RED→GREEN**: migrate `specs/specodelic.md`
invariants to kernel expressions per-file; extracted fixtures must produce
fresh report evidence with revision discipline."

Concretely:

1. Establish green characterization first: `just lint-specs && cargo test
   --test cli` at HEAD (record results).
2. Migrate **eligible** `specs/specodelic.md` equational and
   bounded-quantified invariants to kernel expressions **per-file** (one
   commit per file where sensible). Follow the gch precedent: kernel rows
   carry coverage property rows so lint stays clean with ZERO exemptions;
   cells whose kernel evaluation would produce an honest counterexample
   (domain-data facts) stay informal — record the eligibility decisions.
3. For each migrated cell: extract a CLI fixture asserting the fresh report
   shows the expected status. Missing/wrong claim evidence FAILs before the
   migration is accepted (RED before GREEN per file).
4. **Revision discipline**: migrating the corpus content may require a
   corpus revision bump under the format's Revision rules — check
   `specs/AGENTS.md` and the recall env facts. If a revision bump is
   needed, it touches `src/guide.rs` (FORMAT_REVISION const) and the
   `'specodelic.md Revision N'` literals in `tests/cli.rs` (version skew
   fixtures) — those ARE allowed for this ticket when the bump is required;
   otherwise do not touch them. Note: task 7.1 (bxk) is a separate FORMAT_
   REVISION chore — if your migration requires only the corpus-header
   revision, do the minimal bump the revision rules demand and leave 7.1's
   scope intact. If you are unsure whether a bump is needed, STOP and
   report rather than guessing.
5. **0 lint baseline additions**; hard gate: 100% of migrated cells appear
   in fresh reports with expected status. Meter: `just lint-specs && cargo
   test --test cli`.
6. After landing, update ONLY the `6.2` checkbox in
   `openspec/changes/add-min-expr-kernel/tasks.md`.

## Hard scope guard

- Allowed files: `specs/specodelic.md`, `src/guide.rs`, `tests/cli.rs`
  (only the revision-literal fixtures IF a revision bump is required),
  `tests/cli/model_check.rs`,
  `openspec/changes/add-min-expr-kernel/tasks.md` (the 6.2 checkbox only),
  regenerated compiled-corpus artifacts under `specodelic/` (via just ci).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/**` except `specs/specodelic.md`, `.wai/resources/**`, this
  template, `src/graph.rs`, `src/main.rs`, `tests/cli/parse_misc.rs`,
  `openspec/changes/add-graph-views/**` (parallel ticket's files).
- Respect shrink-only ratchets: run `pretender check` before committing.
  `src/model_check.rs` is pinned — new tests go in `tests/` files.
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
- **Compiled-corpus artifacts** under `specodelic/` regenerate via just ci —
  land them in your feature commit or a separate `chore(artifacts)` commit.
- **New source files** need Purpose/Responsibilities/Rationale headers.
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
