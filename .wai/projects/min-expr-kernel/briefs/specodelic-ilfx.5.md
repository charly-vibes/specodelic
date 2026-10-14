# Subagent brief: specodelic-ilfx.5 — docs: worked example — bioimage.data pack (pilot)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-ilfx.2` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

NOTE (siblings .2–.4 already shipped): `docs/src/examples/data-lineage.md`,
`numeric-predicates.md`, `empirical-registry.md` are the precedents — same
structure, same meter pattern. Read them first; keep yours consistent
(naming, section shape, failure-section approach, honest-empty disclosure).
IMPORTANT: bioimage.data is the PILOT pack — `scripts/check_pack_doc_counts.py`
derives standard = all packs minus bioimage.data. Never call it standard; it
is a pilot/experimental pack. Keep pack ids named, never counted.

## Orientation (do this first)

- `export WAI_PROJECT=min-expr-kernel`
- Read, in order: `bd show specodelic-ilfx` and `bd show specodelic-ilfx.5`
  (scope + REVIEW FIX meter note), `docs/src/examples/worked-example.md` and
  the SHIPPED siblings `docs/src/examples/data-lineage.md` /
  `numeric-predicates.md` / `empirical-registry.md` (structural mirrors —
  real spec file + real lint output, failures included then fixed),
  `packs/bioimage-data.md` (the pack definition — source of truth for its
  vocabulary), and `docs/src/using-packs.md` (the guide — cross-link it;
  do not restate its mechanism prose).
- `wai search "packs"` and `wai search "worked example"` — accumulated patterns.
- Check how the existing corpus uses bioimage.data vocabulary:
  `openspec/specs/bioimage-data_pack/spec.md` and any corpus files with
  `## Axes` rows (`grep -rl "bioimage.data" openspec/specs/`).
- OME/NGFF + BioImage.IO stay EXTERNAL — opaque binding column, prose
  pointer only; do not model their schema rows in-page.

## What to build

New worked-example page `docs/src/examples/bioimage-data.md` (companion
spec embedded in-page, mirroring worked-example.md and the shipped
siblings), wired into `docs/src/SUMMARY.md` (under the examples section).
Required content per the ticket:

- A real, self-contained spec file exercising the bioimage.data pack
  vocabulary: a `## Axes` section with rows (name/axis/scale/unit columns),
  the `bioimage.transform` property kind, the `same_shape_as` reference
  field, the named pack-qualified checkers (`bioimage.dtype_is`,
  `bioimage.shape_eq`, `bioimage.units_convertible`), and the
  kind-dependent floor (preserves+dtype). USE ONLY reference fields the
  pack's `## References` manifest actually declares (the .2 lesson).
- **Real lint output, captured live**: run
  `cargo run -- lint <the example file>` from repo root against the
  companion spec placed in a scratch dir where pack discovery applies, and
  paste the actual output. The page MUST show the pack-activation advisory
  naming `bioimage.data` (warnings channel, exit 0).
- **Failures included then fixed**, per worked-example.md and the .2
  sibling: show genuinely failing variants with real labeled findings
  (real caught failures at HEAD, e.g. a dangling `traces_to` →
  `linter.total_refs`, or the `linter.orphan_vocabulary` pair when the
  pack isn't discovered), plus an explicit "what the linter does not catch
  (yet)" section noting honest-empty checker semantics. Do not fabricate
  output — every block must be real captured output at HEAD.
- Structure guidance: what the example is, the spec file (annotated or
  walk-through), the lint loop, the failure→fix, pointers to
  `specs/packs/spec.md` and the pack definition.

Content discipline:

- Source of truth for vocabulary is `packs/bioimage-data.md` and
  `openspec/specs/bioimage-data_pack/spec.md` — mirror its row/column
  names exactly; do not invent checker behavior (v1 checkers are
  honest-empty: activation + empty checked-set; violation evaluation is
  NOT yet implemented — specodelic-ilfx.1's guide documents this; keep
  failure examples to things the linter actually catches at HEAD and say
  which rule fired).
- Anti-goal: no count-phrase prose ('N standard packs'). Name the pack id,
  never count packs.
- Write for a cold read: define every term on first use; cross-link
  `using-packs.md` for mechanism, this page for hands-on.

## Meters (must pass before you report done)

1. From repo root: `spk lint` (i.e. `cargo run -- lint`) against the
   example's companion spec MUST exit 0 AND emit the pack-activation
   advisory naming `data.lineage` — exit-0 alone proves nothing
   (byte-identical guarantee); the activation line is the observable.
2. `mdbook build docs` — exit 0 (page renders, links resolve).
3. `python3 scripts/check_summary_completeness.py` — SUMMARY.md entry valid.
4. `python3 scripts/check_pack_doc_counts.py` — still green (no counted
   pack-phrase prose introduced).
5. `just test` — green (docs-only change expected).
6. If llm.txt becomes stale from the SUMMARY entry, run
   `python3 scripts/stamp_llms.py root` per its own remediation (same as
   specodelic-ilfx.1 did).

## Hard scope guard

- Allowed files: `docs/src/examples/bioimage-data.md` (new),
  `docs/src/SUMMARY.md` (one entry), `llm.txt` (generated restamp only).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `packs/` definitions, `src/**`, `.wai/resources/**`, `docs/src/using-packs.md`
  (cross-links FROM other pages TO the guide are fine; do not edit the guide).
- Respect shrink-only ratchets: run `pretender check` before committing.
  Docs ticket — you should not touch `src/` at all.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; commit the export after
  `bd close` — but for THIS ticket: do NOT `bd close`; the orchestrator
  owns closure.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `cargo run -- lint <companion spec>` → <exit + activation advisory naming
  bioimage.data present?>
- `mdbook build docs` → <result>
- `python3 scripts/check_summary_completeness.py` → <result>
- `python3 scripts/check_pack_doc_counts.py` → <result>
- `just test` → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">
