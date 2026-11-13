---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-04c-kernel-vocabulary-pack, pipeline-step:orient]
---

# ORIENT — specodelic-04c: kernel vocabulary pack

## Ticket
`bd show specodelic-04c` (P2, IN_PROGRESS, parent specodelic-ez9a): decide how
`kernel.binding` vocabulary is enabled — `orphan_vocabulary` labels every
file whose Constraints/Properties table carries a `kernel.binding` column
until a `kind: profile` pack in namespace `kernel` is discovered.

## Owner decision of record (countersigned 2026-11-13)
Ship a **kernel vocabulary pack** — a `kind: profile` spec file in the
corpus, namespace `kernel` — NOT a linter carve-out. Packs are the declared
extension path (packs spec: enablement by vocabulary use or a declared
`uses` edge, never by config, never by narrowing a base closed set).

## Mechanism facts (read in src/packs.rs, not guessed)
- `orphan_vocabulary`'s vocabulary half (`vocab_orphan_tokens`,
  src/packs.rs) scans **structured positions only**: `spec.intent.kind`,
  each Constraint/Property `kind` cell, and each Constraint/Property
  **column header** (`cells.keys()`). A `kernel.binding` column header
  yields the token `kernel.binding` → namespace `kernel`.
- A token is orphaned iff its namespace matches **no discovered pack**.
  `pack_namespaces` = the first dotted segment of every discovered pack's
  frontmatter id (src/packs.rs, `pack_pass`). So a pack with id
  `kernel.pack` puts `kernel` in the namespace set → the finding stops
  firing — no config, no carve-out, no code change.
- Discovery is a **corpus scan** anchored at the git repository toplevel of
  every linted corpus (`discovery_roots`), finding every `kind: profile`
  file — so a pack anywhere in the repo tree is discovered.
- Pack vocabulary (what a consumer's use matches) = `## Kinds` +
  `## Sections` + `## References` tokens (`Manifest::vocabulary`). The
  `kernel.binding` **column** is a field → it belongs in `## References`
  (the facet the discovery rule can actually match).
- `pack_shape` (src/packs.rs `shape_findings`): all six manifest tables
  must be present, **non-empty**, two-column rows; `## Kinds` rows must be
  pack-qualified (never a base closed-set name).

## Corpus facts
- `kind: profile` files live today in `packs/*.md` (id `numeric.predicates`,
  `data.lineage`, …), while `openspec/specs/<name>-pack/spec.md` are the
  kind: intent *capability* specs for them.
- The corpus revision is **20** (`src/guide.rs` FORMAT_REVISION,
  `openspec/specs/specodelic/spec.md` Revision 20).
- `openspec/specs/kernel/spec.md` (id `kernel`) already exists and
  documents the `kernel.binding` column; the pack adds the *enablement*,
  not the column semantics.

## Deploy-path finding (why direct authoring)
The change-delta path **cannot** express a pack capability: `openspec
archive` drops every non-Requirement section of a delta ("section '##
Sections' is not carried into the main spec" — reproduced in a scratch
project), and `spk archive-companion` re-attaches only the sidecar's
frontmatter + `## Constraints`/`## Model`/`## Properties`
(`assemble`, src/archive_companion.rs). The six manifest facets would not
survive a deploy → `pack_shape` would fire six findings on the deployed
file. Hence direct authoring of the new corpus file (the brief's stated
fallback), recorded as a deviation.

## CI gates that constrain the landing site
- `scripts/check_summary_completeness.py`: every frontmatter-bearing
  `openspec/specs/<cap>/spec.md` must be linked from `docs/src/SUMMARY.md`
  → one SUMMARY line is mandatory for a corpus-tree pack (the other
  `*-pack` capability specs are listed there too).
- `scripts/check_pack_doc_counts.py` derives the pack inventory from
  `packs/*.md` → adding a file under `packs/` would force pack-count prose
  updates on 5 surfaces; a corpus-tree pack leaves that inventory
  untouched.
- `scripts/check_lint_baseline.py`: the corpus must lint with **0** issues.
- `tests/cli/lint.rs::lint_corpus_is_fully_clean` pins `files_linted`
  (48 → 49 with the new corpus file).

## Baseline
`cargo run -q -- lint openspec/specs` → 48 files, **0 issues**, 6 packs
discovered (bioimage.data, data.lineage, empirical.registry,
numeric.predicates, quant.finance, svc.contracts).

