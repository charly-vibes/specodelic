# Subagent brief: specodelic-lf4b.4 — Add static diagrams: four-layer relationship and lint checker DAG

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-lf4b.4` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=domain-specific-extensions`
- Read: `bd show specodelic-lf4b.4` (acceptance criteria), the epic
  `bd show specodelic-lf4b` (docs-only scope + staleness gate), and
  `docs/src/graph-views.md` (the existing derived-diagrams page — your
  anti-goal reference).
- **Staleness gate (mandatory, epic rule):** re-verify the ticket's claim
  against current main before implementing. `spk explain lint-rules`
  currently enumerates **33 rules** across the linter families listed in
  `docs/src/SUMMARY.md` (frontmatter, schema_shape, ears_syntax, coverage,
  graph_shape, model_shape, referential_integrity, external_completeness,
  failure_shape, observability) — the ticket's "eight-checker" phrasing is
  stale. Derive the DAG content from `spk explain lint-rules` output at
  implementation time, not from the ticket text.
- `wai search "diagrams"` / `wai search "docs"` — check accumulated patterns.

## What to build

Two static Mermaid diagrams in the docs book (mdBook; rendering is ALREADY
wired — `book.toml:20-21` has the `mdbook-mermaid` preprocessor, do not
touch book.toml):

1. **Four-layer relationship** — Intent / Constraints / Model (States +
   Transitions) / Properties within one spec file, showing how rows link
   (traces_to, derives_from, guard citations, observes). Source of truth:
   `specs/specodelic.md` (the corpus core) — read it, then compress.
2. **Lint checker DAG** — the checker families with the **advisory vs
   gating distinction marked**. Advisory rules per `spk explain lint-rules`
   (verify at run time): `linter.observability` (warnings channel, exit 0),
   `linter.skew_advisory` (warnings channel), and the tiered warn-half of
   `linter.single_root_reachable` (cross-file-only rows warn). All others
   gate (exit 1). Model the DAG as the linter's actual pass structure if
   derivable from `specs/linter-*.md`; a clean layered/flow diagram of
   families is acceptable where a true dependency DAG is not derivable —
   say so in the page prose rather than inventing edges.
3. Optional (only if it costs little): verify-workflow diagram.

Placement: one new hand-authored page (e.g. `docs/src/architecture.md` —
pick the name that fits the book's voice) added to `docs/src/SUMMARY.md`.
Do NOT place diagrams inside `docs/src/specs/` or `docs/src/openspec/` —
those are GENERATED copies (gitignored, deployed by docs.yml); hand-authored
pages like `graph-views.md` are the pattern to follow.

Anti-goal (from the ticket): no hand-maintained duplication of the derived
graph views — instead of restating them, link the `spk graph --format
mermaid` recipes (`docs/src/graph-views.md`).

Meter (all must pass):
- `just docs-build` green
- two new mermaid blocks present in the built book (grep `book/` for them)
- diagram content matches `spk explain lint-rules` / `spk explain` output

## Hard scope guard

- Allowed files: `docs/src/architecture.md` (or equivalent new page),
  `docs/src/SUMMARY.md`, and optionally small cross-link edits in
  `docs/src/index.md`.
- Never edit: `openspec/specs/`, `openspec/changes/`, `.espectacular/`,
  `pretender.toml`, `specs/` (the corpus), `book.toml`, `src/**`,
  `tests/**`, `.wai/resources/**`, `docs/src/specs/**`,
  `docs/src/openspec/**` (generated), this template.
- Respect shrink-only ratchets: run `pretender check` before committing; if a
  pinned source file breaches, move new tests to `tests/` rather than raising
  entries. (Docs-only ticket — you should not touch pinned files at all.)

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; commit the export after
  `bd close` (`bd export` gotcha: run it or the close is invisible to git).
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
- `just docs-build` → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">
