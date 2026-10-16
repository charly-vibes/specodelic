# Using packs

A domain pack is specodelic's extension mechanism: a way to give a
domain — data lineage, numeric predicates, bioimage metadata — its own
typed sections, kinds, and reference fields **without a format
revision** and without touching the core's closed kind sets. This page
is the practical guide: what a pack is, how a workspace enables one,
what it adds, and the CLI loop you run against a pack-enabled spec.
The machine truth lives in the [packs spec](specs/packs/spec.md); this
page explains how to *use* the mechanism, not how it is specified.

Everything shown below is real output from `specodelic 0.11.0` (on a
TTY `spk` prints human-readable text; piped, it emits a JSON envelope).

## What a pack is

A pack is a **four-layer spec file with `kind: profile` in
frontmatter** — the same Intent → Constraints → Model → Properties
shape as every other spec, plus six per-facet manifest tables:

> `## Sections`, `## Kinds`, `## References`, `## Checkers`,
> `## Floors`, `## Requires`

There is no separate artifact format and no sidecar manifest: the pack
file is self-hosting, and `linter.pack_shape` checks the six tables'
row shapes at lint time in every lifecycle state. A malformed manifest
is a labeled failure, never a silent pass.

In this repo the pack definitions live in
[`packs/*.md`](https://github.com/charly-vibes/specodelic/tree/main/packs)
(`packs/data-lineage.md`, `packs/numeric-predicates.md`, …). The
frontmatter there is the **source of truth for the pack inventory** —
CI derives the pack set from it
(`scripts/check_pack_doc_counts.py`, the `specodelic-typ0` gate) so
doc pages that talk about "the standard packs" cannot drift from what
actually ships. The in-repo packs are the five standard packs
(`data.lineage`, `numeric.predicates`, `empirical.registry`,
`quant.finance`, and `svc.contracts` — the last still `draft`) plus
the `bioimage.data` D6 pilot (the first cross-pack `## Requires`
user). Each pack also has a capability spec in the rendered
corpus — e.g.
[data-lineage-pack](specs/data-lineage-pack/spec.md).

## How a pack is enabled: corpus scan, not a config file

This is the part people coming from other tools trip on, so it is
stated plainly: **there is no `packs.toml`, no config file, no
registry outside the corpus.** Enablement is corpus-scan discovery:

- Every `kind: profile` file in the workspace is a candidate pack,
  found by scanning alone. Put a `kind: profile` file anywhere in the
  repo and it is a pack.
- The scan is **anchored at the git repository toplevel of each linted
  directory** (the lint target directory itself when no git root
  exists). Every linted path contributes its anchor, so discovery is
  order-independent: `spk lint spec/a.md spec/b.md` and
  `spk lint spec/b.md spec/a.md` discover the same pack set. Linting a
  subdirectory still discovers packs living anywhere in the workspace,
  because the anchor is the toplevel, not the subdirectory.
- The pack's frontmatter `id` (e.g. `data.lineage`) is how everything
  else refers to it.

Activation is **advisory-first and opt-in per file**: a spec activates
a discovered pack either by *using its vocabulary* (a word-boundary
match; matching is longest-prefix) or by *declaring a `uses`
reference* to the pack's id on a Constraint row. A pack file is never
a consumer of its own vocabulary — its manifest rows never trigger
its checkers. Identical vocabulary declared by two discovered packs
activates both, with findings attributed per pack.

## What a pack adds

Once a pack is active for a file, the mechanism adds six things. All
of them are **namespaced**: pack vocabulary is pack-qualified
(`data.dataset`, not `dataset`), so two packs may independently claim
a same-named concept and never collide — collision resolves by
pack-qualified naming, not by error.

1. **Namespaced sections.** The pack's `## Sections` table declares
   typed extra sections with a fixed row shape. `data.lineage`
   declares a `## Data` section whose rows are
   `| name | kind | dtype | units | binding |`.
2. **Fiber kinds.** Pack-declared kinds usable in the closed `kind`
   columns of base tables. The base closed sets never grow — the
   walkers accept a fiber kind only when its declaring pack is active
   for *that file* (base ∪ active-pack-fiber, never narrower); the
   same token without its pack is the labeled `orphan_vocabulary`
   finding below.
3. **Reference fields.** Set-valued typed references such as
   `produced_by` / `consumed_by`, resolving to declared rows of the
   same file (see the pack's `## References` table).
4. **Pack-qualified checkers.** Rules named inside the pack's
   namespace, e.g. `data.lineage.closure`. Checkers ship as
   declarations — the mechanism interprets the declaration at
   activation and reports honestly.
5. **Kind-dependent case-label floors.** A pack's `## Floors` table
   can require named cases for law-kind properties of specific fiber
   kinds (e.g. `data.artifact` laws must carry a `provenance:` case).
6. **A `## Requires` pin.** Each pack pins the base format revision it
   was written against; skew with the workspace corpus is advisory
   (below), never a failure.

Two hard guarantees round this off:

- **Byte-identical guarantee.** A spec that uses none of the pack
  vocabulary lints byte-identically to pre-mechanism behavior — no
  pack advisory, no new findings, nothing. With zero `kind: profile`
  files discovered, the machinery is inert.
- **Append-only packs.** A pack release may only *add* vocabulary;
  removing or narrowing a previously declared entry fails
  `linter.pack_shape`, naming the entry and the append-only law. The
  same law keeps base closed sets frozen — a pack can never narrow or
  retype an existing entry.

### Lifecycle and honest-empty checkers

A pack is `draft`, `published`, or `deprecated` (declared through the
pack's own Model section). `draft` packs activate advisory-first with
the draft status named in the finding; deprecated packs keep checking
with the deprecation named. A checker with no workspace fixtures,
data, or vocabulary instances reports an **empty checked-set** —
honestly, with no fabricated discoveries. That is exactly what the
activation advisories below show: `empty checked-set (no violations
to report)`.

## The CLI loop

Set up a scratch workspace with a pack file and a consumer spec (here:
`data.lineage` from the repo's `packs/` directory, and a lab-ETL
spec):

```sh
mkdir demo && cd demo && git init
cp /path/to/specodelic/packs/data-lineage.md .
```

### `spk lint` — discovery and activation

A consumer spec that declares a typed `## Data` table using the pack's
vocabulary:

```markdown
---
id: lab.etl
kind: intent
statement: "WHEN the lab runs its ETL pipeline THE system SHALL record every dataset the pipeline produces or consumes."
---

## Data

| name      | kind          | dtype | units | binding          |
|-----------|---------------|-------|-------|------------------|
| raw_reads | data.dataset  | fastq | reads | s3://lab/reads   |
| counts    | data.artifact | tsv   | count | parquet://counts |

| row       | produced_by | consumed_by |
|-----------|-------------|-------------|
| raw_reads |             | counts      |
| counts    | raw_reads   | qc_report   |
```

Lint it (the rest of the four-layer file is elided here):

```sh
spk lint lab-etl.md
```

```text
lint: 1 file(s) linted, 0 finding(s), 1 advisory warning(s)
  advisory lab.etl [linter.observability] pack `data.lineage` activated for `lab.etl` (vocabulary match: data.dataset, data.artifact, Data, produced_by, consumed_by) — empty checked-set (no violations to report)
→ Run: specodelic graph
```

exit 0 — the pack was discovered by corpus scan (the git toplevel is
the anchor) and activated by vocabulary match. The advisory is the
activation note on the warnings channel: the pack's declared rules
ran and found nothing to report. No uses edge was needed.

The explicit route is a **declared `uses` edge** — a `uses` column on
a Constraint row pointing at the pack's frontmatter id:

```markdown
| id | kind      | expr     | traces_to     | uses            |
|----|-----------|----------|---------------|-----------------|
| c1 | invariant | `plain`  | [[lab.etl]]   | [[data.lineage]] |
```

```text
lint: 1 file(s) linted, 0 finding(s), 2 advisory warning(s)
  advisory lab.etl [linter.skew_advisory] pack `data.lineage` pins base Revision 14, workspace corpus is at Revision 18 — revision skew is advisory (never failing); consider updating the pack's `## Requires` base pin
  advisory lab.etl [linter.observability] pack `data.lineage` activated for `lab.etl` (declared uses edge) — declared rules: 6; empty checked-set (no violations to report)
→ Run: specodelic graph
```

The `skew_advisory` fires because this copy of the pack pins an older
base revision than the corpus — a labeled advisory on the warnings
channel, never a failure.

### `spk lint` — what failure looks like

A **declared `uses` edge naming a pack that is not discovered** — say
you lint the consumer file in a directory with no pack file in it —
fails with labeled findings, never a silent pass (exit 1):

```text
lint: 1 file(s) linted, 2 finding(s), 0 advisory warning(s)
  lab-etl.md [linter.total_refs] dangling reference `[[data.lineage]]` from constraints.uses — target not defined in any spec file
  lab.etl [linter.orphan_vocabulary] orphan vocabulary: `uses` edge targets `data.lineage`, but no `kind: profile` pack with that id is discovered — candidate pack `data.lineage`; remediations: add/enable a `kind: profile` pack file declaring it, or fix the vocabulary (remove or retype the `uses` edge)
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

The `orphan_vocabulary` finding always names the candidate
pack — prefix-derived from the vocabulary token's namespace when only
the namespace is known (a `data.*` token with no `data.lineage`-like
pack discovered names the `data.*` namespace) — and states **both**
remediations: enable a pack declaring the vocabulary, or fix the
vocabulary. Pack-qualified tokens in a base table's `kind` column
fire the same rule when their namespace has no discovered pack; the
pack-added section's own rows stay inert without the pack. A
malformed pack manifest (wrong row shapes, a missing facet table, a
non-pack-qualified kind) is a `linter.pack_shape` labeled failure on
the pack file itself.

### `spk graph` and `spk compile` — same loop, pack-aware

```sh
spk graph lab-etl.md
```

```text
graph: 1 file(s), 5 node(s), 5 edge(s), 0 dangling, 0 typing violation(s), 0 supersedes cycle(s), 0 external boundary(ies)
→ Run: specodelic lint
```

```sh
spk compile lab-etl.md
```

```text
compile: 1 compiled, 0 failed
  lab-etl.md [lab.etl] → specodelic/lab-etl.toml, specodelic/lab-etl_props.rs, specodelic/lab-etl.tla
→ Run: specodelic verify (consumes the *_props.rs artifacts)
```

The declared `uses` edge resolves **corpus-wide**: with the pack file
in the linted corpus, both the dangling-reference finding and the
`orphan_vocabulary` finding clear. It is an **outbound leaf** — it
joins no reachability path and no acyclic edge set (mutual pack use
is well-formed), and v1's graph projection records no edge row for
it: pack enablement is a lint-level fact. Compile and verify treat a
pack-enabled spec like any other: pack vocabulary adds no
compile-time artifacts, it extends what lint understands.

## Where to go next

- The mechanism's normative home: [packs — domain-pack
  mechanism](specs/packs/spec.md) (every guarantee on this page is a
  constraint or property row there).
- The in-repo packs' capability specs:
  [data-lineage-pack](specs/data-lineage-pack/spec.md),
  [numeric-predicates-pack](specs/numeric-predicates-pack/spec.md),
  [empirical-registry-pack](specs/empirical-registry-pack/spec.md),
  [bioimage-data-pack](specs/bioimage-data-pack/spec.md),
  [quant-finance-pack](specs/quant-finance-pack/spec.md),
  [svc-contracts-pack](specs/svc-contracts-pack/spec.md).
- Writing a spec end-to-end (without packs): the
  [worked example](examples/worked-example.md).
- Rule semantics offline: `spk explain packs` and
  `spk explain lint-rules`.
