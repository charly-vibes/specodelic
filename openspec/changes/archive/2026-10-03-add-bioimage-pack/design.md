# Design: add-bioimage-pack

## Context

The mechanism (`add-domain-pack-mechanism`, archived 2026-10-03) made
packs thin instances; the three standard packs (data/lineage,
numeric-predicates, empirical-registry — all archived 2026-10-03) proved
declaration over one's own sections and kinds. What none of them
exercised: vocabulary that *resolves against another pack's section*
(the numeric pack explicitly deferred `same_shape_as` as "the bioimage
pilot's data-shaped half"), a fiber kind whose natural home is a
base-table kind column (ung landed exactly that, 2026-10-03), and a
`## Requires` table with pack deps. The D6 pilot is the vendor design
that forces all three at once — R2's five-predicate grammar resolving
against `## Data`, mistral's kind-dependent law floor, grok-bioimage's
bridge-don't-absorb stance.

## Goals / Non-Goals

- **Goal:** the D6 pilot — a thin domain pack that stress-tests
  cross-pack resolution, base-table kind-column fiber typing, and
  cross-pack Requires, consuming the three standard packs' vocabulary
  rather than redeclaring any of it.
- **Goal:** zero core growth — no base closed set changes, no `src/`
  edits, no checker code.
- **Non-goal:** executing predicates (`dtype_is` etc. compile nowhere —
  `specodelic-rjb`'s track), parsing OME/NGFF, or validating checksums
  (R6, deferred).

## Decisions

### D1 — Thin vocabulary (one section, one kind, one field, four checkers, one floor, cross-pack Requires)

| Facet    | Declaration |
|----------|-------------|
| Sections | `## Axes` with row shape `\| name \| axis \| scale \| unit \|` — the workspace's declared image axes (OME-NGFF-style T/C/Z/Y/X semantics); `scale` and `unit` columns are opaque prose the format never parses (bridge-never-absorb) |
| Kinds    | `bioimage.transform` — a scientific image-transform law: a stated property about how a pipeline step transforms image data; a pack-fiber PROPERTY kind, typeable in a base Properties row's `kind` column when the pack is active (the ung mechanism — the pilot is the first pack whose kind naturally lands in a base kind column) |
| References | `same_shape_as` — resolves to a `## Data` row of the declaring file (the data-lineage pack's section); outbound leaf, intra-file resolution, the `measured_by`/`tested_by` precedent with a cross-pack *target section* |
| Checkers | `bioimage.dtype_is`, `bioimage.shape_eq`, `bioimage.units_convertible` — R2's data-shaped closed predicate grammar declared as named pack-qualified checkers over `## Data` rows: every predicate occurrence in a law predicate resolves against the closed set; with no `## Data` rows the checked-set is empty. Declaration-only — the mechanism can name them, never execute them. Plus `bioimage.same_shape_closed` (implementation fix-pass): the delta's dangling-`same_shape_as` requirement needs a declared closure checker (the `data.lineage.closure` precedent) — the scaffold's three-checker enumeration under-listed it |
| Floors   | `bioimage.transform` requires `preserves` + `dtype` case labels (the kind-dependent floor: what invariant the transform preserves, and its dtype contract) |
| Requires | `base` pinned at `specodelic.md Revision 14` (the mechanism revision) plus pack deps `data.lineage`, `numeric.predicates`, `empirical.registry` — the first cross-pack `## Requires` consumer |

Kept minimal deliberately; `append_only_packs` lets later releases add
the pipeline kind, further predicates, and R6 binding fields without
breaking this one.

### D2 — Vocabulary hygiene applied up front (the lesson, pre-paid three times)

Every vocabulary-carrying facet was audited against `grep -rlw` over
`specs/` and the dual-format `openspec/specs/` tree:

- `bioimage.transform`, `bioimage.dtype_is`, `bioimage.shape_eq`,
  `bioimage.units_convertible` — **absent everywhere** (0 files).
  Dotted, word-boundary-safe.
- Bare `bioimage` — **present in corpus prose** (3 + 3 files: USAGE
  §packs, STATUS, CHANGELOG + the three capability specs' mirrored
  text). Documented asymmetry, same as the numeric pack's `numeric.*`
  and the empirical pack's `empirical.*`: dotted tokens cannot match
  prose words (`contains_word` whole-token matching verified there), so
  prose mentions cannot falsely activate.
- `same_shape_as`, `dtype_is`, `shape_eq`, `units_convertible` —
  **absent** (0 files) — but the predicate names are never declared
  bare anyway: they ride the pack-qualified checker rule names, so even
  future prose use of the bare words stays inert.
- `Axes` — **absent** (0 files). CamelCase plural like `StatTests`,
  `Quantities`, `Data`; the residual-future-prose risk is the accepted
  trade of the sibling packs (append-only: a later release may qualify
  the section name, never un-declare it).
- `preserves` — **absent** (0 files); survives only as a floor case
  label (floors are not in `vocabulary()`).
- `dtype` — **present** (1 + 1 files — the data pack's own `## Data`
  column, which this pack consumes). Appears only as the `## Axes` row's
  opaque column header and a floor case label — neither columns nor
  floors are in `vocabulary()`, so no false activation is possible, and
  reusing the consumed pack's column name is consistency, not collision.
- `axis`, `scale` — **absent** (0 files); column headers only.
- `pipeline` — **prose-heavy** (9 + 1 files). Rejected outright (see
  Rejected alternatives); the kind is deferred to a later release.
- `validated_against` — **absent** (0 files) but R6-deferred, not
  declared here.

### D3 — Cross-pack resolution is intra-file: the pack is cross-pack, the check stays file-local

`same_shape_as` targets `## Data` — a section the data-lineage pack
declares — but resolves to a row **of the declaring file**, exactly like
`produced_by` and `measured_by` do. Cross-pack-ness lives in the
*dependency graph* (the `## Requires` pack deps) and in which pack's
vocabulary the target section belongs to, not in a corpus-wide
resolution pass: advisory-first, file-local checking, no reachability
join, no acyclic edge set. A dangling reference is a labeled finding
naming the row id and both remediations, never a generic dangling
message. A file referencing `## Data` rows without the data-lineage
pack active gets the orphan-vocabulary finding for that pack — the
mechanism's labeled failure, not silence.

### D4 — The kind-dependent floor makes mistral's critique concrete

The base `law` floor (identity/associativity) is meaningless for image
transforms — the flagged non-universality that motivated per-kind
floors. `bioimage.transform`'s floor requires `**preserves:**` and
`**dtype:**` case labels: what the transform preserves and the dtype
contract it holds. Purely additive — the base `law` floor is
byte-identical with the pack discovered; a kind with no declared floor
inherits none; floor labels never join the activation vocabulary. A
`bioimage.transform` property is itself floor-not-ceiling relative to
additive labels from the consumed packs (`**alpha:**`/`**window:**`
stat labels ride along when the empirical registry is active).

### D5 — Pack file lives at `packs/bioimage-data.md`

Corpus-scan discovery anchors at the git toplevel; `packs/` is the
convention all three prior packs set. File-naming law: stem
`bioimage-data` ⇔ id `bioimage.data`.

### D6 — Lifecycle: starts at the full three-state machine (published)

Same as all three prior packs: the mechanism reads the full
draft/published/deprecated Model as `published`; the artifact ships at
the steady state.

### D7 — The delta file's own activation is honest noise

Same as all three prior packs: the dual-format delta's mirrored
requirement text names the pack's kinds and fields, so linting the
`openspec` tree vocabulary-activates the pack for the `spec` delta file
— warnings channel only, exit 0, findings empty. It disappears when the
delta archives; the capability spec under `openspec/specs/` carries the
same text deliberately.

### D8 — Requires grows a pack-dep row shape (the first consumer, still declaration-only)

The three prior packs pinned only `base`. The mechanism spec
(`lifecycle_labeled`) already reads the `## Requires` table as pinning
"the base format_revision **and pack dependencies**" — the pilot is its
first consumer: dep rows name pack ids (`data.lineage`,
`numeric.predicates`, `empirical.registry`) with the base corpus
revision they were verified against. Declaration-only at this revision:
skew/lifecycle handling for pack deps is advisory-first machinery the
mechanism already sketches; nothing in `src/` grows here.

## Rejected alternatives

- **A `bioimage.pipeline` kind** — `pipeline` is prose-heavy (9 corpus
  files, vocabulary-carrying facets are prose-safe by construction) and
  mistral's `kind: pipeline` + handoff unification (rank ≤ 6) is
  mechanism work, not vocabulary; a later append-only release may add
  it.
- **A closed dtype enum** — grok-bioimage's bridge-don't-absorb stance:
  OME/NGFF and BioImage.IO stay authoritative; a closed dtype enum would
  absorb one external standard's vocabulary into the pack fiber. The
  `dtype_is` predicate names the *check*, the `dtype` column stays
  opaque.
- **Declaring `within`** — prose-toxic (13 corpus files); the numeric
  pack already ruled it out of every vocabulary-carrying facet.
- **A `## Data` redeclaration** ("own copy of the data table so the pack
  is self-contained") — redeclaring another pack's section is exactly
  the collision namespacing exists to prevent; the pack *consumes* the
  data-lineage vocabulary via Requires deps.
- **Cross-pack corpus-wide reference resolution** (`same_shape_as` →
  any file's `## Data` row) — turns an outbound leaf into a
  reachability join, breaks file-local advisory-first checking, and
  adds an acyclic edge set for no demand; intra-file resolution (D3)
  covers the vendor designs.

## Review outcome

Ro5 review at proposal time: verdict READY WITH_NOTES — findings folded
into this delta before implementation: **FIND-1 (substantive)** — the
predicate grammar had no facet home (predicates are not references, not
kinds); resolved by declaring them as named pack-qualified checker rows
(D1 Checkers), which gives them a vocabulary surface, keeps the bare
tokens undeclared, and reuses the existing Checkers row shape;
FIND-2 — `same_shape_as`'s cross-pack target initially read as a
corpus-wide join; tightened to intra-file resolution with the
pack-ness living in the Requires graph (D3), avoiding a new acyclic
edge set; FIND-3 — `dtype` reappearing as a floor label and Axes column
is intentional consistency with the consumed pack, verified non-activation
(columns/floors ∉ `vocabulary()`) and recorded in D2; FIND-4 — the
Requires pack-dep row shape was unspecified by the mechanism spec
("and pack dependencies" prose-only); D8 scopes the pilot's usage as
declaration-only and notes the skew semantics stay advisory;
FIND-5 — the empirical registry's role needed stating (why a transform
pack needs stat tests): `bioimage.transform` properties may be
empirically held (`empirical.statistic` typing + `tested_by` edges) —
recorded in D1's Requires row and D4's additivity note. No blocking
findings.

## Implementation fix-pass notes (2026-10-03, probes)

- **Closure checker added**: `bioimage.same_shape_closed` — the delta's
  dangling-`same_shape_as` requirement is unrealizable without a
  declared closure checker (pack-declared reference closure rides
  declared checkers; the `data.lineage.closure` precedent). D1's
  scaffold enumeration said three checkers; the artifact ships four.
- **Vocabulary surface corrected**: `vocabulary()` = kinds + sections +
  references only — checker rule names never join it (verified
  empirically; matches all three sibling packs). The task-2.2 surface
  expectation of six tokens corrects to three: `[bioimage.transform,
  Axes, same_shape_as]`.
- **Single-state variants**: pack_shape is clean in draft, published,
  and deprecated; the hand-stripped single-state variants carry two
  residual model-shape advisories (state row without transitions) —
  variant-construction artifacts, not pack findings.
- **Requires pack-dep advisory**: declaration-only at this revision (D8
  as scoped) — a dep naming no discovered pack fires nothing; recorded,
  not waved at.
- **Mechanism gap found (filed, not folded)**: vocabulary-triggered
  orphan labeling (pack vocabulary used, no pack discovered, no `uses`
  edge) is unimplemented in `packs.rs` — the orphan rule fires only on
  declared `uses` edges; `packs.md`'s `orphan_vocabulary_labeled`
  requires both halves. Filed as its own beads ticket: this change's
  no-`src/`-edit anti-goal holds.
