# Worked example — the bioimage.data pack, failures included

The [generic worked example](worked-example.md) builds a spec with the
core format alone, and the [data.lineage](data-lineage.md),
[numeric.predicates](numeric-predicates.md), and
[empirical.registry](empirical-registry.md) pages run the same journey
pack by pack. This page does it for
**[`bioimage.data`](https://github.com/charly-vibes/specodelic/tree/main/packs/bioimage-data.md)**
— the pack mechanism's *pilot*: the thin instance that stress-tests what
the packs it consumes could not exercise alone, and the mechanism's
first cross-pack `## Requires` consumer. It adds a typed `## Axes`
section (image axes with OME-NGFF-style semantics), the
`bioimage.transform` fiber property kind, a `same_shape_as` reference
field resolving to `## Data` rows, a named data-shaped predicate
grammar (`bioimage.dtype_is`, `bioimage.shape_eq`,
`bioimage.units_convertible`), and a kind-dependent case-label floor
enumerating **`preserves:`** and **`dtype:`** — all without a format
revision, without growing any base closed set, and while OME/NGFF and
BioImage.IO stay authoritative external standards. You will see a small
spec use the pack's vocabulary end to end, the pack-activation advisory
that proves the pack actually engaged, genuinely failing variants with
their real labeled findings, and — just as important — the failures the
linter does *not* catch yet, shown honestly.

How packs are enabled and what a pack adds mechanically is the job of
[using packs — the practical guide](../using-packs.md); read that for
the mechanism, this page for the hands-on loop. Every lint and graph
block below is real output from `specodelic 0.10.0` on the file as it
stood at that moment. On a TTY `spk` prints the human-readable text
shown here; piped, it emits a JSON envelope instead.

## The story

An imaging facility runs a light-sheet pipeline: raw microscopy frames
land from the microscope, a flat-field correction is applied, and the
corrected volume is written as an OME/NGFF zarr store. The spec's job
is to record that conversion — the image axes every dataset lives on,
which datasets share a voxel geometry, and the law each transform step
asserts about how it reshapes image data. One capability, one file,
named `plate7-ngff.md`; the format's naming law maps `-` to the
namespace dot, so the frontmatter id is `plate7.ngff`.

Deliberately out of scope: *how* OME/NGFF structures the zarr store and
*what* BioImage.IO says about the acquisition metadata. Those are the
opaque columns' job — `scale`, `unit`, and `binding` are prose the
format never parses, keeping OME/NGFF and BioImage.IO authoritative
outside the format. The pack bridges them; it never absorbs them. Lint
behavior is byte-identical no matter what those columns carry.

## The setup

Packs are discovered by scanning the workspace, not by any config file:
every `kind: profile` file in the git repository toplevel of the linted
spec is a candidate pack. `bioimage.data` is a *consumer* pack — its
`## Requires` table pins the base format revision plus the packs whose
vocabulary it reuses (`data.lineage` for the `## Data` section,
`numeric.predicates` for the opaque unit domain its
`bioimage.units_convertible` checker bridges, `empirical.registry` for
empirically held transform claims). So the demo workspace copies the
pack under test and the three it consumes:

```sh
mkdir demo && cd demo && git init
cp /path/to/specodelic/packs/{bioimage-data,data-lineage,numeric-predicates,empirical-registry}.md .
```

That is the whole enablement story. The discovery and activation rules
(corpus-scan anchors, word-boundary vocabulary matching, the
byte-identical guarantee) are in [the guide](../using-packs.md); this
page just runs the loop.

## Layer 1 — Intent

```markdown
---
id: plate7.ngff
kind: intent
statement: "WHEN the imaging pipeline processes a light-sheet plate THE system SHALL record the image axes of every dataset it writes together with the transform laws each pipeline step asserts about how it reshapes image data."
---

# Plate 7 NGFF conversion

A small bioimage record for one light-sheet run: the pipeline acquires
raw microscopy frames, applies flat-field correction, writes an
OME/NGFF zarr store, and every transform step states the law it obeys —
what it preserves and in which dtype.
```

Lint it as it stands:

```sh
spk lint plate7-ngff.md
```

```text
lint: 1 file(s) linted, 1 finding(s), 0 advisory warning(s)
  plate7-ngff.md [linter.model_present] Model section incomplete: 0 states, 0 transitions — both sections must exist (empty-but-present beats absent)
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

`linter.model_present` — a spec with an intent but no Model section is
incomplete by construction, same as in the generic worked example. Note
what is *missing* from this output: no pack advisory. The file uses none
of the pack vocabulary yet, so the byte-identical guarantee holds —
the pack machinery is invisible. That changes with the next section.

## Layer 2 — the `## Data` and `## Axes` registries

Two registries carry the pack's data claims. The first is
`data.lineage`'s `## Data` section (the pack consumes it rather than
redeclaring it — that is what the `## Requires` table records), with
its fixed row shape `| name | kind | dtype | units | binding |`:

