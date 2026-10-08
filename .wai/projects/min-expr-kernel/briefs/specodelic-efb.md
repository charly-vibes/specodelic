# Subagent brief: specodelic-efb — acset instance builder panics on member-path (label-qualified) link targets

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-efb` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: `src/acset/instance.rs` (the panic site ~line 126,
  'uninterned node id' inside `Instance::from_specs`), `src/acset/` resolve
  logic (where `[[v.row.deep]]` resolves to a label-qualified member target
  `v.row (deep)`), and the graph build path that interns only defined ids.
- `wai search "member path instance intern"` — check accumulated patterns
  (add-graph-views D2 normalizes display qualifiers at projection — do not
  confuse projection-time normalization with instance-build interning).
- This ticket maps to a tdd-ro5 run. Start it and execute each step's
  prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-efb: acset instance panic on member-path label-qualified link targets"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`. If the ephemeral spawn dies, re-spawn with a named
  persisting session (known gotcha).

## What to build

**Bug** (pre-existing, found in gre.1): a corpus with a member-path link
(e.g. `[[v.row.deep]]` resolving to `v.row (deep)`) panics at
`src/acset/instance.rs:126` with `uninterned node id: v.row (deep)` during
`Instance::from_specs` — graph::build interns only defined ids, but
resolve() can return label-qualified member targets. Reachable via plain
`spk graph` with no flags.

**Fix direction** (ticket offers two — pick deliberately, record why):
intern normalized ids in `from_specs`, or normalize at resolve(). Note
resolve()'s `a.b (c.d)` return shape is pinned by existing unit tests —
touching that contract is a deliberate decision; whichever option you pick,
keep every existing passing test passing (update a pinned test only if the
ticket's chosen fix genuinely requires it, and call it out in Deviations).

RED first: build the fixture from the ticket (file id `v`, rows `row` +
`mem`, `mem` traces_to `[[v.row.deep]]`) and show `spk graph --json <dir>`
panicking before the fix; after the fix it must exit cleanly with a
correct projection. Also cover: the member target genuinely existing vs
dangling (dangling member-path links should stay on their existing labeled
path if one exists — do not invent silent success).

## Hard scope guard

- Allowed files: `src/acset/**` (instance builder + resolve internals),
  `src/commands/graph.rs` only if the fix requires plumbing, new/updated
  tests under `tests/` (CLI graph tests + acset unit tests),
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
- **/tmp quota gotcha**: use `TMPDIR=/var/tmp/spk-efb` for test runs if
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