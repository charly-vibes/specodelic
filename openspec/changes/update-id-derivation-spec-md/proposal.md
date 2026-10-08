# Update: id derivation — spec.md files take their id from the parent directory

## Why

The naming law (`id_matches_file`) required `frontmatter.id` to equal the
filename stem, forcing every `spec.md` — the openspec-mandated filename for
both deltas and deployed capability specs — to declare `id: spec`. Combined
with the self-containment rule for `id: spec` files (`total_refs` scoped
index, the command scope law's `isolated_scope_required`), this structurally
locked a two-tree architecture: a real-id corpus for the typed graph plus a
deployed dual-format tree produced by per-repo deploy transforms. Every new
user inherited the distinction and had to learn two vocabularies.

Evidence (pilot, 2026-10-08): dropping a real-id corpus file as
`openspec/specs/<cap>/spec.md` yielded exactly one finding —
`id_matches_file`. Everything else already composes under a single tree:
`openspec validate --strict` passes on four-layer dual-format content,
`spk lint` `total_refs` resolves cross-file links across the lint set,
espectacular reads paths from config.

## What Changes

- **`id_matches_file`** — `expected_id_from_path(path)`: a file named
  `spec.md` derives its expected id from its PARENT DIRECTORY name
  (`openspec/specs/ge-cli/spec.md` → `id: ge.cli`); any other file derives
  from its own stem; a bare `spec.md` with no parent directory falls back
  to the stem. `-` maps to the namespace dot, `_` is literal.
- **`id: spec` retires.** `dual_format_valid` no longer polices the id;
  `total_refs` resolves every file corpus-wide (the self-containment
  scoping and its bare-local arm are gone); the command scope law reduces
  to `duplicate_corpus_identity` (a same-id pair — transitionally an active
  change's delta and its deployed spec — is refused outright on
  model-check/verify/orchestrate paths, strictly stronger than the old pair
  of laws).
- **Scaffolds teach the single tree.** `spk new` targets
  `openspec/specs/<id>/spec.md` by default; `spk migrate` generates the
  real derived id in the frontmatter it scaffolds.
- **Format revision bump** Revision 17 → 18 in `specs/specodelic.md` (the
  format's own corpus, which carries `## Revision 18`); `FORMAT_REVISION`
  in `src/guide.rs` bumped in lockstep.
- **This repo's openspec tree migrates**: every dual-format `spec.md`
  under `openspec/specs/` and the active changes now carries its real
  parent-dir-derived id and file-qualified refs; archived deltas drop
  their frontmatter (frozen pre-protocol evidence, parse-skip for spk,
  openspec grammar unchanged); boilerplate lifecycle models gained a real
  `process_lifecycle` invariant with cited guards (they previously linted
  clean only because the shared `id: spec` intent node accidentally
  anchored every file's rows — the collision was load-bearing).

## Impact

- Affected specs: `spec-integration` (the dual-format law — MODIFIED),
  plus the format corpus `specs/specodelic.md`, `specs/linter-frontmatter.md`,
  `specs/model_check.md`, `specs/linter-referential_integrity.md` (the
  specodelic-layer statements of record).
- Affected code: `src/lint/rules.rs`, `src/lint/graph.rs`, `src/graph.rs`,
  `src/lint/observability.rs`, `src/acset/instance.rs`,
  `src/citation_corpus.rs`, `src/migrate.rs`, `src/commands/manage.rs`
  (`new`/`migrate`), `src/guide.rs` (`FORMAT_REVISION`), `src/guide.md`
  (embedded dual-format topic).
- NOT in scope (follow-up rollout): the corpus single-tree migration
  (moving `specs/*.md` into `openspec/specs/<cap>/spec.md`), the tambor
  contract re-keying, and `archive-companion`'s write-path retirement —
  the revision makes them possible; they land as separate changes.
