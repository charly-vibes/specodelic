# Tasks: add-domain-pack-mechanism

Spec-first per repo convention: the corpus spec file lands before any
checker code. Each phase maps to a red→green→refactor cycle; refactors
are separate tasks from features.

## 1. Spec-first (corpus)

- [ ] 1.1 Create `specs/packs.md` — four-layer corpus spec describing the
      pack mechanism (kind: profile, six manifest tables, corpus-scan
      discovery, advisory-first opt-in, `uses` outbound leaf, lifecycle,
      honest-empty). File-naming law: `id: packs`. Must pass `spk lint`.
- [ ] 1.2 Revision 9 delta in `specs/specodelic.md`: append-only Revision
      heading — INTENT_KINDS grows by `profile`, Reference Typing grows by
      `uses` (outbound leaf → profile id). Update STATUS.md §1 primer +
      USAGE.md.
- [ ] 1.3 Regenerate corpus artifacts (`spk compile`) so committed artifacts
      stay byte-stable; `just ci` + `just lint-specs` green.

## 2. Guide seam (core growth)

- [ ] 2.1 RED: extend guide.rs tests — `INTENT_KINDS` contains `profile`,
      `REFERENCE_TYPING` contains `uses` typed as outbound leaf (target:
      profile/intent file id); `spk new` template renders the grown sets;
      drift-guard unit test still passes.
- [ ] 2.2 GREEN: grow the two closed sets in `src/guide.rs`; bump
      `FORMAT_REVISION` to `specodelic.md Revision 9`; update guide.md
      primer topics with pack guidance (append-only topics, never
      renumbered).

## 3. Manifest parser (pack_shape RED→GREEN)

- [ ] 3.1 RED: `pack_shape` fixture corpus in tests/cli.rs — well-formed
      `kind: profile` file with all six tables → 0 findings; each
      single-facet mutation (missing table, malformed row, non-profile
      frontmatter claiming a manifest) → labeled finding naming the facet.
- [ ] 3.2 GREEN: manifest parser in `src/packs.rs` (six facet tables,
      fixed row shapes) + append-only lint rule `pack_shape` registered in
      lint.rs RULE_TABLE (rule_id + rule_semantics); wire the
      checker-ownership row.
- [ ] 3.3 `spk explain lint-rules` renders the new rule (full-catalog test
      bumps); CHANGELOG entry.

## 4. Discovery + opt-in (advisory-first)

- [ ] 4.1 RED: discovery tests — corpus scan finds every `kind: profile`
      file (order-independent, no config); zero profile files → machinery
      inert, lint output byte-identical to pre-mechanism.
- [ ] 4.2 GREEN: `src/packs.rs` discovery over the parsed corpus; surfaced
      in lint envelope data (packs discovered) and `spk doctor` output.
- [ ] 4.3 RED: opt-in tests — vocabulary-triggered activation for files
      using declared vocabulary; `uses` edge enables pinning/skew checks;
      orphan vocabulary → labeled finding naming the pack + both
      remediations (error-contract).
- [ ] 4.4 GREEN: vocabulary matching + `uses` edge in Reference Typing
      (graph paths: typing violations for wrong-kind `uses` targets reuse
      existing typing machinery); orphan finding rides the labeled
      finding convention.

## 5. Lifecycle + skew

- [ ] 5.1 RED: lifecycle tests — draft/published/deprecated states parsed
      from the pack's Model; `## Requires` pinning; skew = warning-channel
      advisory naming both revisions, exit 0; deprecated pack findings
      name the deprecation.
- [ ] 5.2 GREEN: lifecycle/skew logic in `src/packs.rs`; skew rides the
      warnings channel (knowledge-currency precedent).

## 6. Gates + docs

- [ ] 6.1 `just ci` (fmt-check, clippy -D warnings, tests, release build) +
      `just lint-specs` + `openspec validate --all --strict` green.
- [ ] 6.2 Docs: docs/src pages for packs (SUMMARY.md entry), README CLI
      section, STATUS §3; corpus `spk explain` topic only if primer-
      adjacent (append-only, never renumbered).
- [ ] 6.3 CHANGELOG entries; beads ticket filed/closed with
      metadata.files; `bd export -o .beads/issues.jsonl` (not bare export).

## 7. Close-out (follow-on triggers)

- [ ] 7.1 Self-dogfood check: `spk lint` over the mechanism's own spec set
      reports no orphan-vocabulary findings.
- [ ] 7.2 File follow-on beads tickets: standard packs (data/lineage,
      numeric predicates, empirical kind + floor registry) and
      `add-bioimage-pack` (D6 pilot) — one proposal each, gated on this
      change archiving.
