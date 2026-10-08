# Subagent brief: specodelic-0zk — graph: projection exit-code semantics — bad path silent success, violations exit 0 in text modes (F3+F8)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-0zk` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: `specs/errors.md` (exit_code_mapping row: 0 clean, 1
  findings-or-failure, 2 invocation error — this is the contract), the graph
  command's help text ("Zero spec files exit 0 with empty output" — a
  nonexistent path is NOT zero spec files), `specs/graph.md` (tool
  semantics, design D3: violations ride along as annotation rows), and the
  graph projection mode code paths.
- `wai search "graph exit code projection"` — check accumulated patterns.
- This ticket maps to a tdd-ro5 run. Start it and execute each step's
  prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-0zk: graph projection exit-code semantics (F3+F8)"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`. If the ephemeral spawn dies, re-spawn with a named
  persisting session (known gotcha).

## What to build

**F3** (re-verified at bd6585b): `spk graph /nonexistent --format edges` →
exit 0, empty stdout/stderr. JSON mode exits 2. A typo'd path looks like an
empty corpus. No spec text sanctions silent success on an unreadable path.

**F8**: `spk graph tests/fixtures/typing_violations` → JSON exits 1; the
same fixture with `--format edges` → exit 0 (violations ride along as
annotation rows per design D3, which is correct — but the exit code is
unspecified, so a pipeline cannot gate on violations).

**Fix direction — decided in the ticket**: one decision covering projection
modes — nonexistent/unreadable path → exit 2 (labeled, matching JSON mode
and spk lint); typing violations present in text projections → exit 1
(matching JSON mode), violations still riding along in output.

Meter (acceptance criteria): the two repro lines print 2 and 1
respectively; `graph --format edges` on tests/fixtures/typing_violations
still emits the six annotation rows.

RED first: both repro lines must show the wrong exit codes (0/0) before the
fix.

ANTI-GOALS: do not silently drop violations from projection output to make
exit codes align; violations must still ride along (design D3); do not
change JSON-mode exit codes.

## Hard scope guard

- Allowed files: `src/commands/graph.rs` and any projection-module file the
  fix requires, new/updated tests under `tests/cli/` (e.g.
  `tests/cli/graph_projection.rs` or wherever graph CLI tests live),
  `specs/CHANGELOG.md` entry at wrap if warranted, `docs/src/commands.md`
  only if user-facing semantics change (the --help "Zero spec files" line
  likely needs correcting — it's docs, not corpus).
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
- **/tmp quota gotcha**: use `TMPDIR=/var/tmp/spk-0zk` for test runs if
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