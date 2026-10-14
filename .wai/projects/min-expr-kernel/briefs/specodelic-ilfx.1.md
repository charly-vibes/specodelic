# Subagent brief: specodelic-ilfx.1 — docs: 'Using packs' guide page

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-ilfx.1` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: `bd show specodelic-ilfx` and `bd show specodelic-ilfx.1`
  (scope + REVIEW FIX notes), `openspec/specs/packs/spec.md` (the pack
  mechanism spec — your primary source of truth for semantics), and
  `docs/src/commands.md` §Domain packs (existing prose you must align with,
  incl. line ~47: 'Lint discovers packs by corpus scan (no config file)').
- `wai search "packs"` — check accumulated patterns before writing.
- This is a docs ticket (no tdd-ro5 run needed). The meters are runnable
  checks listed below — treat RED (build/lint failure) → GREEN (all pass) as
  your loop, plus a content-review pass against the ticket's checklist.

## What to build

New guide page `docs/src/using-packs.md`, wired into `docs/src/SUMMARY.md`
(under 'The Format', after the packs spec link — a guide next to the spec it
explains). Required content per the ticket:

- **What a `kind: profile` pack is** — `packs/*.md` frontmatter as source of
  truth; the committed `specodelic-typ0` CI gate that keeps pack-inventory
  prose honest.
- **Enablement = corpus-scan discovery, NOT a config file**: a `kind: profile`
  file under the git toplevel of each linted dir is discovered by lint
  (`src/packs.rs` discovery_roots; `commands.md:47` 'no config file').
  Document the discovery anchor rule (specodelic-o5bh: anchor = git toplevel
  of each linted dir). There is NO packs.toml — say so explicitly so readers
  coming from other tools don't hunt for one.
- **What the mechanism adds**: namespaced sections, pack-qualified kinds,
  reference fields resolving same-file, pack-qualified checkers,
  kind-dependent case-label floors.
- **Byte-identical guarantee** for spec files using none of the pack
  vocabulary.
- **Honest-empty checker declarations**.
- **CLI loop**: `spk lint` / `spk graph` / `spk compile` against a
  pack-enabled spec — show the actual commands and the advisories/findings to
  expect (pack-activation notes on the warnings channel, exit 0;
  `orphan_vocabulary` labeled failures naming candidate pack + both
  remediations; `linter.pack_shape` findings for malformed manifests).
- **Cross-links**: from `commands.md` and `faq.md` pack mentions to the new
  guide. Keep all existing pack mentions enumerated, not counted.

Content discipline:

- Source of truth for semantics is `openspec/specs/packs/spec.md` — do not
  invent mechanism details; mirror its vocabulary exactly.
- Anti-goal: no count-phrase prose ('N standard packs',
  'six packs ship'). `scripts/check_pack_doc_counts.py` enumerates
  README/index/faq/commands/llms — new surfaces are unregistered, but keep
  pack mentions enumerated anyway. Name the five standard packs
  (`data.lineage`, `numeric.predicates`, `empirical.registry`,
  `bioimage.data`, `quant.finance`) and note `svc.contracts` is draft.
- Write for a cold read: define every term on first use, link to
  `specs/packs/spec.md` rather than restating schema rows.

## Meters (must pass before you report done)

1. `mdbook build docs` — exit 0 (page renders, links resolve).
2. `python3 scripts/check_summary_completeness.py` — SUMMARY.md entry valid.
3. `python3 scripts/check_pack_doc_counts.py` — still green (no counted
   pack-phrase prose introduced anywhere you touch).
4. `cargo run -- lint docs/src/using-packs.md` — the guide itself is a
   non-spec doc file (no frontmatter ⇒ exempt), but confirm lint doesn't
   error on it if you run it; not a required pass.
5. `just test-smart` (falls back to `cargo test`) for any touched Rust (only
   if you touch scripts/ shell/python, run their invocations manually).

## Hard scope guard

- Allowed files: `docs/src/using-packs.md` (new),
  `docs/src/SUMMARY.md` (one entry), `docs/src/commands.md` (cross-link only),
  `docs/src/faq.md` (cross-link only), `docs/src/index.md` (cross-link only
  if a natural anchor exists).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template, `src/**`
  (Rust code), `packs/` definitions.
- Respect shrink-only ratchets: run `pretender check` before committing; if a
  pinned source file breaches, move new tests to `tests/` rather than raising
  entries. `src/model_check.rs` is pinned (see `pretender.toml`) — new
  model-check tests go in `tests/` integration files. (Docs ticket — you
  should not touch `src/` at all.)

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; commit the export after
  `bd close` (`bd export` gotcha: run it or the close is invisible to git).
  — but for THIS ticket: do NOT `bd close`; the orchestrator owns closure.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive — author them in the
  archive commit, never mid-change. (Not relevant here — no spec change.)
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers skill convention). (Docs prose is exempt, but add a short
  purpose note in the page intro if natural.)
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `mdbook build docs` → <result>
- `python3 scripts/check_summary_completeness.py` → <result>
- `python3 scripts/check_pack_doc_counts.py` → <result>
- `just test` → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">
