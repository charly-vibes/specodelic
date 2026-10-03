# Design: add-empirical-registry-pack

## Context

The pack mechanism (archived 2026-10-03) makes standard packs thin
instances — the mechanism interprets declared vocabulary, advisory-first.
R4 converged on the statistical shape: an empirically held property kind
and per-kind required case-label floors reusing the `**name:**`
enumeration machinery the `update-law-named-cases` Revision made
machine-checkable. The Floors manifest facet has shipped with two
declared instances (data.artifact → `provenance`; numeric.tolerance →
`bound, against`) but no floor *whose required labels differ in kind* —
this pack is the Floors facet's first real consumer: a floor driven by
statistical practice (significance + observation window), not algebra.

## Goals / Non-Goals

- **Goal:** the third standard pack, completing what the bioimage D6
  pilot consumes (data/lineage + numeric predicates + empirical registry).
- **Goal:** zero core growth — no base closed set changes, no `src/`
  edits, no checker code.
- **Non-goal:** executing statistical tests or parsing alpha values.
  The pack makes the vocabulary declarable and nameable; the numeric
  predicates' executable-fragment track is `specodelic-rjb`.
- **Non-goal:** R6 dataset/benchmark binding (`validated_against`,
  checksums) — a separate facet with its own fail-closed semantics.

## Decisions

### D1 — Thin vocabulary (one section, one kind, one field, two checkers, one floor)

