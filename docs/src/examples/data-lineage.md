# Worked example — the data.lineage pack, failures included

The [generic worked example](worked-example.md) builds a spec with the
core format alone. This page does the same job for a **domain pack**:
[`data.lineage`](https://github.com/charly-vibes/specodelic/tree/main/packs/data-lineage.md),
the pack that adds a typed `## Data` section, dataset/artifact/environment
kinds, and lineage reference fields — without a format revision and
without growing any base closed set. You will see a small spec use the
pack's vocabulary end to end, the pack-activation advisory that proves
the pack actually engaged, two genuinely failing variants with their real
labeled findings, and — just as important — the failures the linter does
*not* catch yet, shown honestly.

How packs are enabled and what a pack adds mechanically is the job of
[using packs — the practical guide](../using-packs.md); read that for the
mechanism, this page for the hands-on loop. Every lint and graph block
below is real output from `specodelic 0.10.0` on the file as it stood at
that moment. On a TTY `spk` prints the human-readable text shown here;
piped, it emits a JSON envelope instead.

## The story

A sequencing lab runs an ETL pipeline: raw reads land in object storage,
a count table and a QC report are derived from them, and every dataset
names the environment (instrument, manifest) it was produced in. The
spec's job is to record that lineage — one capability, one file, named
`lab-etl.md`; the format's naming law maps `-` to the namespace dot, so
the frontmatter id is `lab.etl`.

Deliberately out of scope: *where* the data physically lives and *which*
external standard governs each artifact. That is the binding column's
job, and it stays opaque — the pack bridges external data standards
(OME/NGFF, Parquet, …), it never absorbs them.

## The setup

Packs are discovered by scanning the workspace, not by any config file:
every `kind: profile` file in the git repository toplevel of the linted
spec is a candidate pack. So the demo workspace needs two files — the
pack definition (copied from this repo's `packs/` directory) and the
spec under test:

```sh
mkdir demo && cd demo && git init
cp /path/to/specodelic/packs/data-lineage.md .
```

That is the whole enablement story. The discovery and activation rules
(corpus-scan anchors, word-boundary vocabulary matching, the
byte-identical guarantee) are in [the guide](../using-packs.md); this
page just runs the loop.

## Layer 1 — Intent

```markdown
---
id: lab.etl
kind: intent
statement: "WHEN the lab's ETL pipeline runs THE system SHALL record every dataset, artifact, and environment it touches together with the lineage edges between them."
---

# Lab ETL lineage

A small data-lineage record for one sequencing ETL run: the pipeline
reads raw reads, produces a count table and a QC report, and every
dataset names the environment it ran in.
```

Lint it as it stands:

```sh
spk lint lab-etl.md
```

```text
lint: 1 file(s) linted, 1 finding(s), 0 advisory warning(s)
  lab-etl.md [linter.model_present] Model section incomplete: 0 states, 0 transitions — both sections must exist (empty-but-present beats absent)
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

`linter.model_present` — a spec with an intent but no Model section is
incomplete by construction, same as in the generic worked example. Note
what is *missing* from this output: no pack advisory. The file uses none
of the pack's vocabulary yet, so the byte-identical guarantee holds —
the pack machinery is invisible. That changes with the next section.

## Layer 2 — the `## Data` section

This is the pack's namespaced section, declared in the pack's
`## Sections` manifest with the fixed row shape
`| name | kind | dtype | units | binding |`:

```markdown
## Data

| name      | kind             | dtype    | units  | binding              |
|-----------|------------------|----------|--------|----------------------|
| raw_reads | data.dataset     | fastq    | reads  | s3://lab/raw-reads   |
| counts    | data.artifact    | tsv      | count  | parquet://lab/counts |
| qc_report | data.artifact    | html     | report | file://lab/qc.html   |
| sequencer | data.environment | manifest | run    | equipment://seq/7    |
```

Each column, defined for a cold read:

- **name** — the row's id inside this file; lineage edges point at it.
- **kind** — the pack's own closed set: `data.dataset` (a bounded
  collection of measurements), `data.artifact` (a derived object a
  pipeline produces or consumes), `data.environment` (the execution or
  measurement context of a dataset or artifact). These are *fiber
  kinds* — pack-declared vocabulary, never a growth of the base closed
  sets.
- **dtype** / **units** — free-form descriptors of the data's shape and
  unit of measure. The pack does not constrain their values.
- **binding** — the opaque pointer to the external system of record
  (`s3://…`, `parquet://…`). The pack declares no syntax for it and the
  linter never interprets it: external data standards stay authoritative.

Lineage edges are the pack's reference fields, declared in its
`## References` manifest — `produced_by` and `consumed_by`, each
resolving to a `## Data` row of the same file. They ride as columns on a
second table under the same section:

```markdown
| row       | produced_by | consumed_by |
|-----------|-------------|-------------|
| raw_reads |             | counts      |
| counts    | raw_reads   | qc_report   |
```

Read the edges aloud: `raw_reads` is consumed by `counts`; `counts` is
produced by `raw_reads` and consumed by `qc_report`. The pack also
declares a checker — `data.lineage.closure`, *every lineage reference
name resolves to a `## Data` row declared in the same file* — and a
case-label floor: law-kind properties about `data.artifact` rows must
carry a `provenance:` case. (This spec's properties are all `unit` kind,
so the floor does not bite; see [the guide](../using-packs.md) for what
floors demand.)

## Layer 3 — Constraints, Model, Properties

The rest of the file is the ordinary four-layer shape — the pack adds
vocabulary, it does not change the layers. Two invariants state the
lineage claims, an `effect` constraint labels the failure output, an
`observes` edge ties the observer to the effect, and the model is a
five-state run lifecycle whose failure terminal emits:

```markdown
## Constraints

| id             | kind      | expr                                                                             | traces_to   | observes               |
|----------------|-----------|-----------------------------------------------------------------------------------|-------------|------------------------|
| lineage_closed | invariant | `every produced_by or consumed_by edge names a ## Data row declared in this file` | [[lab.etl]] |                        |
| env_binds_run  | invariant | `every dataset names exactly one environment as its execution context`            | [[lab.etl]] |                        |
| lineage_gap    | effect    | `lab.etl.lineage_gap(detail)`                                                     | [[lab.etl]] |                        |
| gap_surfaced   | invariant | `every lineage gap reaches the operator with the offending edge named`            | [[lab.etl]] | [[lab.etl.lineage_gap]] |

## Model

### States

- `started`
- `ingested`
- `counted`
- `reported`
- `failed` (emits: `[[lab.etl.lineage_gap]]`)

### Transitions

| id     | from     | to       | guard                                                    |
|--------|----------|----------|-----------------------------------------------------------|
| begin  | started  | ingested | `sequencer manifest present ∧ [[lab.etl.env_binds_run]]`  |
| count  | ingested | counted  | `[[lab.etl.lineage_closed]]`                              |
| report | counted  | reported | `qc_report binding written`                               |
| fail   | started  | failed   | `¬[[lab.etl.env_binds_run]]`                              |

## Properties

| id            | kind | derives_from               | generator                    | predicate                                               |
|---------------|------|-----------------------------|------------------------------|---------------------------------------------------------|
| edges_resolve | unit | [[lab.etl.lineage_closed]] | `runs_with_lineage_edges()`  | `every lineage edge resolves to a declared ## Data row`  |
| env_recorded  | unit | [[lab.etl.env_binds_run]]  | `runs_with_datasets()`       | `every dataset carries its environment binding`          |
| gap_emitted   | unit | [[lab.etl.lineage_gap]]    | `runs_with_undefined_rows()` | `the failure carries the lineage_gap label with detail`  |
| gap_named     | unit | [[lab.etl.gap_surfaced]]   | `runs_with_undefined_rows()` | `the operator message names the offending lineage edge`  |
```

Nothing pack-specific here — `traces_to`, guards, `emits`, coverage:
all core-format machinery, taught properly in the
[generic worked example](worked-example.md).

## First full lint — a dangling reference, then the fix

The author's first full draft carries one typo: the `lineage_closed`
constraint traces to `[[lab.ets]]` — plural, misremembered, the same
classic as the reservation example. Lint the file:

```sh
spk lint lab-etl.md
```

```text
lint: 1 file(s) linted, 1 finding(s), 1 advisory warning(s)
  lab-etl.md [linter.total_refs] dangling reference `[[lab.ets]]` from constraints.traces_to — target not defined in any spec file
  advisory lab.etl [linter.observability] pack `data.lineage` activated for `lab.etl` (vocabulary match: data.dataset, data.artifact, data.environment, Data, produced_by, consumed_by) — empty checked-set (no violations to report)
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

Exit 1 — a finding, never a silent pass. Two things to read here:

1. **The finding and the advisory ride different channels.** The pack
   activated (its vocabulary is all over the file — note the match list
   names every token that triggered it), but activation lives on the
   *warnings* channel and never gates. The dangling reference lives on
   the *findings* channel and does.
2. **`empty checked-set (no violations to report)`** — the pack's
   declared `data.lineage.closure` checker ran and honestly reports an
   empty violation set. More on what that does and does not mean below.

Fix the typo — `[[lab.ets]]` → `[[lab.etl]]` — and lint again:

```text
lint: 1 file(s) linted, 0 finding(s), 1 advisory warning(s)
  advisory lab.etl [linter.observability] pack `data.lineage` activated for `lab.etl` (vocabulary match: data.dataset, data.artifact, data.environment, Data, produced_by, consumed_by) — empty checked-set (no violations to report)
→ Run: specodelic graph
```

Zero findings, exit 0 — and now the advisory is the *whole* output.
Read it as the meter it is: an exit-0 lint alone proves nothing (a spec
using none of the pack vocabulary also exits 0, silently); **the
activation line naming `data.lineage` is the observable** that the pack
was discovered, matched, and engaged. The pipeline's next stage runs
pack-aware too:

```sh
spk graph lab-etl.md
```

```text
graph: 1 file(s), 18 node(s), 21 edge(s), 0 dangling, 0 typing violation(s), 0 supersedes cycle(s), 0 external boundary(ies)
→ Run: specodelic lint
```

## What a missing pack looks like

The demo depends on the pack file being *discovered*. Delete the setup
step — lint the same finished spec in a directory with no
`kind: profile` file in it — and the machinery goes inert:

```text
lint: 1 file(s) linted, 0 finding(s), 0 advisory warning(s)
→ Run: specodelic graph
```

No advisory, no findings — the byte-identical guarantee in action. The
`## Data` table is now just markdown the linter does not interpret, and
that is by design: a pack can never gate a file that did not opt in.
But inert cuts both ways, and the linter tells you when vocabulary
*expects* a pack. Say the author, pack-enthused, writes a base-table
`kind` cell as `data.dataset` instead of `invariant`, in that same
pack-less directory:

```text
lint: 1 file(s) linted, 2 finding(s), 0 advisory warning(s)
  lab.etl [linter.constraint_kind_closed] constraint `lineage_closed` has kind `data.dataset` — outside the closed set {invariant, advisory, effect, extension_point} (kinds.md constraint_row_shape)
  lab.etl [linter.orphan_vocabulary] orphan vocabulary: pack-qualified token(s) `data.dataset` used by `lab.etl`, but no `kind: profile` pack in namespace `data` is discovered — candidate pack `data.*` (prefix-derived: only the namespace is known); remediations: add/enable a `kind: profile` pack file declaring the vocabulary, or fix the vocabulary (un-qualify or retype the token)
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

`linter.orphan_vocabulary` — the finding always names the candidate
(namespace-derived, `data.*` here) and states **both** remediations:
enable a pack declaring the vocabulary, or fix the token. Copy the pack
file back into the workspace and the orphan finding clears — with the
pack active, the walkers accept a fiber kind in a kind column
(base ∪ active-pack-fiber), so that particular mis-typing only surfaces
when the pack is missing. The reliable diagnostic is the one this
section started with: a spec you *meant* to be pack-enabled but lint
with no activation advisory is a spec whose pack was not discovered.

## What the linter does not catch (yet)

Honesty check — the failure this page has *not* shown is the one you
might expect first: a lineage edge that dangles. Change `qc_report` to
`ghost_row` in the `consumed_by` column and lint:

```text
lint: 1 file(s) linted, 0 finding(s), 1 advisory warning(s)
  advisory lab.etl [linter.observability] pack `data.lineage` activated for `lab.etl` (vocabulary match: data.dataset, data.artifact, data.environment, Data, produced_by, consumed_by) — empty checked-set (no violations to report)
→ Run: specodelic graph
```

Byte-for-byte the clean output — exit 0, activation advisory, nothing
else. That is not the linter missing the dangle by accident: `data.lineage.closure`
ships as a **declaration**. In v1, pack checkers are honest-empty —
activation and the empty checked-set are reported truthfully, violation
*evaluation* is not implemented yet (the
[guide](../using-packs.md) documents this, and
[verification boundaries](../verification-boundaries.md) places it on
the guarantee ladder). The format's answer to the gap is the one this
spec already embodies: the dangling-edge claim lives where the evidence
plan lives — the `lineage_closed` invariant, its `edges_resolve`
property, and eventually a real executable predicate. When the checker
gains evaluation, the declaration is already in place; the spec does not
change.

## The finished spec

Every block above assembles into this one file, `lab-etl.md` — exactly
as it lints at 0 findings, 1 activation advisory:

```markdown
---
id: lab.etl
kind: intent
statement: "WHEN the lab's ETL pipeline runs THE system SHALL record every dataset, artifact, and environment it touches together with the lineage edges between them."
---

# Lab ETL lineage

A small data-lineage record for one sequencing ETL run: the pipeline
reads raw reads, produces a count table and a QC report, and every
dataset names the environment it ran in.

## Constraints

| id             | kind      | expr                                                                             | traces_to   | observes               |
|----------------|-----------|-----------------------------------------------------------------------------------|-------------|------------------------|
| lineage_closed | invariant | `every produced_by or consumed_by edge names a ## Data row declared in this file` | [[lab.etl]] |                        |
| env_binds_run  | invariant | `every dataset names exactly one environment as its execution context`            | [[lab.etl]] |                        |
| lineage_gap    | effect    | `lab.etl.lineage_gap(detail)`                                                     | [[lab.etl]] |                        |
| gap_surfaced   | invariant | `every lineage gap reaches the operator with the offending edge named`            | [[lab.etl]] | [[lab.etl.lineage_gap]] |

## Data

| name      | kind             | dtype    | units  | binding              |
|-----------|------------------|----------|--------|----------------------|
| raw_reads | data.dataset     | fastq    | reads  | s3://lab/raw-reads   |
| counts    | data.artifact    | tsv      | count  | parquet://lab/counts |
| qc_report | data.artifact    | html     | report | file://lab/qc.html   |
| sequencer | data.environment | manifest | run    | equipment://seq/7    |

| row       | produced_by | consumed_by |
|-----------|-------------|-------------|
| raw_reads |             | counts      |
| counts    | raw_reads   | qc_report   |

## Model

### States

- `started`
- `ingested`
- `counted`
- `reported`
- `failed` (emits: `[[lab.etl.lineage_gap]]`)

### Transitions

| id     | from     | to       | guard                                                    |
|--------|----------|----------|-----------------------------------------------------------|
| begin  | started  | ingested | `sequencer manifest present ∧ [[lab.etl.env_binds_run]]`  |
| count  | ingested | counted  | `[[lab.etl.lineage_closed]]`                              |
| report | counted  | reported | `qc_report binding written`                               |
| fail   | started  | failed   | `¬[[lab.etl.env_binds_run]]`                              |

## Properties

| id            | kind | derives_from               | generator                    | predicate                                               |
|---------------|------|-----------------------------|------------------------------|---------------------------------------------------------|
| edges_resolve | unit | [[lab.etl.lineage_closed]] | `runs_with_lineage_edges()`  | `every lineage edge resolves to a declared ## Data row`  |
| env_recorded  | unit | [[lab.etl.env_binds_run]]  | `runs_with_datasets()`       | `every dataset carries its environment binding`          |
| gap_emitted   | unit | [[lab.etl.lineage_gap]]    | `runs_with_undefined_rows()` | `the failure carries the lineage_gap label with detail`  |
| gap_named     | unit | [[lab.etl.gap_surfaced]]   | `runs_with_undefined_rows()` | `the operator message names the offending lineage edge`  |
```

To reproduce every block on this page: copy
`packs/data-lineage.md` from the repo into a fresh git workspace, save
the file above as `lab-etl.md`, and run `spk lint lab-etl.md`.

## Where to go next

- The mechanism, end to end: [using packs — the practical
  guide](../using-packs.md) — discovery, activation, honest-empty
  semantics, the full CLI loop.
- The pack's normative home: [packs — domain-pack
  mechanism](../specs/packs/spec.md), and the pack definition itself —
  [`packs/data-lineage.md`](https://github.com/charly-vibes/specodelic/tree/main/packs/data-lineage.md)
  is the source of truth for the vocabulary this page used.
- The capability spec behind the pack:
  [data-lineage-pack](../specs/data-lineage-pack/spec.md).
- The same journey without packs: [worked example —
  reservation/order](worked-example.md).
- Rule semantics offline: `spk explain packs` and
  `spk explain lint-rules`.
