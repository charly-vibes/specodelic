# Design: add-numeric-predicates-pack

## Context

The pack mechanism (archived 2026-10-03) makes standard packs thin
instances — the mechanism interprets declared vocabulary, advisory-first.
R1+R2+R4 converged on the numeric shape: a typed quantity table with
units, real-valued bounds as vocabulary, and tolerance laws whose case
labels differ from the algebraic floor. The data/lineage pack proved the
scaffold end to end and surfaced the vocabulary-hygiene trap this design
front-loads.

## Goals / Non-Goals

- **Goal:** the numeric half of the standard-pack trio, completing what
  the bioimage D6 pilot consumes.
- **Goal:** zero core growth — no base closed set changes, no `src/`
  edits, no checker code.
- **Non-goal:** executing predicates. R2's compiled-invariants grammar
  (`within`, `units_convertible` as *executable* fragments) is
  `specodelic-rjb`'s predicate-fragment work; this pack only makes the
  vocabulary declarable and nameable.
- **Non-goal:** unit conversion, UCUM parsing, or any unit system in a
  closed set — `unit`/`domain` columns are opaque, like `binding`.

## Decisions

### D1 — Thin vocabulary (one section, three kinds, one field, two checkers, one floor)

| Facet    | Declaration |
|----------|-------------|
| Sections | `## Quantities` with row shape `\| name \| kind \| unit \| domain \|` |
| Kinds    | `numeric.quantity`, `numeric.bound`, `numeric.tolerance` — the R1/R4 closed trio |
| References | `measured_by` — resolves to a `## Quantities` row of the declaring file (outbound leaf, intra-file, the data-pack precedent) |
| Checkers | `numeric.quantity_closed` (every `measured_by` name resolves to a declared `## Quantities` row; honest-empty otherwise), `numeric.tolerance_labels` (tolerance laws enumerate `**bound:**` and `**against:**` case labels) |
| Floors   | `numeric.tolerance` requires `bound` + `against` case labels (R4's reuse phrasing) |
| Requires | `base` pinned at `specodelic.md Revision 14` (the mechanism revision) |

Kept minimal deliberately; `append_only_packs` lets later releases add
predicates and unit machinery without breaking this one.

### D2 — Vocabulary hygiene applied up front (the D6 lesson, pre-paid)

`add-data-lineage-pack`'s design D6 cut bare `produces` after dogfooding
caught corpus-prose false activation. This pack applies the lesson
before writing the artifact: every vocabulary-carrying facet was
audited against `grep -rcw` over `specs/` and the dual-format
`openspec/specs/` tree.

- `within` — **excluded**: 13 corpus files + 7 dual-format files use the
  English word. The `within`-class bound is instead carried by the
  `numeric.bound` kind (a `## Quantities` row of kind `numeric.bound`
  names its bound and tolerance); a later release may add a predicate
  name once activation can disambiguate.
- `bound`, `against`, `unit`, `domain`, `metric`, `measure` — appear in
  the corpus; none of them is declared as vocabulary. `bound`/`against`
  survive only as **floor case labels** (`**bound:**` / `**against:**`
  in law predicates) — floors are not in `vocabulary()` (the mechanism
  scans kinds + sections + references only), so they cannot trigger
  activation.
- `measured_by` — verified absent from the corpus (safe; `measures`
  was also absent but `measured_by` reads better as a directed edge).
- `Quantities` (section token) — verified absent from the corpus today.
  Residual risk, accepted: future corpus prose that capitalizes
  "Quantities" as a word would falsely activate the pack — the same
  accepted trade the data pack makes with its `Data` section token;
  `append_only_packs` means a later release can rename/qualify the
  section name but never un-declare it, so the token was chosen at its
  safest form (plural, capitalized — matching the `## Data` precedent
  where the accepted risk is documented in that pack's D6/D7).
- `numeric.*` kinds are dotted and word-boundary-safe.

### D3 — Quantity rows resolve intra-file, outbound-leaf

`measured_by` resolves to a `## Quantities` row of the declaring file —
file-local checking (advisory-first, no corpus-wide pass), no
reachability join, no acyclic edge set (the `uses`/`produced_by`
precedent). A dangling reference is a labeled finding naming the row id
and both remediations, never a generic dangling message.

### D4 — Unit and domain columns stay opaque

The `unit` and `domain` columns carry typed prose pointers (UCUM codes,
ISO4217 currencies, microscopy units) that the format never parses. This
is the grok bridge-never-absorb counter-position made structural, same as
the data pack's `binding`. Lint behavior is byte-identical regardless of
the unit system named.

### D5 — Pack file lives at `packs/numeric-predicates.md`

Corpus-scan discovery anchors at the git toplevel; a dedicated `packs/`
directory is convention (the data pack set it). File-naming law: stem
`numeric-predicates` ⇔ id `numeric.predicates`.

### D6 — Lifecycle: starts at the full three-state machine (published)

The mechanism reads the full draft/published/deprecated Model as
`published`; single-state variants were proven pack_shape-clean for the
data pack and hold here too (same `pack_shape` rule, state-independent).
The artifact ships directly at the steady state; the publish/deprecate
transitions stay declared for the lifecycle machinery.

### D7 — The delta file's own activation is honest noise

Same as the data pack's D7: the dual-format delta's mirrored requirement
text names the pack's kinds and fields, so linting the `openspec` tree
vocabulary-activates the pack for the `spec` delta file — warnings
channel only, exit 0, findings empty. It disappears when the delta
archives; the capability spec under `openspec/specs/` carries the same
text deliberately.

## Rejected alternatives

- **Declaring the R2 five-predicate grammar** — `dtype_is`/`shape_eq`/
  `same_shape_as` are data-shaped (they resolve against `## Data`, the
  other pack's section) and would need a cross-pack `## Requires` dep
  plus a `## Data`-aware checker contract; that is the bioimage pilot's
  job to stress-test. `within` is prose-toxic (D2). The numeric pack
  stays domain-neutral.
- **A `## Units` registry table** — every vendor's unit layer disagreed
  (UCUM vs ISO4217 vs microscopy); a closed registry would absorb one
  vendor's system. Opaque columns keep all of them authoritative.
- **Bare `measures` instead of `measured_by`** — safe per the audit,
  but `measured_by` states the direction (constraint row → quantity row)
  and reads unambiguously next to `produced_by`/`consumed_by`.

## Review outcome

Ro5 review at proposal time: verdict READY WITH_NOTES — findings folded
back into this delta before implementation (D2's pre-paid hygiene audit,
D4's opacity consequence, D6's lifecycle simplification, D7's honest
noise).