| Facet    | Declaration |
|----------|-------------|
| Sections | `## StatTests` with row shape `\| name \| metric \| alpha \| window \|` — the registry table where a workspace declares its statistical tests: what they measure (metric), their significance level (alpha), their observation window (window) |
| Kinds    | `empirical.statistic` — an empirically held statistical claim, stated with a declared stat test; a pack-fiber PROPERTY kind, typeable in a base Properties row's `kind` column when the pack is active (see D8 — the mechanism follow-up that makes base-table kind columns accept active-pack fiber kinds) |
| References | `tested_by` — resolves to a `## StatTests` row of the declaring file (outbound leaf, intra-file, the `measured_by`/`produced_by` precedent) |
| Checkers | `empirical.statistic_labels` (statistic properties enumerate their `**alpha:**` and `**window:**` case labels per the floor), `empirical.stat_test_closed` (every `tested_by` name resolves to a declared `## StatTests` row; honest-empty otherwise) |
| Floors   | `empirical.statistic` requires `alpha` + `window` case labels (R4's mistral phrasing) |
| Requires | `base` pinned at `specodelic.md Revision 14` (the mechanism revision) |

Kept minimal deliberately; `append_only_packs` lets later releases add
stat-test vocabularies, binding fields, and further floors without
breaking this one.

### D2 — Vocabulary hygiene applied up front (the D6 lesson, pre-paid twice)

Every vocabulary-carrying facet was audited against `grep -rlw` over
`specs/` and the dual-format `openspec/specs/` tree:

- `empirical.statistic` — **absent everywhere** (0 files). Dotted,
  word-boundary-safe. Bare `empirical` appears in corpus prose (USAGE
  §2.8, STATUS, AGENTS) but the activation scanner matches whole
  dotted tokens — bare `empirical` cannot falsely activate; documented
  as the same accepted asymmetry as the numeric pack's `numeric.*`.
- `StatTests` — **absent** (0 files). CamelCase compound, not a bare
  English word; the residual-future-prose risk is the accepted trade
  the data pack makes with `Data` and the numeric pack with
  `Quantities` (append-only: a later release may qualify the section
  name, never un-declare it).
- `tested_by` — **absent** (0 files). States the direction (property
  row → stat-test row) and reads unambiguously next to
  `measured_by`/`produced_by`.
- `stat_test` — **absent** (0 files); survives only as the natural
  name of the row-shape concept in prose, never as a declared token.
- `window` — **present in corpus prose** (1 spec file: the rate-limit
  window). Excluded from every vocabulary-carrying facet; survives only
  as a floor case label (`**window:**`) and a table column header —
  neither floors nor columns are in `vocabulary()`, so no false
  activation is possible.
- `alpha` — **absent** (0 files); same treatment as `window` (floor
  label + column only).

### D3 — Stat-test rows resolve intra-file, outbound-leaf

`tested_by` resolves to a `## StatTests` row of the declaring file —
file-local checking (advisory-first, no corpus-wide pass), no
reachability join, no acyclic edge set. A dangling reference is a
labeled finding naming the row id and both remediations, never a
generic dangling message.

### D4 — The floor registry is additive enumeration, not a base-floor edit

The per-kind floor reuses the `**name:**` enumeration machinery with a
different required label set; the base `law` floor
(identity/associativity/commutativity/idempotence) is untouched and
unchanged — floors are per-kind in the *pack fiber*, so `linter-law`
semantics for base `law` properties are byte-identical with the pack
discovered. A property kind with no declared floor inherits no floor
(the floor registry is opt-in per kind, never a default).

### D5 — Pack file lives at `packs/empirical-registry.md`

Corpus-scan discovery anchors at the git toplevel; `packs/` is the
convention the two prior packs set. File-naming law: stem
`empirical-registry` ⇔ id `empirical.registry`.

### D6 — Lifecycle: starts at the full three-state machine (published)

Same as both prior packs (D6 there): the mechanism reads the full
draft/published/deprecated Model as `published`; single-state variants
were proven pack_shape-clean. The artifact ships at the steady state.

### D7 — The delta file's own activation is honest noise

Same as both prior packs: the dual-format delta's mirrored requirement
text names the pack's kinds and fields, so linting the `openspec` tree
vocabulary-activates the pack for the `spec` delta file — warnings
channel only, exit 0, findings empty. It disappears when the delta
archives; the capability spec under `openspec/specs/` carries the same
text deliberately.

### D8 — Kind columns learn pack-fiber vocabulary (mechanism follow-up, filed)

The R4 convergence is a *property kind* in the pack fiber — its natural
home is a base Properties row's `kind` column. The mechanism v1 shipped
activation + declarations only: `property_kind_closed` walks the static
base set `{unit, law}` (`guide::PROPERTY_KINDS`) and would reject
`empirical.statistic` even with the pack discovered. Filling that is a
mechanism capability (R8 lineage — the closed-set walkers consult
discovered packs' fiber vocabulary), not core growth: no base set grows,
the effective set is `base ∪ active-pack-fiber`, and the sibling packs
sidestepped it only because their kinds live in pack-added sections.
Filed as its own beads ticket (kind-column fiber acceptance); this pack
ships vocabulary + floor declarations, and its delta states the
end-state contract — the same contract-vs-declaration-only depth the
siblings' `measured_by`/`data.lineage.closure` rows already carry.
Until the follow-up lands, a `law`-kind row using the pack's vocabulary
can carry `**alpha:**`/`**window:**` labels additively (the base floor
is a floor, not a ceiling) — usable today, retypable when the mechanism
arrives.

## Rejected alternatives

- **A second kind (`empirical.bound`)** — overlaps the numeric pack's
  `numeric.bound`/`numeric.tolerance`; cross-pack vocabulary overlap is
  legal but redundant here, and grok's "scaling generators" are
  executable semantics that belong to `specodelic-rjb`'s
  predicate-fragment track, not to a vocabulary declaration.
- **A `## Datasets` section with checksums** — R6's binding layer;
  absorbs fail-closed verify semantics this pack must not own.
- **Bare `statistic` as the kind name** — pack_shape rejects
  unqualified kinds and the base-set collision risk (grok-quant put
  `tolerance` on Constraint.kind, mistral-science on Property kinds —
  the exact collision namespacing dissolves) is real; dotted
  `empirical.statistic` is fiber-relative by construction.
- **Making the floor registry a cross-pack rule** ("every pack
  declaring an empirical.* kind must declare its floor") — that is a
  checker over other packs' manifests, i.e. governance code; the
  mechanism's pack_shape already validates Floors row shape, and a
  kind without a floor simply inherits none (D4).

## Review outcome

Ro5 review at proposal time: verdict READY WITH_NOTES — findings folded
into this delta before implementation: **FIND-1 (substantive)** — the
`empirical.statistic` fiber kind is not typeable in base Properties
`kind` columns under mechanism v1 (`property_kind_closed` walks the
static base set) → D8 added, mechanism follow-up filed as its own
beads ticket, delta gains the `fiber_kinds_typeable` constraint +
requirement, and the additive-labels-today path is stated; FIND-2 —
the floor's binding clarified as per-fiber-kind with the additive
(no-ceiling) reading for base `law` rows spelled out in D4/D8; FIND-3 —
D2's audit extended to the bare `empirical` asymmetry (dotted tokens
cannot match prose words, `contains_word` verified); FIND-4 — the
rejected `## Datasets` alternative recorded with its R6 rationale;
FIND-5 — the section token's residual-future-prose risk stated in D2
with the append-only consequence. No blocking findings.
