# Subagent brief: specodelic-lf4b.7 — Add a cold-reader glossary to the docs book

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-lf4b.7` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=domain-specific-extensions`
- Read: `bd show specodelic-lf4b.7` (acceptance criteria + anti-goal), the
  epic `bd show specodelic-lf4b` (docs-only scope + staleness gate),
  `docs/src/index.md` and `docs/src/commands.md` (the pages whose undefined
  terms you must cover), `docs/src/SUMMARY.md`.
- **Model to follow:** the Mistral report's section-2 glossary —
  `~/Downloads/cv/specodelic-v0-7-0-design-first-specification-report-&-bundle.md`
  (quote the path exactly; spaces and `&` in the filename). It is a jargon
  table sized for a cold read. Adapt, don't copy blindly — verify every
  definition against the corpus (staleness gate below) before using it.
- **Staleness gate (mandatory, epic rule):** every glossary definition must
  be verified against current main. Authoritative sources, in order of
  authority: `specs/specodelic.md` (format core), the matching
  `specs/*.md` corpus file for tool terms, `spk explain` topics
  (enumerate with the topic list), and `spk --help`. The Mistral report was
  written against v0.7.0 by an external reviewer — treat its wording as a
  draft, not ground truth. Cross-check: `docs/src/verification-boundaries.md`
  (lf4b.6) already defines the guarantee ladder — link to it rather than
  duplicating its full semantics.
- `wai search "glossary"` / `wai search "docs"` — accumulated patterns.

## What to build

**`docs/src/glossary.md`** — a glossary sized for a cold read (a table or
compact definition list; follow the Mistral section-2 shape but verify
content):

**Required terms (meter: ≥12, all of these must appear):**
- EARS
- TLA+
- proptest
- stateright
- kernel
- scope_sha256
- SUT
- oracle
- dual-format
- pack
- claim partition
- guarantee ladder

Plus **self-hosting round** (flagged by both Z.ai and the docs review —
see `docs/src/status.md`'s rewritten "In progress" entries from lf4b.6 for
the verified plain-language meaning).

**Coverage sweep (the meter's other half):** grep `docs/src/index.md` and
`docs/src/commands.md` for jargon used without definition — every such term
must be present in the glossary. Common suspects to check: linter family
names, `frontmatter`, `Constraints`/`Model`/`Properties` layer names,
`derives_from`/`traces_to`/`observes` link kinds, `counterexample`,
`advisory` (vs gating). Add what the sweep actually finds; don't pad.

**Anti-goal (from the ticket): no glossary entries for terms the corpus
never uses** — before adding any term beyond the required list, confirm it
appears in the corpus or docs; drop it otherwise.

Definitions should be one to three sentences, cold-reader-first (what it
is, why it matters here), and where a deeper explanation exists in the
book or corpus, link it (e.g. guarantee ladder →
`verification-boundaries.md`; pack semantics → the relevant `spk explain`
topic or corpus file) instead of restating.

**Meter (all must pass):**
- ≥12 required terms present
- every term used in index.md/commands.md without definition is present
  (paste your sweep evidence in the report)
- `docs/src/SUMMARY.md` links the page
- `just docs-build` green

## Hard scope guard

- Allowed files: `docs/src/glossary.md` (new), `docs/src/SUMMARY.md`,
  optionally small cross-link edits in `docs/src/index.md` (e.g. pointing
  "jargon?" → glossary).
- Never edit: `openspec/specs/`, `openspec/changes/`, `.espectacular/`,
  `pretender.toml`, `specs/` (the corpus), `book.toml`, `src/**`,
  `tests/**`, `.wai/resources/**`, `docs/src/specs/**`,
  `docs/src/openspec/**` (generated), this template.
- Respect shrink-only ratchets: run `pretender check` before committing
  (docs-only ticket — you should not touch pinned files at all).

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; order note from lf4b.5:
  `bd close specodelic-lf4b.7` FIRST, then `bd export -o
  .beads/issues.jsonl`, then commit the export — so the close is recorded
  in the committed JSONL.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.
- **Scratch work outside /tmp**: /tmp quota exhausts — use `/var/tmp/<slug>`.
- Hand-authored docs pages carry no Purpose/Rationale header comments —
  follow the existing pages' precedent.

## Mechanical finish checklist (execute in order, do not skip)

1. Term sweep evidence: list the index.md/commands.md undefined terms you
   found and confirm each is in the glossary
2. Anti-goal self-check: every glossary entry appears in corpus or docs
   (grep evidence for any term beyond the required 13)
3. `just docs-build` → green
4. `grep -n glossary docs/src/SUMMARY.md` → link present
5. `git status --short` → only your files; stage explicitly
6. `git commit` with honest attribution
7. `bd close specodelic-lf4b.7` → `bd export -o .beads/issues.jsonl` →
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

**Sweep evidence**
- <terms found undefined in index.md/commands.md → all present in glossary>

**Anti-goal check**
- <terms considered and dropped because the corpus never uses them, if any>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">