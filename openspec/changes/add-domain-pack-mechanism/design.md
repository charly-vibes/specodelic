# Design: add-domain-pack-mechanism

Decision of record: approach **02-profile-intent** (wai design matrix,
`.wai/projects/domain-specific-extensions/designs/matrix/decision.md`),
D1 v2 manifest shape (Ro5-reviewed READY WITH_NOTES, TypeSafe-verified on
HIGH findings), D6 pilot = bioimage-data (follow-on change).

## Categorical frame (grounded in specs/theory.md)

- Core = the schema category 𝒦 (5 kinds) + generating morphisms (Reference
  Typing) + laws + append-only discipline. An instance is a functor
  I : 𝒦 → Set; linting checks I is well-formed.
- **Strict core rule (D7):** core grows only when a mechanism is required
  for *any* instance to be a well-formed functor at all. The pack mechanism
  itself is such a mechanism (R8); the vocabulary packs carry is extra
  structure on I and never touches 𝒦.
- **Packs are Grothendieck one level up:** pack-qualified kinds
  (`<pack>.<kind>`) reuse the existing ∫-over-file-ids namespacing with
  pack ids as the index. Membership in closed sets becomes
  *fiber-relative*; the base kind set literally stops growing. No new
  mathematics.

## What the core Revision delta contains (exactly)

