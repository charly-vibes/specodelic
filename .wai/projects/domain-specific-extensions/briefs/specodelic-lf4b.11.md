# Subagent brief: specodelic-lf4b.11 — Sync stamped machine artifacts (release.md, llm.txt) for repo readers

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-lf4b.11` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=domain-specific-extensions`
- Read: `bd show specodelic-lf4b.11` (acceptance criteria + anti-goals),
  the epic `bd show specodelic-lf4b`, `scripts/stamp_llms.py` (the stamp
  pipeline — note its `main()` takes `page|llms` and has `tag`/`sha`
  injection params), `scripts/test_stamp_llms.py` (its tests — the mvl
  must-stay-gone guard lives here), `docs/src/release.md` (stale: shows
  v0.6.0-era content per the ticket), repo-root `llm.txt` (43 lines, the
  stale narrative summary) vs `llms.txt` (123 lines, deploy target),
  `justfile` (where stamp targets/hooks live — grep `stamp`), and
  `.github/workflows/docs.yml` + `release.yml` (where stamping is
  invoked at build/release time).
- **Banked facts (do not re-derive):** (1) llm.txt = repo-root narrative
  summary REQUIRED at root by the DDL conformance check
  (`s2_llms_txt_at_root`-class rule, specodelic-mvl); its cke-era
  must-stay-gone guard was REVERSED — llm.txt is restored policy, llms.txt
  stays the deploy target with merged Install & Links. (2) llm.txt is NOT
  deployed by docs.yml (repo policy). (3) docs.yml deploys only from main;
  release.yml tag push = release authorization.
- **Staleness gate (mandatory):** re-verify every claim above against
  current main — the workflows and stamp script may have changed since.
- `wai search "llm.txt"` / `wai search "stamp"` / `wai search "release"` —
  accumulated patterns.

## What to build

Fix the STAMP PIPELINE, not the artifacts (anti-goal: never hand-edit
generated files):

1. **`docs/src/release.md` stale version**: it is tracked in git and
   regenerated only at docs-build time, so repo browsers (and RAG-style
   fetchers) see a two-releases-old page that says "do not hand-edit".
   Per the ticket: commit-stamp at release time OR replace the checked-in
   file with a placeholder that points at the built book. Pick the
   mechanism that (a) keeps `grep release.md for version == Cargo.toml
   version at HEAD` true at every commit point, and (b) doesn't fight the
   existing docs-build flow. Document the choice in the script/docstring.
   Likely shape: make `release.md` a small generated-at-release-time
   artifact committed by release.yml, with the checked-in default being a
   redirect/placeholder page — verify what release.yml can actually do
   before committing to it (workflow edits are allowed if small; do not
   create new workflows).
2. **`llm.txt` stale content**: extend the stamp pipeline so `llm.txt` is
   regenerated from the current corpus at stamp time — its missing
   sections (graph-views, dual-format, hooks, archive-companion) must
   appear, sourced from the same content that feeds `llms.txt` (123
   lines) so the two can't silently diverge again. Cross-link llms.txt
   (or make the two collision-proof) per the ticket — an agent grabbing
   the wrong summary must be able to tell them apart.
3. Wire so this CAN'T rot again: the natural point is the existing
   stamp invocation path (docs-build / release flow). If a `just` target
   can regenerate both cheaply, add `just stamp-artifacts` (or wire into
   an existing target) and run it as part of this change's commit so the
   regenerated artifacts land in this ticket.

**Meter (all must pass):**
- `grep release.md for crate version == Cargo.toml version at HEAD`
- `grep llm.txt for graph-views and dual-format` (both present)
- `just docs-build` green
- llm.txt cross-links or disambiguates llms.txt
- existing stamp tests (`scripts/test_stamp_llms.py`) updated for any
  pipeline change and green

**Anti-goals:** no hand-editing generated files; do NOT delete llm.txt
(mvl guard); do NOT touch `openspec/`.

## Hard scope guard

- Allowed files: `scripts/stamp_llms.py`, `scripts/test_stamp_llms.py`,
  `justfile`, `.github/workflows/release.yml` and/or `docs.yml` (small
  edits only, if the mechanism needs it), `docs/src/release.md`,
  `llm.txt` (regenerated via the pipeline, committed), and `book/llms.txt`
  if the stamp writes it (note: `book/` is gitignored — generated at
  docs-build).
- Never edit: `openspec/**`, `.espectacular/`, `pretender.toml`, `specs/`
  (the corpus), `src/**`, `tests/**`, `.wai/resources/**`,
  `docs/src/specs/**`, `docs/src/openspec/**` (generated), `llms.txt`
  (source of truth for the deploy artifact — only the stamp touches its
  OUTPUT in book/), this template.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
- **beads**: `bd close specodelic-lf4b.11` FIRST, then `bd export -o
  .beads/issues.jsonl`, then commit the export.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags.
- **Scratch work outside /tmp**: use `/var/tmp/<slug>`.
- **Quality gates before finish**: `just test` must be green (full green
  is the bar since a59448e); `pretender check` exit 0 (script role
  thresholds: cyc 15, cognitive 31, function_lines 59, file_lines 300 —
  keep edits inside them; split files if a script would breach).

## Mechanical finish checklist (execute in order, do not skip)

1. Meter greps at final HEAD (paste evidence): release.md version ==
   Cargo.toml; llm.txt has graph-views + dual-format + cross-link
2. `python3 scripts/test_stamp_llms.py` → green (updated tests)
3. `just test` → green; `pretender check` → exit 0
4. `just docs-build` → green
5. `git status --short` → only your files; stage explicitly
6. `git commit` with honest attribution
7. `bd close specodelic-lf4b.11` → `bd export -o .beads/issues.jsonl` →
   commit the export
8. Do NOT push; do NOT run `wai close`
9. End with the Report block below

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `just test` → <result>
- `just docs-build` → <result>
- `ah check` → <result>
- `openspec validate --all --strict` → <result>

**Mechanism chosen**
- <release.md: committed-stamp vs placeholder and why; llm.txt: how
  regenerated + divergence-proofed>

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">