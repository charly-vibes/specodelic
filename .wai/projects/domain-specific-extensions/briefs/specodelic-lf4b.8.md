# Subagent brief: specodelic-lf4b.8 — Add 'when to use this command' context to command reference

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-lf4b.8` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=domain-specific-extensions`
- Read: `bd show specodelic-lf4b.8` (acceptance criteria), the epic
  `bd show specodelic-lf4b`, `docs/src/commands.md` (the target page),
  and `docs/src/glossary.md` (lf4b.7 — link it for jargon instead of
  redefining).
- **Staleness gate (mandatory, epic rule):** every when/why note must match
  current main. Source of truth: `spk explain` topics (esp. `lifecycle`),
  `spk <verb> --help`, and the corpus files (`specs/compile.md`,
  `specs/verify.md`, `specs/model_check.md`, `specs/rename.md`,
  `specs/merge.md`, `specs/graph.md`, `specs/refactor.md`,
  `specs/orchestrate.md`). Verify verb behavior by running the binary where
  cheap (e.g. `spk migrate --help` for the --rekey semantics).
- `wai search "commands"` / `wai search "docs"` — accumulated patterns.

## What to build

Context notes in `docs/src/commands.md` only — **do not restructure the
book around commands** (anti-goal per review DRAFT-002; this page stays
secondary to the tutorial/worked-example content).

For **every implemented verb section** in commands.md, open with a one-line
when/why note. Required examples from the ticket (verify wording against
the binary):
- `migrate --rekey`: when it applies (scope_sha256/citation rekeying —
  verify the actual trigger conditions)
- `model-check`: how to interpret outcomes — clean /
  counterexample_found / timed_out / exploration_only (match
  `specs/model_check.md` claim_aggregate_governs semantics)
- `verify` vs `model-check`: when to choose which

**Meter (all must pass):**
- `grep -c 'when to use|Use when|Choose'` ≥ 1 per verb section for all
  implemented verbs (paste per-verb counts in the report)
- content matches `spk explain lifecycle` topic (no invented semantics)
- `just docs-build` green

**Anti-goals:** no book restructuring around commands; no aspirational
capability stated as implemented.

## Hard scope guard

- Allowed files: `docs/src/commands.md`, optionally one glossary/link line
  in `docs/src/index.md`.
- Never edit: `openspec/specs/`, `openspec/changes/`, `.espectacular/`,
  `pretender.toml`, `specs/` (the corpus), `book.toml`, `src/**`,
  `tests/**`, `.wai/resources/**`, `docs/src/specs/**`,
  `docs/src/openspec/**` (generated), this template.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
- **beads**: `bd close specodelic-lf4b.8` FIRST, then `bd export -o
  .beads/issues.jsonl`, then commit the export.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags.
- **Scratch work outside /tmp**: use `/var/tmp/<slug>`.
- Hand-authored docs pages carry no Purpose/Rationale header comments.

## Mechanical finish checklist (execute in order, do not skip)

1. Per-verb grep counts (meter) — paste in report
2. `just docs-build` → green
3. `git status --short` → only your files; stage explicitly
4. `git commit` with honest attribution
5. `bd close specodelic-lf4b.8` → `bd export -o .beads/issues.jsonl` →
   commit the export
6. Do NOT push; do NOT run `wai close`
7. End with the Report block below

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `just docs-build` → <result>
- `ah check` → <result>
- `openspec validate --all --strict` → <result>

**Meter evidence**
- <per-verb when/why grep counts>

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">