1. `specs/specodelic.md` — Revision 9 heading: INTENT_KINDS grows by
   `profile`; Reference Typing grows by `uses` (outbound leaf → profile
   file's frontmatter id). Instance-definition changes only, per §7.
2. `specs/packs.md` — new corpus spec file (created FIRST, spec-first
   convention) describing the pack mechanism in the format it describes.
3. `INTENT_KINDS` in `src/guide.rs` + `REFERENCE_TYPING` gains `uses`.
4. New append-only lint rule `pack_shape` (+ checker-ownership table row)
   validating manifest structure.

`uses` is core (a generating morphism the mechanism requires);
`satisfies` is NOT overloaded — see "Opt-in edge" below.

## D1 v2 — pack artifact

- Four-layer spec file, `kind: profile`. EARS statement pattern for the
  intent statement: "WHEN a workspace enables this pack, THE format SHALL
  provide …".
- Manifest = six per-facet closed tables (Checker Ownership precedent):

| Table    | Declares                                                     |
|----------|--------------------------------------------------------------|
| Sections | optional sections the pack introduces (row shape per pack)   |
| Kinds    | namespaced kind growth (e.g. `bioimage.tolerance`)           |
| References | new reference *fields* the pack adds (outbound leaves)     |
| Checkers | lint rules the pack owns (ownership-table rows)              |
| Floors   | required case-label sets per kind (`**name:**` machinery)    |
| Requires | pack-authored requirements: the `base` format_revision pin + dependent pack ids with their revision pins (row shape: `\| <dep id \| base> \| <required revision> \|`) |

## Resolved open questions (from D1 v2's open list)

- **Opt-in edge: `uses`, not `satisfies` overload.** `satisfies` means
  "conforms to a published contract" (targets an extension_point
  Constraint); pack enablement is a different relation ("this file uses a
  vocabulary fiber"). Overloading would muddy typing-violation messages
  and the audit trail. `uses` is a new outbound leaf in the same Revision
  delta — nothing targets it, so `single_root_reachable` needs no
  carve-out (mistral D4 precedent via `validated_against`).
  Full Reference Typing row (mirrors the `satisfies`/`observes` pattern):

  | Field  | Appears on           | Must resolve to |
  |--------|----------------------|-----------------|
  | `uses` | Constraint, any file | Intent of a `kind: profile` file (the pack file's frontmatter id) — set-valued (multiple `[[...]]` targets in one cell, like `traces_to`); an outbound leaf that joins no reachability path and no acyclic edge set, so mutual pack use (`packA` consumes `packB`'s vocabulary and vice versa) is well-formed |
- **`## References` rows add fields, never reshape existing ones.** A pack
  may introduce a new reference field or a new outbound-leaf column
  vocabulary; it may never narrow or retype an existing field
  (`append_only_variants` applies to packs themselves).
- **`## Requires` is authored by the pack, not the consumer.** The pack
  declares what it needs (base revision, pack deps); a consumer declares
  enablement via a `uses` edge. Skew compares the pack's `base` pin
  against the workspace corpus revision (FORMAT_REVISION) — one
  direction, no consumer-side pin artifact.
- **Section registry (D2):** sections are registered per-pack in the
  manifest; two packs claiming the same section *mechanism* is legitimate;
  collision exists only at vocabulary level, resolved by pack-qualified
  naming. Registry law = declaration, not first-come ownership (D2/D3).
- **No `section_kind_closed` core rule:** anything-goes-with-registered-
  checker holds at core; the pack's own `## Sections` table is the closed
  set, checked by `pack_shape` only for structural well-formedness.

## Discovery & opt-in

- Discovery = corpus scan: every `kind: profile` file in the workspace is
  a candidate pack; no config file, no external registry. The scan is
  anchored at the git repository toplevel (the lint target directory
  itself when no git root exists) — linting a subdirectory still
  discovers packs living anywhere in the workspace, so vocabulary used
  in a subdir never produces spurious orphan findings.
- **Pack self-exemption:** a pack file is never a consumer of declared
  vocabulary — its own manifest rows never trigger its or any pack's
  checkers and never produce orphan findings.
- **Activation is per-pack:** identical vocabulary declared by two
  discovered packs activates both, findings attributed per pack;
  vocabulary matching is longest-prefix (a `bioimage.tolerance.high` use
  matches the most specific declaring pack).
- Opt-in is advisory-first, vocabulary-use-triggered: when a file uses
  vocabulary a discovered pack declares, the pack's checkers run for that
  file. Two modes: implicit (vocabulary match) and declared (`uses` edge
  to the pack id) — declared mode enables pinning/skew checks.
- Vocabulary used with no matching pack discovered = labeled finding
  naming the candidate pack (error-contract: name both remediations —
  enable/declare the pack, or fix the vocabulary).
- Orphan vocabulary (declared vocabulary used with no pack discovered or
  declared) is a labeled **failure** finding (non-zero exit) naming the
  candidate pack — prefix-derived when only the namespace is known — and
  both remediations (error-contract).
- Files without packs lint byte-identically to today (zero migration).

## Lifecycle & skew

- Pack Model states: `draft` → `published` → `deprecated` (frontmatter
  `## Model` section, standard four-layer shape). `pack_shape` must pass
  at parse in every state; a `draft` pack's checkers activate
  advisory-first with the draft status named in findings (draft is a
  maturity label, not an activation gate).
- `## Requires` pins base format_revision (+ optional pack deps). Skew
  between a consumer's enabled pack and the workspace corpus revision =
  labeled advisory on the warnings channel, never silent, never failing
  (knowledge-currency precedent from `spk doctor`).
- Deprecated packs: advisory only; vocabulary still checks, finding labels
  name the deprecation.

## Honest-empty convention

Pack checkers report empty checked-sets (`invariants_checked: []`-style)
when the workspace provides no fixtures/data for the declared vocabulary —
praised by every vendor, preserved verbatim.

## Rejected alternatives

- **01 status quo** — global closed sets collide across independent packs.
- **03 row-level extension_point** — the edge cannot carry section/kind/
  floor registration (matrix column 03 red on floors/time/EARS; D5).
- **04 external manifest** — violates the self-hosting ethic; pack
  knowledge would live outside the corpus.

## Follow-on sequencing

1. This change: the mechanism (core revision + tooling + specs/packs.md).
2. Standard packs (data/lineage, numeric predicates, empirical kind +
   floor registry) — one proposal each, governance rule from packs.md.
3. `add-bioimage-pack` — thin domain instance, D6 pilot; quant second.
