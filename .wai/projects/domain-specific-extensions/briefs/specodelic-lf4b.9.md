# Subagent brief: specodelic-lf4b.9 — Document tool relationships: OpenSpec dual-format and the ah companion CLI

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-lf4b.9` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=domain-specific-extensions`
- Read: `bd show specodelic-lf4b.9` (acceptance criteria — note it is
  labeled `constraint-locked`), the epic `bd show specodelic-lf4b`, and the
  HARD CONSTRAINT in this repo's `AGENTS.md` § Sibling-tool constraints
  (espectacular section).
- **Sources for the content (verify all against current main):**
  - `spk explain dual-format` — the explain topic the section must surface
  - `specs/AGENTS.md` (repo workflow: file naming, revision discipline)
  - The repo `AGENTS.md` espectacular section: espectacular is READ-ONLY
    over `openspec/specs/` + `openspec/changes/`; spec↔test correspondence
    via contract TOMLs in `.espectacular/<capability>/`; `ah check` /
    `ah doctor` / `ah explain`
  - The espectacular README for the companion CLI's self-description —
    find its repo path (sibling under `~/para/areas/dev/gh/charly/espectacular`)
    and link the upstream README URL, do NOT inline its whole content
- **Audience:** outsiders who know OpenSpec but not specodelic (Z.ai #6).
  Explain: how specodelic's format relates to OpenSpec change workflow
  (dual-format spec files: openspec deltas AND standalone specodelic
  specs), and where the ah/espectacular companion fits (spec↔test
  correspondence checking) — without describing enforcement behavior as
  specodelic-owned (anti-goal).

## What to build

A docs section explaining the tool relationships:

- **Placement:** a new page `docs/src/tool-relationships.md` (or equivalent
  name fitting the book's voice) linked from `docs/src/SUMMARY.md`; plus a
  short pointer from `docs/src/index.md` if natural.
- Content must cover:
  1. The OpenSpec dual-format relationship — surface `spk explain
     dual-format` (link/quote the command so readers run it themselves)
  2. The ah/espectacular companion CLI role: spec↔test correspondence via
     `ah check`; state the read-only boundary honestly (espectacular never
     edits openspec/, enforcement lives elsewhere)
- Meter: the section links the dual-format explain topic AND the
  espectacular README; `just docs-build` green.

**Anti-goals (constraint-locked):**
- **NO edits under `openspec/`** — read-only per sibling-tool rules
  (verify step will diff-check this)
- no enforcement-behavior changes to ah described as specodelic-owned
- documentation only; all edits in `docs/src/` + README cross-link at most

## Hard scope guard

- Allowed files: new page under `docs/src/`, `docs/src/SUMMARY.md`,
  optionally small cross-link edits in `docs/src/index.md`.
- Never edit: `openspec/**` (HARD), `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `book.toml`, `src/**`, `tests/**`,
  `.wai/resources/**`, `docs/src/specs/**`, `docs/src/openspec/**`
  (generated), this template.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
- **beads**: `bd close specodelic-lf4b.9` FIRST, then `bd export -o
  .beads/issues.jsonl`, then commit the export.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags.
- **Scratch work outside /tmp**: use `/var/tmp/<slug>`.
- Hand-authored docs pages carry no Purpose/Rationale header comments.

## Mechanical finish checklist (execute in order, do not skip)

1. Confirm `git diff main --stat` (uncommitted) / commit diff touches NO
   path under `openspec/` — paste evidence in report
2. `spk explain dual-format` referenced in the section (link or quoted
   command) — grep evidence
3. `just docs-build` → green
4. SUMMARY.md link present
5. `git status --short` → only your files; stage explicitly
6. `git commit` with honest attribution
7. `bd close specodelic-lf4b.9` → `bd export -o .beads/issues.jsonl` →
   commit the export
8. Do NOT push; do NOT run `wai close`
9. End with the Report block below

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `just docs-build` → <result>
- `ah check` → <result>
- `openspec validate --all --strict` → <result>

**Constraint evidence**
- <proof no openspec/ path touched; dual-format explain topic + espectacular
  README links present>

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">