```markdown
## Data

| name        | kind             | dtype   | units     | binding              |
|-------------|------------------|---------|-----------|----------------------|
| raw_frames  | data.dataset     | uint16  | intensity | file://scope/raw-p7  |
| ome_zarr    | data.artifact    | uint16  | intensity | zarr://ngff/plate7   |
| flat_field  | data.artifact    | float32 | gain      | file://calib/p7      |
| lightsheet  | data.environment | manifest| run       | equipment://lsm9     |
```

Each column, defined for a cold read:

- **name** — the row's id inside this file; the pack's `same_shape_as`
  edges point at it.
- **kind** — `data.lineage`'s closed set: `data.dataset` (a bounded
  collection of measurements), `data.artifact` (a derived object a
  pipeline produces or consumes), `data.environment` (the execution or
  measurement context). These are *fiber kinds* — pack-declared
  vocabulary, never a growth of the base closed sets.
- **dtype** — free-form descriptor of the data's element type
  (`uint16`, `float32`). The pack does not constrain its values —
  though the **`dtype:`** floor case below makes the law *state* it.
- **units** / **binding** — opaque prose for the unit of measure and
  the pointer to the external system of record (`zarr://…`,
  `file://…`). The linter never interprets either: OME/NGFF and
  BioImage.IO stay authoritative.

The second registry is the pack's own namespaced section, declared in
its `## Sections` manifest with the fixed row shape
`| name | axis | scale | unit |`:

```markdown
## Axes

| name | axis | scale | unit    |
|------|------|-------|---------|
| x    | x    | 0.65  | µm      |
| y    | y    | 0.65  | µm      |
| z    | z    | 2.00  | µm      |
| c    | c    | 1     | channel |
| t    | t    | 300   | s       |
```

- **name** — the axis row's id inside this file.
- **axis** — the dimensional label with OME-NGFF-style semantics
  (`x`, `y`, `z`, `c` for channel, `t` for time). Free prose; the pack
  does not constrain it.
- **scale** / **unit** — opaque prose recording voxel size and its unit
  (`0.65` `µm`). The format never parses these columns — that is
  exactly what keeps OME/NGFF external.

Shape claims are the pack's reference field, declared in its
`## References` manifest — `same_shape_as`, resolving to a `## Data`
row of the same file. It rides as a column on a second table under
`## Data`:

```markdown
| row       | same_shape_as |
|-----------|---------------|
| ome_zarr  | raw_frames    |
```

Read the edge aloud: the written `ome_zarr` store keeps the voxel
geometry of the acquired `raw_frames` — same x/y/z/c/t extents, per the
`## Axes` rows above.

