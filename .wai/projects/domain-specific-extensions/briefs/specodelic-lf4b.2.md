---
tags: [pipeline-run:epic-orchestrator-2026-10-09-specodelic-lf4b-2-add-expected-outputs-to-the-installation-md-quickstart, pipeline-step:spawn-subagent]
---

# Subagent brief: specodelic-lf4b.2 — expected outputs in the installation.md quickstart

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`.
The orchestrator has claimed `specodelic-lf4b.2` and is hands-off. Verify with
real gates — never claim work the diff doesn't contain.

## Task

`docs/src/installation.md` has a first-spec quickstart (`spk new order.cancel`
→ lint → graph → compile → model-check) but shows NO sample output. Add
captured verbatim output after every command block in the Quickstart section,
including at least one REAL lint finding shown and interpreted.

## Staleness gate (mandatory, before editing)

The source reviews used a v0.7.0 snapshot whose outputs already drifted.
Capture ALL outputs fresh at HEAD: build the binary (`cargo build`), run the
full documented sequence in a throwaway dir with the repo-built
`target/debug/spk` (NOT the installed spk if versions differ — check
`git log -1 --format=%h` + `target/debug/spk --version`), and paste verbatim
what it prints. If the docs' command sequence itself drifted from what the
binary accepts, fix the commands to match HEAD behavior first and note it.

## Anti-goals (from the ticket)

- No fabricated/idealized output — only verbatim captures.
- No outputs from commands outside the scaffolded file.
- Do not restructure the section into a command catalog.

## Acceptance meters

1. Every command block in the Quickstart section is followed by captured
   verbatim output; at least one real lint finding is shown AND interpreted
   (one sentence pointing at the finding's rule_id/semantics).
2. Re-running the full documented sequence at HEAD and diffing each captured
   output → zero diffs.
3. `just docs-build` green (mdbook toolchain; if mdbook/mdbook-mermaid are
   not installed locally, install via cargo or record the deviation and rely
   on the unresolved-include check the recipe ends with).

## Hard scope guard

- Allowed files: `docs/src/installation.md` ONLY.
- Never edit: `src/**`, `tests/**`, `specs/**`, `openspec/**`,
  `.espectacular/**`, other docs pages, `justfile`, generated artifacts
  (`docs/src/release.md`, llms.txt), `.wai/resources/**`.
- lf4b.1 landed here earlier today — pull fresh; rebase in commit order.

## Repo facts (boilerplate)

- Commit hygiene: `git status --short` before any commit; stage only files
  YOU authored (`git add <paths>`, never bare `git add`); unstage foreign.
- Non-interactive shells: `-f`/`-rf`/`-y` flags always.
- TMPDIR gotcha: TMPDIR must EXIST before cargo test/build tempfiles
  (TMPDIR=/var/tmp/specodelic-lf4b-2, mkdir -p).
- beads: do NOT close the ticket; do NOT run `wai close`.
- Commit on main, then `git pull --rebase && git push` (this run: you push).

## Report format (end with this)

## Report

**Commits**
- `<hash>` <message> — <one line>

**Gates run**
- sequence re-run diff → <result>
- `just docs-build` → <result>

**Deviations** (or "none")

**Next**
- <next action for orchestrator, or "ticket complete">
