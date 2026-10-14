# Worked example — the numeric.predicates pack, failures included

The [generic worked example](worked-example.md) builds a spec with the
core format alone, and [the data.lineage page](data-lineage.md) does the
same for the first domain pack. This page runs the identical journey for
**[`numeric.predicates`](https://github.com/charly-vibes/specodelic/tree/main/packs/numeric-predicates.md)**
— the pack that adds a typed `## Quantities` section, the
`numeric.quantity` / `numeric.bound` / `numeric.tolerance` kinds, a
`measured_by` reference field, and a case-label floor for tolerance laws
— without a format revision and without growing any base closed set.
You will see a small spec use the pack's vocabulary end to end, the
pack-activation advisory that proves the pack actually engaged, two
genuinely failing variants with their real labeled findings, and — just
as important — the failures the linter does *not* catch yet, shown
honestly.

How packs are enabled and what a pack adds mechanically is the job of
[using packs — the practical guide](../using-packs.md); read that for the
mechanism, this page for the hands-on loop. Every lint and graph block
below is real output from `specodelic 0.10.0` on the file as it stood at
that moment. On a TTY `spk` prints the human-readable text shown here;
piped, it emits a JSON envelope instead.

## The story

A brewery runs a fermentation line: every batch measures mash
temperature and pH, derives an ABV yield from a gas-chromatography
assay, and must respect a regulatory ethanol cap and a pH window. The
spec's job is to record those quantities and the limits that govern
them — one capability, one file, named `batch-yield.md`; the format's
naming law maps `-` to the namespace dot, so the frontmatter id is
`batch.yield`.

Deliberately out of scope: *which* unit system the numbers are written
in and *where* each domain of validity is defined. Those are the
`unit` and `domain` columns' job, and they stay opaque — the pack
bridges external unit systems (UCUM, ISO 4217, microscopy units, …), it
never absorbs them. Lint behavior is byte-identical no matter what
prose the columns carry.

## The setup

Packs are discovered by scanning the workspace, not by any config file:
every `kind: profile` file in the git repository toplevel of the linted
spec is a candidate pack. So the demo workspace needs two files — the
pack definition (copied from this repo's `packs/` directory) and the
spec under test:

```sh
mkdir demo && cd demo && git init
cp /path/to/specodelic/packs/numeric-predicates.md .
```

That is the whole enablement story. The discovery and activation rules
(corpus-scan anchors, word-boundary vocabulary matching, the
byte-identical guarantee) are in [the guide](../using-packs.md); this
page just runs the loop.

## Layer 1 — Intent

```markdown
---
id: batch.yield
kind: intent
statement: "WHEN a fermentation batch is processed THE system SHALL record every quantity it measures or derives together with the bounds and tolerances that govern them."
---

# Batch yield quantities

A small numeric record for one fermentation line: the process measures
mash temperature and pH, derives an ABV yield from a gas-chromatography
assay, and every stated limit is a bound or a tolerance with a unit and
a domain of validity.
```

Lint it as it stands:

```sh
spk lint batch-yield.md
```

```text
lint: 1 file(s) linted, 1 finding(s), 0 advisory warning(s)
  batch-yield.md [linter.model_present] Model section incomplete: 0 states, 0 transitions — both sections must exist (empty-but-present beats absent)
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

`linter.model_present` — a spec with an intent but no Model section is
incomplete by construction, same as in the generic worked example. Note
what is *missing* from this output: no pack advisory. The file uses none
of the pack's vocabulary yet, so the byte-identical guarantee holds —
the pack machinery is invisible. That changes with the next section.

## Layer 2 — the `## Quantities` section

This is the pack's namespaced section, declared in the pack's
`## Sections` manifest with the fixed row shape
`| name | kind | unit | domain |`:

```markdown
## Quantities

| name          | kind              | unit                     | domain     |
|---------------|-------------------|--------------------------|------------|
| mash_temp     | numeric.quantity  | degC (UCUM Cel)          | process    |
| ph            | numeric.quantity  | pH (dimensionless)       | process    |
| ethanol_assay | numeric.quantity  | percent v/v (UCUM [pct]V)| product    |
| abv_yield     | numeric.quantity  | percent v/v (UCUM [pct]V)| product    |
| ethanol_cap   | numeric.bound     | percent v/v (UCUM [pct]V)| regulatory |
| ph_window     | numeric.tolerance | pH                       | process    |
```

Each column, defined for a cold read:

- **name** — the row's id inside this file; `measured_by` edges point
  at it.
- **kind** — the pack's own closed set: `numeric.quantity` (a measured
  or derived real-valued quantity), `numeric.bound` (a stated
  real-valued limit on a quantity), `numeric.tolerance` (an empirically
  held tolerance on a quantity). These are *fiber kinds* —
  pack-declared vocabulary, never a growth of the base closed sets.
- **unit** / **domain** — typed prose pointers to the unit system and
  the domain of validity. The pack declares no syntax for either and
  the linter never interprets them: external unit systems stay
  authoritative, and lint behavior is identical regardless of what the
  columns say.

Edges come from the pack's single reference field, declared in its
`## References` manifest — `measured_by`, resolving to a `## Quantities`
row of the same file. It rides as a column on a second table under the
same section:

```markdown
| row       | measured_by  |
|-----------|--------------|
| abv_yield | ethanol_assay|
```

Read the edge aloud: the derived `abv_yield` is measured by the
`ethanol_assay` quantity — the assay is the quantity's source of
evidence, and it is itself a declared row.

The pack also declares two checkers — `numeric.quantity_closed`, *every
`measured_by` name resolves to a `## Quantities` row declared in the
same file*, and `numeric.tolerance_labels`, *`numeric.tolerance` laws
enumerate their `bound` and `against` case labels per the pack's floor*
— and one floor: law-kind properties about `numeric.tolerance` rows
must carry **`bound:`** and **`against:`** case labels. The Properties
layer below pays that floor in full.

## Layer 3 — Constraints, Model, Properties

The rest of the file is the ordinary four-layer shape — the pack adds
vocabulary, it does not change the layers. Two invariants state the
quantity claims, an `effect` constraint labels the failure output, an
`observes` edge ties the observer to the effect, and the model is a
five-state batch lifecycle whose failure terminal emits. The Properties
table includes one `law` row, `window_cases`, which enumerates *four*
case labels: the base format's mandatory **`identity:`** and
**`associativity:`** floor (`linter.law_cases` demands those of every
law) plus the pack floor's **`bound:`** and **`against:`**:

```markdown
## Constraints

| id                | kind      | expr                                                                                      | traces_to        | observes                    |
|-------------------|-----------|-------------------------------------------------------------------------------------------|------------------|-----------------------------|
| rows_declared     | invariant | `every quantity row is declared in the ## Quantities table of this file`                   | [[batch.yield]]  |                             |
| yield_measured    | invariant | `every derived quantity names exactly one assay quantity as its measured_by source`        | [[batch.yield]]  |                             |
| cap_breach        | effect    | `batch.yield.cap_breach(detail)`                                                          | [[batch.yield]]  |                             |
| breach_surfaced   | invariant | `every bound breach reaches the operator with the quantity and the offending bound named`  | [[batch.yield]]  | [[batch.yield.cap_breach]]  |

## Model

### States

- `mashed`
- `fermented`
- `assayed`
- `released`
- `failed` (emits: `[[batch.yield.cap_breach]]`)

### Transitions

| id      | from      | to        | guard                                                    |
|---------|-----------|-----------|-----------------------------------------------------------|
| ferment | mashed    | fermented | `[[batch.yield.rows_declared]]`                           |
| assay   | fermented | assayed   | `[[batch.yield.yield_measured]]`                          |
| release | assayed   | released  | `abv_yield within ethanol_cap`                            |
| fail    | mashed    | failed    | `¬[[batch.yield.rows_declared]]`                          |

## Properties

| id               | kind | derives_from                     | generator                      | predicate                                                       |
|------------------|------|-----------------------------------|--------------------------------|-----------------------------------------------------------------|
| rows_resolvable  | unit | [[batch.yield.rows_declared]]     | `batches_with_quantity_rows()` | `every measured_by name resolves to a declared ## Quantities row`|
| assay_recorded   | unit | [[batch.yield.yield_measured]]    | `batches_with_derived_quantities()` | `every derived quantity carries its assay source`            |
| breach_emitted   | unit | [[batch.yield.cap_breach]]        | `batches_with_bound_breaches()`| `the failure carries the cap_breach label with detail`           |
| breach_named     | unit | [[batch.yield.breach_surfaced]]   | `batches_with_bound_breaches()`| `the operator message names the quantity and the bound`          |
| window_cases     | law  | [[batch.yield.rows_declared]]     | `batches_with_tolerance_rows()` | `the tolerance holds in every declared case — **identity:** a quantity measured against itself is within any window; **associativity:** composing assays is order-independent; **bound:** the lower and upper pH bounds hold; **against:** readings outside the window fail against the stated tolerance` |
```

Nothing pack-specific in the layers themselves — `traces_to`, guards,
`emits`, coverage: all core-format machinery, taught properly in the
[generic worked example](worked-example.md). Omit either pair of case
labels from `window_cases` and `linter.law_cases` names the missing
floor cases exactly.

## First full lint — a dangling reference, then the fix

The author's first full draft carries one typo: the `rows_declared`
constraint traces to `[[batch.yiled]]` — plural, misremembered, the
same classic as the reservation example. Lint the file:

```sh
spk lint batch-yield.md
```

```text
lint: 1 file(s) linted, 1 finding(s), 1 advisory warning(s)
  batch-yield.md [linter.total_refs] dangling reference `[[batch.yiled]]` from constraints.traces_to — target not defined in any spec file
  advisory batch.yield [linter.observability] pack `numeric.predicates` activated for `batch.yield` (vocabulary match: numeric.quantity, numeric.bound, numeric.tolerance, Quantities, measured_by) — empty checked-set (no violations to report)
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

Exit 1 — a finding, never a silent pass. Two things to read here:

1. **The finding and the advisory ride different channels.** The pack
   activated (its vocabulary is all over the file — note the match list
   names every token that triggered it), but activation lives on the
   *warnings* channel and never gates. The dangling reference lives on
   the *findings* channel and does.
2. **`empty checked-set (no violations to report)`** — the pack's
   declared checkers ran and honestly report an empty violation set.
   More on what that does and does not mean below.

Fix the typo — `[[batch.yiled]]` → `[[batch.yield]]` — and lint again:

```text
lint: 1 file(s) linted, 0 finding(s), 1 advisory warning(s)
  advisory batch.yield [linter.observability] pack `numeric.predicates` activated for `batch.yield` (vocabulary match: numeric.quantity, numeric.bound, numeric.tolerance, Quantities, measured_by) — empty checked-set (no violations to report)
→ Run: specodelic graph
```

Zero findings, exit 0 — and now the advisory is the *whole* output.
Read it as the meter it is: an exit-0 lint alone proves nothing (a spec
using none of the pack vocabulary also exits 0, silently); **the
activation line naming `numeric.predicates` is the observable** that
the pack was discovered, matched, and engaged. The pipeline's next
stage runs pack-aware too:

```sh
spk graph batch-yield.md
```

```text
graph: 1 file(s), 18 node(s), 21 edge(s), 0 dangling, 0 typing violation(s), 0 supersedes cycle(s), 0 external boundary(ies)
→ Run: specodelic lint
```

## What a missing pack looks like

The demo depends on the pack file being *discovered*. Copy the finished
spec into a directory with no `kind: profile` file in it — the
machinery goes inert:

```text
lint: 1 file(s) linted, 0 finding(s), 0 advisory warning(s)
→ Run: specodelic graph
```

No advisory, no findings — the byte-identical guarantee in action. The
`## Quantities` table is now just markdown the linter does not
interpret, and that is by design: a pack can never gate a file that did
not opt in. But inert cuts both ways, and the linter tells you when
vocabulary *expects* a pack. Say the author, pack-enthused, writes a
base-table `kind` cell as `numeric.quantity` instead of `invariant`,
in that same pack-less directory:

```text
lint: 1 file(s) linted, 2 finding(s), 0 advisory warning(s)
  batch.yield [linter.constraint_kind_closed] constraint `rows_declared` has kind `numeric.quantity` — outside the closed set {invariant, advisory, effect, extension_point} (kinds.md constraint_row_shape)
  batch.yield [linter.orphan_vocabulary] orphan vocabulary: pack-qualified token(s) `numeric.quantity` used by `batch.yield`, but no `kind: profile` pack in namespace `numeric` is discovered — candidate pack `numeric.*` (prefix-derived: only the namespace is known); remediations: add/enable a `kind: profile` pack file declaring the vocabulary, or fix the vocabulary (un-qualify or retype the token)
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

`linter.orphan_vocabulary` — the finding always names the candidate
(namespace-derived, `numeric.*` here) and states **both** remediations:
enable a pack declaring the vocabulary, or fix the token. Copy the pack
file back into the workspace and the orphan finding clears — with the
pack active, the walkers accept a fiber kind in a kind column (base ∪
active-pack-fiber), so that particular mis-typing only surfaces when
the pack is missing. The reliable diagnostic is the one this section
started with: a spec you *meant* to be pack-enabled but lint with no
activation advisory is a spec whose pack was not discovered.

## What the linter does not catch (yet)

Honesty check — the failure this page has *not* shown is the one you
might expect first: a `measured_by` edge that dangles. Change
`ethanol_assay` to `ghost_assay` in the `measured_by` column and lint:

```text
lint: 1 file(s) linted, 0 finding(s), 1 advisory warning(s)
  advisory batch.yield [linter.observability] pack `numeric.predicates` activated for `batch.yield` (vocabulary match: numeric.quantity, numeric.bound, numeric.tolerance, Quantities, measured_by) — empty checked-set (no violations to report)
→ Run: specodelic graph
```

Byte-for-byte the clean output — exit 0, activation advisory, nothing
else. That is not the linter missing the dangle by accident:
`numeric.quantity_closed` ships as a **declaration**. In v1, pack
checkers are honest-empty — activation and the empty checked-set are
reported truthfully, violation *evaluation* is not implemented yet (the
[guide](../using-packs.md) documents this, and
[verification boundaries](../verification-boundaries.md) places it on
the guarantee ladder). The format's answer to the gap is the one this
spec already embodies: the dangling-edge claim lives where the evidence
plan lives — the `rows_declared` invariant, its `rows_resolvable`
property, and eventually a real executable predicate. When the checker
gains evaluation, the declaration is already in place; the spec does not
change.

## The finished spec

Every block above assembles into this one file, `batch-yield.md` —
exactly as it lints at 0 findings, 1 activation advisory:

```markdown
---
id: batch.yield
kind: intent
statement: "WHEN a fermentation batch is processed THE system SHALL record every quantity it measures or derives together with the bounds and tolerances that govern them."
---

# Batch yield quantities

A small numeric record for one fermentation line: the process measures
mash temperature and pH, derives an ABV yield from a gas-chromatography
assay, and every stated limit is a bound or a tolerance with a unit and
a domain of validity.

## Quantities

| name          | kind              | unit                     | domain     |
|---------------|-------------------|--------------------------|------------|
| mash_temp     | numeric.quantity  | degC (UCUM Cel)          | process    |
| ph            | numeric.quantity  | pH (dimensionless)       | process    |
| ethanol_assay | numeric.quantity  | percent v/v (UCUM [pct]V)| product    |
| abv_yield     | numeric.quantity  | percent v/v (UCUM [pct]V)| product    |
| ethanol_cap   | numeric.bound     | percent v/v (UCUM [pct]V)| regulatory |
| ph_window     | numeric.tolerance | pH                       | process    |

| row       | measured_by  |
|-----------|--------------|
| abv_yield | ethanol_assay|

## Constraints

| id                | kind      | expr                                                                                      | traces_to        | observes                    |
|-------------------|-----------|-------------------------------------------------------------------------------------------|------------------|-----------------------------|
| rows_declared     | invariant | `every quantity row is declared in the ## Quantities table of this file`                   | [[batch.yield]]  |                             |
| yield_measured    | invariant | `every derived quantity names exactly one assay quantity as its measured_by source`        | [[batch.yield]]  |                             |
| cap_breach        | effect    | `batch.yield.cap_breach(detail)`                                                          | [[batch.yield]]  |                             |
| breach_surfaced   | invariant | `every bound breach reaches the operator with the quantity and the offending bound named`  | [[batch.yield]]  | [[batch.yield.cap_breach]]  |

## Model

### States

- `mashed`
- `fermented`
- `assayed`
- `released`
- `failed` (emits: `[[batch.yield.cap_breach]]`)

### Transitions

| id      | from      | to        | guard                                                    |
|---------|-----------|-----------|-----------------------------------------------------------|
| ferment | mashed    | fermented | `[[batch.yield.rows_declared]]`                           |
| assay   | fermented | assayed   | `[[batch.yield.yield_measured]]`                          |
| release | assayed   | released  | `abv_yield within ethanol_cap`                            |
| fail    | mashed    | failed    | `¬[[batch.yield.rows_declared]]`                          |

## Properties

| id               | kind | derives_from                     | generator                      | predicate                                                       |
|------------------|------|-----------------------------------|--------------------------------|-----------------------------------------------------------------|
| rows_resolvable  | unit | [[batch.yield.rows_declared]]     | `batches_with_quantity_rows()` | `every measured_by name resolves to a declared ## Quantities row`|
| assay_recorded   | unit | [[batch.yield.yield_measured]]    | `batches_with_derived_quantities()` | `every derived quantity carries its assay source`            |
| breach_emitted   | unit | [[batch.yield.cap_breach]]        | `batches_with_bound_breaches()`| `the failure carries the cap_breach label with detail`           |
| breach_named     | unit | [[batch.yield.breach_surfaced]]   | `batches_with_bound_breaches()`| `the operator message names the quantity and the bound`          |
| window_cases     | law  | [[batch.yield.rows_declared]]     | `batches_with_tolerance_rows()` | `the tolerance holds in every declared case — **identity:** a quantity measured against itself is within any window; **associativity:** composing assays is order-independent; **bound:** the lower and upper pH bounds hold; **against:** readings outside the window fail against the stated tolerance` |
```

To reproduce every block on this page: copy
`packs/numeric-predicates.md` from the repo into a fresh git workspace,
save the file above as `batch-yield.md`, and run
`spk lint batch-yield.md`.

## Where to go next

- The mechanism, end to end: [using packs — the practical
  guide](../using-packs.md) — discovery, activation, honest-empty
  semantics, the full CLI loop.
- The pack's normative home: [packs — domain-pack
  mechanism](../specs/packs/spec.md), and the pack definition itself —
  [`packs/numeric-predicates.md`](https://github.com/charly-vibes/specodelic/tree/main/packs/numeric-predicates.md)
  is the source of truth for the vocabulary this page used.
- The capability spec behind the pack:
  [numeric-predicates-pack](../specs/numeric-predicates-pack/spec.md).
- The same journey without packs: [worked example —
  reservation/order](worked-example.md), and with the sibling pack:
  [the data.lineage pack](data-lineage.md).
- Rule semantics offline: `spk explain packs` and
  `spk explain lint-rules`.