The pack declares four named checkers in its `## Checkers` manifest —
`bioimage.dtype_is`, `bioimage.shape_eq`, and
`bioimage.units_convertible` (the data-shaped predicate grammar,
resolving against declared `## Data` rows — the last one bridging
`numeric.predicates`' opaque unit domain), and `bioimage.same_shape_closed`
(*every `same_shape_as` name resolves to a `## Data` row declared in
the same file*). It also declares a kind-dependent case-label floor in
its `## Floors` manifest: `bioimage.transform` properties must carry
**`preserves:`** and **`dtype:`** case labels. The Properties layer
below pays that floor in full.

## Layer 3 — Constraints, Model, Properties

The rest of the file is the ordinary four-layer shape — the pack adds
vocabulary, it does not change the layers. Two invariants state the
record's claims, an `effect` constraint labels the failure output, an
`observes` edge ties the observer to the effect, and the model is a
five-state run lifecycle whose failure terminal emits. The Properties
table includes one row with the pack's fiber kind, `norm_cases` —
`kind: bioimage.transform`, typeable in the base-table kind column
because the pack is active — which enumerates the pack floor's
**`preserves:`** and **`dtype:`** case labels:

```markdown
## Constraints

| id             | kind      | expr                                                                                      | traces_to        | observes                  |
|----------------|-----------|-------------------------------------------------------------------------------------------|------------------|---------------------------|
| axes_declared  | invariant | `every dataset's voxel geometry is declared in the ## Axes table of this file`              | [[plate7.ngff]]  |                           |
| shapes_paired  | invariant | `every ome_zarr names its raw dataset as its same_shape_as source`                          | [[plate7.ngff]]  |                           |
| norm_fail      | effect    | `plate7.ngff.norm_fail(detail)`                                                            | [[plate7.ngff]]  |                           |
| fail_surfaced  | invariant | `every normalization failure reaches the operator with the offending dataset named`         | [[plate7.ngff]]  | [[plate7.ngff.norm_fail]] |

## Model

### States

- `acquired`
- `corrected`
- `written`
- `published`
- `failed` (emits: `[[plate7.ngff.norm_fail]]`)

### Transitions

| id       | from     | to        | guard                                                    |
|----------|----------|-----------|-----------------------------------------------------------|
| correct  | acquired | corrected | `[[plate7.ngff.axes_declared]]`                           |
| write    | corrected| written   | `[[plate7.ngff.shapes_paired]]`                           |
| publish  | written  | published | `ome_zarr binding written`                                |
| fail     | acquired | failed    | `¬[[plate7.ngff.axes_declared]]`                          |

## Properties

| id               | kind                | derives_from                     | generator                        | predicate                                                       |
|------------------|---------------------|----------------------------------|----------------------------------|-----------------------------------------------------------------|
| axes_resolvable  | unit                | [[plate7.ngff.axes_declared]]    | `plates_with_axis_rows()`        | `every dataset's axes resolve to declared ## Axes rows`          |
| shapes_resolvable| unit                | [[plate7.ngff.shapes_paired]]    | `plates_with_paired_shapes()`    | `every same_shape_as name resolves to a declared ## Data row`    |
| fail_emitted     | unit                | [[plate7.ngff.norm_fail]]        | `plates_with_norm_failures()`    | `the failure carries the norm_fail label with detail`            |
| fail_named       | unit                | [[plate7.ngff.fail_surfaced]]    | `plates_with_norm_failures()`    | `the operator message names the offending dataset`               |
| norm_cases       | bioimage.transform  | [[plate7.ngff.axes_declared]]    | `plates_with_flatfield_steps()`  | `the flat-field step transforms image data in every declared case — **preserves:** the corrected volume keeps the acquired voxel geometry; **dtype:** the corrected volume stays in the acquired dtype, uint16` |
```

Nothing pack-specific in the layers themselves — `traces_to`, guards,
`emits`, coverage: all core-format machinery, taught properly in the
[generic worked example](worked-example.md). Note what the `norm_cases`
row does *not* carry: the base `law` floor's **`identity:`** and
**`associativity:`** cases are not demanded here, because the floor is
kind-dependent — the base law floor stays untouched, and a kind with no
declared floor inherits none.

## First full lint — a dangling reference, then the fix

The author's first full draft carries one typo: the `axes_declared`
constraint traces to `[[plate7.nff]]` — plural, misremembered, the same
classic as the reservation example. Lint the file:

```sh
spk lint plate7-ngff.md
```

```text
lint: 1 file(s) linted, 1 finding(s), 2 advisory warning(s)
  plate7-ngff.md [linter.total_refs] dangling reference `[[plate7.nff]]` from constraints.traces_to — target not defined in any spec file
  advisory plate7.ngff [linter.observability] pack `bioimage.data` activated for `plate7.ngff` (vocabulary match: bioimage.transform, Axes, same_shape_as) — empty checked-set (no violations to report)
  advisory plate7.ngff [linter.observability] pack `data.lineage` activated for `plate7.ngff` (vocabulary match: data.dataset, data.artifact, data.environment, Data) — empty checked-set (no violations to report)
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

Exit 1 — a finding, never a silent pass. Three things to read here:

1. **The finding and the advisories ride different channels.** Both
   packs activated (their vocabulary is all over the file — the match
   list names every token that triggered each), but activation lives on
   the *warnings* channel and never gates. The dangling reference lives
   on the *findings* channel and does.
2. **Two advisories, two packs.** `bioimage.data` activated on its own
   vocabulary (`bioimage.transform`, `Axes`, `same_shape_as`);
   `data.lineage` activated on the `## Data` rows the consumer pack
   reuses. The two packs the file uses *no* vocabulary of —
   `numeric.predicates`, `empirical.registry` — stay silent: activation
   is per-vocabulary-use, not per-`## Requires` row.
3. **`empty checked-set (no violations to report)`** — the packs'
   declared checkers ran and honestly report an empty violation set.
   More on what that does and does not mean below.

Fix the typo — `[[plate7.nff]]` → `[[plate7.ngff]]` — and lint again:

```text
lint: 1 file(s) linted, 0 finding(s), 2 advisory warning(s)
  advisory plate7.ngff [linter.observability] pack `bioimage.data` activated for `plate7.ngff` (vocabulary match: bioimage.transform, Axes, same_shape_as) — empty checked-set (no violations to report)
  advisory plate7.ngff [linter.observability] pack `data.lineage` activated for `plate7.ngff` (vocabulary match: data.dataset, data.artifact, data.environment, Data) — empty checked-set (no violations to report)
→ Run: specodelic graph
```

Zero findings, exit 0 — and now the advisories are the *whole* output.
Read them as the meter they are: an exit-0 lint alone proves nothing (a
spec using none of the pack vocabulary also exits 0, silently); **the
activation line naming `bioimage.data` is the observable** that the
pack was discovered, matched, and engaged. The pipeline's next stage
runs pack-aware too:

```sh
spk graph plate7-ngff.md
```

```text
graph: 1 file(s), 19 node(s), 22 edge(s), 0 dangling, 0 typing violation(s), 0 supersedes cycle(s), 0 external boundary(ies)
→ Run: specodelic lint
```

## What a missing pack looks like

The demo depends on the pack file being *discovered*. Copy the finished
spec into a directory with no `kind: profile` file in it — the
machinery goes inert, and now that inertness bites, because
`norm_cases` uses the pack's fiber kind in a base-table `kind` column:

```text
lint: 1 file(s) linted, 2 finding(s), 0 advisory warning(s)
  plate7.ngff [linter.property_kind_closed] property `norm_cases` has kind `bioimage.transform` — outside the closed set {unit, law} (kinds.md property_row_shape)
  plate7.ngff [linter.orphan_vocabulary] orphan vocabulary: pack-qualified token(s) `bioimage.transform` used by `plate7.ngff`, but no `kind: profile` pack in namespace `bioimage` is discovered — candidate pack `bioimage.*` (prefix-derived: only the namespace is known); remediations: add/enable a `kind: profile` pack file declaring the vocabulary, or fix the vocabulary (un-qualify or retype the token)
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

`linter.orphan_vocabulary` — the finding always names the candidate
(namespace-derived, `bioimage.*` here) and states **both**
remediations: enable a pack declaring the vocabulary, or fix the token.
`linter.property_kind_closed` fires alongside it: with no pack active,
the walkers accept only the base kinds in the `kind` column (base ∪
active-pack-fiber, never narrower). Copy the pack files back into the
workspace and both findings clear — the same file, byte-identical,
lints clean. The reliable diagnostic is the one this section started
with: a spec you *meant* to be pack-enabled but lint with no activation
advisory is a spec whose pack was not discovered.

## What the linter does not catch (yet)

Honesty check — the failures this page has *not* shown are the two you
might expect first, because the pack declares a checker and a floor for
each.

**A `same_shape_as` edge that dangles.** Change `raw_frames` to
`ghost_row` in the `same_shape_as` column and lint the pack-enabled
workspace:

```text
lint: 1 file(s) linted, 0 finding(s), 2 advisory warning(s)
  advisory plate7.ngff [linter.observability] pack `bioimage.data` activated for `plate7.ngff` (vocabulary match: bioimage.transform, Axes, same_shape_as) — empty checked-set (no violations to report)
  advisory plate7.ngff [linter.observability] pack `data.lineage` activated for `plate7.ngff` (vocabulary match: data.dataset, data.artifact, data.environment, Data) — empty checked-set (no violations to report)
→ Run: specodelic graph
```

Byte-for-byte the clean output — exit 0, activation advisories, nothing
else. That is not the linter missing the dangle by accident:
`bioimage.same_shape_closed` ships as a **declaration**.

**Floor labels dropped.** Strip the **`preserves:`** and **`dtype:`**
case labels out of `norm_cases`'s predicate and lint:

```text
lint: 1 file(s) linted, 0 finding(s), 2 advisory warning(s)
  advisory plate7.ngff [linter.observability] pack `bioimage.data` activated for `plate7.ngff` (vocabulary match: bioimage.transform, Axes, same_shape_as) — empty checked-set (no violations to report)
  advisory plate7.ngff [linter.observability] pack `data.lineage` activated for `plate7.ngff` (vocabulary match: data.dataset, data.artifact, data.environment, Data) — empty checked-set (no violations to report)
→ Run: specodelic graph
```

Identical again. The pack's floor is the same kind of declaration: the
manifest *requires* the labels, but in v1 pack checkers are
honest-empty — activation and the empty checked-set are reported
truthfully, violation *evaluation* is not implemented yet (the
[guide](../using-packs.md) documents this, and
[verification boundaries](../verification-boundaries.md) places it on
the guarantee ladder). The format's answer to the gap is the one this
spec already embodies: each unenforced claim lives where the evidence
plan lives — the dangling-edge claim in the `shapes_paired` invariant
and its `shapes_resolvable` property, the floor in the `norm_cases`
predicate's enumerated cases — each waiting on a real executable
predicate. When the checkers gain evaluation, the declarations are
already in place; the spec does not change.

## The finished spec

Every block above assembles into this one file, `plate7-ngff.md` —
exactly as it lints at 0 findings, 2 activation advisories:

```markdown
---
id: plate7.ngff
kind: intent
statement: "WHEN the imaging pipeline processes a light-sheet plate THE system SHALL record the image axes of every dataset it writes together with the transform laws each pipeline step asserts about how it reshapes image data."
---

# Plate 7 NGFF conversion

A small bioimage record for one light-sheet run: the pipeline acquires
raw microscopy frames, applies flat-field correction, writes an
OME/NGFF zarr store, and every transform step states the law it obeys —
what it preserves and in which dtype.

## Data

| name        | kind             | dtype   | units     | binding              |
|-------------|------------------|---------|-----------|----------------------|
| raw_frames  | data.dataset     | uint16  | intensity | file://scope/raw-p7  |
| ome_zarr    | data.artifact    | uint16  | intensity | zarr://ngff/plate7   |
| flat_field  | data.artifact    | float32 | gain      | file://calib/p7      |
| lightsheet  | data.environment | manifest| run       | equipment://lsm9     |

| row       | same_shape_as |
|-----------|---------------|
| ome_zarr  | raw_frames    |

## Axes

| name | axis | scale | unit    |
|------|------|-------|---------|
| x    | x    | 0.65  | µm      |
| y    | y    | 0.65  | µm      |
| z    | z    | 2.00  | µm      |
| c    | c    | 1     | channel |
| t    | t    | 300   | s       |

## Constraints

| id             | kind      | expr                                                                                      | traces_to        | observes                  |
|----------------|-----------|-------------------------------------------------------------------------------------------|------------------|---------------------------|
| axes_declared  | invariant | `every dataset's voxel geometry is declared in the ## Axes table of this file`              | [[plate7.ngff]]  |                           |
| shapes_paired  | invariant | `every ome_zarr names its raw dataset as its same_shape_as source`                          | [[plate7.ngff]]  |                           |
| norm_fail      | effect    | `plate7.ngff.norm_fail(detail)`                                                            | [[plate7.ngff]]  |                           |
| fail_surfaced  | invariant | `every normalization failure reaches the operator with the offending dataset named`         | [[plate7.ngff]]  | [[plate7.ngff.norm_fail]] |

## Model

### States

- `acquired`
- `corrected`
- `written`
- `published`
- `failed` (emits: `[[plate7.ngff.norm_fail]]`)

### Transitions

| id       | from     | to        | guard                                                    |
|----------|----------|-----------|-----------------------------------------------------------|
| correct  | acquired | corrected | `[[plate7.ngff.axes_declared]]`                           |
| write    | corrected| written   | `[[plate7.ngff.shapes_paired]]`                           |
| publish  | written  | published | `ome_zarr binding written`                                |
| fail     | acquired | failed    | `¬[[plate7.ngff.axes_declared]]`                          |

## Properties

| id               | kind                | derives_from                     | generator                        | predicate                                                       |
|------------------|---------------------|----------------------------------|----------------------------------|-----------------------------------------------------------------|
| axes_resolvable  | unit                | [[plate7.ngff.axes_declared]]    | `plates_with_axis_rows()`        | `every dataset's axes resolve to declared ## Axes rows`          |
| shapes_resolvable| unit                | [[plate7.ngff.shapes_paired]]    | `plates_with_paired_shapes()`    | `every same_shape_as name resolves to a declared ## Data row`    |
| fail_emitted     | unit                | [[plate7.ngff.norm_fail]]        | `plates_with_norm_failures()`    | `the failure carries the norm_fail label with detail`            |
| fail_named       | unit                | [[plate7.ngff.fail_surfaced]]    | `plates_with_norm_failures()`    | `the operator message names the offending dataset`               |
| norm_cases       | bioimage.transform  | [[plate7.ngff.axes_declared]]    | `plates_with_flatfield_steps()`  | `the flat-field step transforms image data in every declared case — **preserves:** the corrected volume keeps the acquired voxel geometry; **dtype:** the corrected volume stays in the acquired dtype, uint16` |
```

To reproduce every block on this page: copy
`packs/bioimage-data.md` and the three pack files it requires
(`packs/data-lineage.md`, `packs/numeric-predicates.md`,
`packs/empirical-registry.md`) from the repo into a fresh git
workspace, save the file above as `plate7-ngff.md`, and run
`spk lint plate7-ngff.md`.

## Where to go next

- The mechanism, end to end: [using packs — the practical
  guide](../using-packs.md) — discovery, activation, honest-empty
  semantics, the full CLI loop.
- The pack's normative home: [packs — domain-pack
  mechanism](../specs/packs/spec.md), and the pack definition itself —
  [`packs/bioimage-data.md`](https://github.com/charly-vibes/specodelic/tree/main/packs/bioimage-data.md)
  is the source of truth for the vocabulary this page used.
- The capability spec behind the pack:
  [bioimage-data-pack](../specs/bioimage-data-pack/spec.md).
- The same journey for the packs this one consumes:
  [the data.lineage pack](data-lineage.md),
  [the numeric.predicates pack](numeric-predicates.md), and
  [the empirical.registry pack](empirical-registry.md) — or without
  packs at all: [worked example —
  reservation/order](worked-example.md).
- Rule semantics offline: `spk explain packs` and
  `spk explain lint-rules`.
