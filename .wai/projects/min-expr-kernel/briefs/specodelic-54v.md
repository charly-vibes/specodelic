# Subagent brief: specodelic-54v — migrate: mixed-delta (ADDED+MODIFIED) file gets an ADDED-only mirror and immediately fails lint

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-54v` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: `src/migrate.rs` (the D6 doc comment describing the
  ADDED-only mirror), the delta-carrying mirror's lint counterpart
  `specs/linter-*.md` rule `requirement_drift` (per-requirement comparison
  across **all** carried delta sections — since CHANGELOG #93), and
  `specs/migrate.md` (tool semantics the fix must satisfy).
- `wai search "migrate mixed delta"` — check accumulated patterns.
- This ticket maps to a tdd-ro5 run. Start it and execute each step's
  prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-54v: mixed-delta migrate mirror fails lint (ADDED+MODIFIED)"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`. If the ephemeral spawn dies, re-spawn with a named
  persisting session (known gotcha).

## What to build

**Bug** (re-verified at bd6585b): `spk migrate` on a delta carrying both
`## ADDED Requirements` and `## MODIFIED Requirements` writes the mirror as
a byte-exact copy of the ADDED body only — the MODIFIED section is dropped.
Since CHANGELOG #93, `linter.requirement_drift` compares per requirement
across all carried delta sections, so the freshly migrated file is
**guaranteed** to fail lint:

```
printf '## ADDED Requirements\n\n### Requirement: One\none holds\n\n## MODIFIED Requirements\n\n### Requirement: Two\ntwo holds\n' > delta.md
spk migrate delta.md   # exit 0
spk lint delta.md      # exit 1: linter.requirement_drift
```

**Fix direction — decided: option 1 (ticket's preference).** Aggregate
ADDED + MODIFIED bodies into the mirror so it mirrors the per-requirement
lint rule. Do not implement option 2 (refuse labeled) — silent
guaranteed-failure output is the worst outcome and the ticket prefers 1.
Handle each delta section's semantics faithfully in the mirror (ADDED
requirements land as new; MODIFIED requirements land in their modified
form) — if a section type has mirror semantics that cannot be faithfully
aggregated, fail labeled with a remediation hint instead of silently
dropping content, and report the deviation.

RED first: the body's repro must fail (`migrate` exit 0 + `lint` exit 1 on
the migrated file) before the fix; after the fix, migrate then lint on a
mixed delta must exit 0.

## Hard scope guard

- Allowed files: `src/migrate.rs` and any module it delegates mirror-writing
  to (keep it minimal — prefer fixing in place), new/updated tests under
  `tests/cli/` (e.g. a migrate integration test file) or `tests/`,
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
- **/tmp quota gotcha**: use `TMPDIR=/var/tmp/spk-54v` for test runs if
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