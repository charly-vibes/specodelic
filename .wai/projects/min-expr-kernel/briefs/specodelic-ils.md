# Subagent brief: specodelic-ils — view scripts: type-based hint routing, Mermaid label escaping, kernel_grammar docstring (B1/B2/B4)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-ils` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: `scripts/graph_views.py` (~140, B1 hint substring
  routing), `scripts/view_common.py` (~184, B2 `mermaid_escape` only
  replaces `"`), `tests/kernel_grammar.rs` (~5, B4 docstring),
  `specs/compile.md` (`kernel_expr_opt_in` — the normative marker-based
  opt-in), and the add-graph-views design D8 (deterministic text output).
- `wai search "graph views scripts"` — check accumulated patterns
  (add-graph-views landed via the gre epic; D2/D3/D8 decisions may be
  referenced).
- This ticket maps to a tdd-ro5 run. Start it and execute each step's
  prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-ils: view scripts B1 hint routing, B2 mermaid escaping, B4 docstring"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`. If the ephemeral spawn dies, re-spawn with a named
  persisting session (known gotcha).

## What to build (post-review scope: B1 + B2 + B4 ONLY)

**B1 — hint routed by substring.** `scripts/graph_views.py:140`:
`hint = LINT_HINT if "lint" in str(error) else …`. An ArtifactInvalid raised
while parsing the lint *envelope itself* ("lint envelope carries no issues
list", view_common.py) contains "lint" and wrongly gets the
fix-your-invariants hint. Use the exception type (already computed one line
above as refused/failure) as the discriminator.

**B2 — incomplete Mermaid escaping.** `scripts/view_common.py:184`
`mermaid_escape` only replaces `"`. Labels embed arbitrary spec-cell text;
`-->`, `#`, `%%`, or a line starting `end` can break or alter the rendered
diagram. Views are generated in CI and published — mild injection surface;
undermines design D8 (deterministic text output). **Mirror the fix into
`src/graph.rs`'s `mermaid_escape` if it shares the gap** (check it — the
Rust side emits mermaid labels too).

**B4 — docstring contradicts spec and tests.**
`tests/kernel_grammar.rs:5` claims "marker-free whole-cell opt-in"; the
spec (`kernel_expr_opt_in`, specs/compile.md) and the tests themselves
require a `**kernel:**` marker. Comment-only fix.

**OUT OF SCOPE (do NOT touch):** B3 (fan-in rule disagreement — moved to
its own design ticket) and B5 (changelog naming — moved to specodelic-s64;
do NOT edit specs/CHANGELOG.md for B5).

Meter (acceptance criteria): new unit tests for the escape cases and hint
routing pass under `python3 -m unittest discover -s scripts`;
`grep -c 'marker-free whole-cell opt-in' tests/kernel_grammar.rs` = 0.

## Hard scope guard

- Allowed files: `scripts/graph_views.py`, `scripts/view_common.py`,
  `scripts/test_*.py` (new/updated python unit tests),
  `src/graph.rs` ONLY for the mermaid_escape mirror if it shares the B2 gap
  (new Rust unit tests for escape cases go in the existing graph test
  module or `tests/cli/`), `tests/kernel_grammar.rs` (comment-only),
  `docs/src/commands.md` only if user-facing semantics change.
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus — including `specs/CHANGELOG.md` for B5),
  `.wai/resources/**`, this template.
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
- **/tmp quota gotcha**: use `TMPDIR=/var/tmp/spk-ils` for test runs if
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