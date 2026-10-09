# Subagent brief: specodelic-lf4b.6 — Add guarantee-ladder / verification-boundaries page

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-lf4b.6` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=domain-specific-extensions`
- Read: `bd show specodelic-lf4b.6` (acceptance criteria + anti-goal), the
  epic `bd show specodelic-lf4b` (docs-only scope + staleness gate),
  `docs/src/status.md` (current capability table — the ticket notes point
  at lines 13-16), `docs/src/index.md`, and the repo `README.md` (it holds
  the canonical "verification is not application testing" statement — link,
  don't restate).
- **Source canvas:** the GPT-6 review's section 13 (per the ticket) proposes
  the guarantee ladder. The underlying semantics live in the repo:
  `spk explain` topics (enumerate with `spk explain --list` or the topic
  list in `docs/src/SUMMARY.md`), `spk verify --help`, `spk model-check
  --help`, and the corpus files `specs/verify.md`, `specs/model_check.md`,
  `specs/compile.md`. Treat any canvas wording as a draft — verify every
  claim against current main.
- **Staleness gate (mandatory, epic rule):** re-verify the ticket's claims
  against current main before writing prose:
  - **py:/ts: fragment emitter status** — the ticket says "labeled-deferred
    (Rust-only executable at 0.7.0 — verify against current main whether
    emitters have shipped)". CHECK THIS AT HEAD: the min-expr-kernel and
    verification-claims epics landed since the review. Look at
    `spk explain compile`-adjacent topics, `specs/compile.md` fragment
    semantics, and try an actual `spk compile` run containing a `**py:**`
    fragment to see what actually happens. State what the binary does NOW,
    not what the review said.
  - **per-verb maturity** — status.md's capability table is the seed, but
    re-derive from `spk --help` + `spk explain` at HEAD.
  - **coverage blind spot** — verify the exact semantics from
    `spk explain coverage` (or the matching topic) before claiming
    "coverage cannot detect MISSING constraints".
- `wai search "guarantee"` / `wai search "limitations"` / `wai search
  "docs"` — accumulated patterns.

## What to build

**Primary deliverable: a guarantee-ladder / verification-boundaries page**
in the docs book:

- The ladder as explicit, visually distinct levels — never collapsed into
  one PASS label:
  1. lint-clean (format-valid)
  2. well-modeled (state Model present/coherent)
  3. checked-within-bounds (coverage/properties exercised claims)
  4. matches-oracle (verification claims / model-check counterexample
     evidence)
  5. better-design (model-check as design critic — see the worked example's
     counterexample section for the precedent)
  Derive each level's actual gate command (`spk lint` / `spk compile` /
  `spk verify` / `spk model-check`) and what passing it does and does NOT
  assure. Source-anchor each level to the `spk explain` topic or corpus
  file that defines it.
- **Known limitations, up front** (each citing a runnable command whose
  output at HEAD substantiates the claim):
  - coverage blind spot: MISSING constraints are undetectable — cite the
    exact coverage semantics
  - py:/ts: fragment status as verified at HEAD (see staleness gate above)
  - per-verb implementation maturity (lint/compile/verify/model-check/
    graph/refactor/rename/merge/orchestrate — only claim what the binary
    does; mark partial verbs honestly)

**Placement:** per the ticket — a "Verification boundaries" section in
`docs/src/index.md` OR a dedicated new page (e.g.
`docs/src/verification-boundaries.md`) linked from SUMMARY.md. Pick the
shape that fits the book's voice; a dedicated page is likely better since
index.md is the entry page. If a dedicated page, keep a short pointer
section in index.md.

**Same-ticket companion edit (from the docs review note):** rewrite
`docs/src/status.md` lines 13-16 "In progress" entries so a cold reader
understands them — expand the "Self-hosting round" and "DDL-u8x epic"
jargon into plain sentences (what the round does, why it matters), without
changing the page's structure or the rest of the content.

**Meter (all must pass):**
- every limitation claim cites a runnable command whose output at HEAD
  substantiates it (capture the output; re-run before commit — diff-clean)
- no aspirational capability stated as implemented (anti-goal)
- `just docs-build` green
- new page linked from `docs/src/SUMMARY.md` (and index.md pointer if
  dedicated page)

## Hard scope guard

- Allowed files: the new page under `docs/src/`, `docs/src/SUMMARY.md`,
  `docs/src/index.md`, `docs/src/status.md`.
- Never edit: `openspec/specs/`, `openspec/changes/`, `.espectacular/`,
  `pretender.toml`, `specs/` (the corpus), `book.toml`, `src/**`,
  `tests/**`, `.wai/resources/**`, `docs/src/specs/**`,
  `docs/src/openspec/**` (generated), `README.md` (link it, don't edit it),
  this template.
- Respect shrink-only ratchets: run `pretender check` before committing;
  if a pinned source file breaches, move new tests to `tests/` rather than
  raising entries. (Docs-only ticket — you should not touch pinned files
  at all.)

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; commit the export after
  `bd close` (`bd export` gotcha: run it or the close is invisible to git).
  Order note from lf4b.5: `bd close` FIRST, then `bd export`, then commit —
  so the close is actually recorded in the committed JSONL.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.
- **Scratch work outside /tmp**: /tmp quota exhausts — use `/var/tmp/<slug>`.
- Hand-authored docs pages carry no Purpose/Rationale header comments —
  follow the existing pages' precedent.

## Mechanical finish checklist (execute in order, do not skip)

1. Re-capture every cited CLI output at final HEAD; diff-clean
2. Grep the page: no capability claimed as implemented without a
   demonstrating command (anti-goal self-check)
3. `just docs-build` → green
4. `grep -n "verification-boundaries\|SUMMARY" docs/src/SUMMARY.md` → link
   present
5. `git status --short` → only your files; stage explicitly
6. `git commit` with honest attribution
7. `bd close specodelic-lf4b.6` → `bd export -o .beads/issues.jsonl` →
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

**Claims verified at HEAD**
- <py:/ts: emitter status found> / <coverage blind spot semantics> /
  <per-verb maturity> — one line each with the substantiating command

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">