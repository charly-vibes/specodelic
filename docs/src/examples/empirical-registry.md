# Worked example — the empirical.registry pack, failures included

The [generic worked example](worked-example.md) builds a spec with the
core format alone, and [the data.lineage](data-lineage.md) and
[numeric.predicates](numeric-predicates.md) pages run the same journey
for the first domain packs. This page does it for
**[`empirical.registry`](https://github.com/charly-vibes/specodelic/tree/main/packs/empirical-registry.md)**
— the pack that adds a typed `## StatTests` registry, the
`empirical.statistic` fiber property kind, a `tested_by` reference
field, and a per-kind case-label floor enumerating **`alpha:`** and
**`window:`** on statistic properties — without a format revision and
without growing any base closed set. It is also the Floors facet's
first real consumer: the floor here is driven by statistical practice
(significance level plus observation window), not algebra. You will see
a small spec use the pack's vocabulary end to end, the pack-activation
advisory that proves the pack actually engaged, genuinely failing
variants with their real labeled findings, and — just as important —
the failures the linter does *not* catch yet, shown honestly.

How packs are enabled and what a pack adds mechanically is the job of
[using packs — the practical guide](../using-packs.md); read that for the
mechanism, this page for the hands-on loop. Every lint and graph block
below is real output from `specodelic 0.10.0` on the file as it stood at
that moment. On a TTY `spk` prints the human-readable text shown here;
piped, it emits a JSON envelope instead.

## The story

A growth team runs A/B experiments: each experiment measures
click-through rate and conversion, and when one concludes, the team
records a *lift claim* — an empirically held statement that variant B
beat variant A, supported by a registered significance test evaluated
at a declared alpha over a declared observation window. The spec's job
is to record those claims and the tests that back them — one
capability, one file, named `claim-lift.md`; the format's naming law
maps `-` to the namespace dot, so the frontmatter id is `claim.lift`.

Deliberately out of scope: *how* each test computes significance and
*what* the observation window means in wall-clock time. Those are the
`alpha` and `window` columns' job, and they stay opaque prose — the
pack declares the registry, statistical methodology stays authoritative
outside the format. Lint behavior is byte-identical no matter what the
columns carry.

## The setup

Packs are discovered by scanning the workspace, not by any config file:
every `kind: profile` file in the git repository toplevel of the linted
spec is a candidate pack. So the demo workspace needs two files — the
pack definition (copied from this repo's `packs/` directory) and the
spec under test:

```sh
mkdir demo && cd demo && git init
cp /path/to/specodelic/packs/empirical-registry.md .
```

That is the whole enablement story. The discovery and activation rules
(corpus-scan anchors, word-boundary vocabulary matching, the
byte-identical guarantee) are in [the guide](../using-packs.md); this
page just runs the loop.

## Layer 1 — Intent

```markdown
---
id: claim.lift
kind: intent
statement: "WHEN an experiment concludes THE system SHALL record every lift claim it holds together with the stat test and evidence window that supports it."
---

# Experiment lift claims

A small empirical registry for one growth team: each experiment measures
click-through and conversion, and every stated lift claim is an
empirically held statistic backed by a registered significance test over
a declared observation window.
```

Lint it as it stands:

```sh
spk lint claim-lift.md
```

```text
lint: 1 file(s) linted, 1 finding(s), 0 advisory warning(s)
  claim-lift.md [linter.model_present] Model section incomplete: 0 states, 0 transitions — both sections must exist (empty-but-present beats absent)
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

`linter.model_present` — a spec with an intent but no Model section is
incomplete by construction, same as in the generic worked example. Note
what is *missing* from this output: no pack advisory. The file uses none
of the pack's vocabulary yet, so the byte-identical guarantee holds —
the pack machinery is invisible. That changes with the next section.

## Layer 2 — the `## StatTests` registry

This is the pack's namespaced section, declared in the pack's
`## Sections` manifest with the fixed row shape
`| name | metric | alpha | window |`:

```markdown
## StatTests

| name   | metric     | alpha | window |
|--------|------------|-------|--------|
| ctr_t  | ctr        | 0.05  | 14d    |
| conv_t | conversion | 0.01  | 30d    |
```

Each column, defined for a cold read:

- **name** — the row's id inside this file; `tested_by` edges point at
  it.
- **metric** — free-form prose naming what the test measures (`ctr`,
  `conversion`). The pack does not constrain its values.
- **alpha** — opaque prose recording the test's significance level.
  The format never parses it.
- **window** — opaque prose recording the observation window the claim
  holds over. Same story: never interpreted.

The registry is deliberately thin: the pack adds *structure* (a place
tests live, names claims can point at), not statistics.

The pack's other two vocabulary pieces ride on the ordinary Properties
table. First, the fiber kind — type one property row's `kind` cell as
`empirical.statistic` (an *empirically held statistical claim, stated
with a declared stat test*, per the pack's `## Kinds` manifest). Base
kind columns accept pack-qualified fiber kinds *when the declaring pack
is active*; the same cell without the pack is a labeled finding, shown
below. Second, the reference field — `tested_by`, declared in the
pack's `## References` manifest, resolving to a `## StatTests` row of
the same file. It rides as a column on the Properties table, because a
claim is exactly the thing a stat test tests:

```markdown
| id              | kind | derives_from                     | generator                    | tested_by | predicate                                                       |
|-----------------|------|-----------------------------------|------------------------------|-----------|-----------------------------------------------------------------|
| claims_resolvable | unit | [[claim.lift.tests_registered]] | `experiments_with_registered_tests()` | ctr_t  | `every tested_by name resolves to a declared ## StatTests row`   |
| ctr_claim        | empirical.statistic | [[claim.lift.lift_tested]]       | `experiments_with_lift_claims()` | ctr_t     | `the CTR lift claim stands at the registered significance — **alpha:** the claim is evaluated at the test's declared alpha; **window:** the claim holds over the declared observation window` |
```

Read the rows aloud: every `tested_by` name points at a `## StatTests`
row declared above; the `ctr_claim` row *is* the empirically held lift
claim, typed with the pack's fiber kind and paying the pack's floor —
its predicate enumerates the two pack-required case labels,
**`alpha:`** and **`window:`**, on top of nothing else (it is not a
law, so the base algebra floor does not bite it).

The pack also declares two checkers — `empirical.stat_test_closed`,
*every `tested_by` name resolves to a `## StatTests` row declared in
the same file*, and `empirical.statistic_labels`, *statistic properties
enumerate their `alpha:` and `window:` case labels per the floor
declaration* — and one floor, in its `## Floors` manifest:
`empirical.statistic` properties owe **`alpha:`** and **`window:`**
case labels. What "declaring" buys you today is the next section's
honesty check, not enforcement — keep reading.

## Layer 3 — Constraints, Model, Properties

The rest of the file is the ordinary four-layer shape — the pack adds
vocabulary, it does not change the layers. Two invariants state the
registry claims, an `effect` constraint labels the failure output, an
`observes` edge ties the observer to the effect, and the model is a
five-state experiment lifecycle whose failure terminal emits. One
Properties row *is* a `law`, `significance_cases`, which enumerates
*four* case labels: the base format's mandatory **`identity:`** and
**`associativity:`** floor (`linter.law_cases` demands those of every
law) plus the pack floor's **`alpha:`** and **`window:`**:

```markdown
## Constraints

| id              | kind      | expr                                                                                      | traces_to      | observes                    |
|-----------------|-----------|-------------------------------------------------------------------------------------------|----------------|-----------------------------|
| tests_registered| invariant | `every stat test named by a claim is declared in the ## StatTests table of this file`      | [[claim.lift]] |                             |
| lift_tested     | invariant | `every empirically held lift claim names exactly one registered test as its tested_by source` | [[claim.lift]] |                          |
| lift_retracted  | effect    | `claim.lift.lift_retracted(detail)`                                                       | [[claim.lift]] |                             |
| retraction_surfaced | invariant | `every retracted claim reaches the operator with the offending test and window named` | [[claim.lift]] | [[claim.lift.lift_retracted]] |

## Model

### States

- `ramped`
- `measured`
- `tested`
- `recorded`
- `failed` (emits: `[[claim.lift.lift_retracted]]`)

### Transitions

| id       | from     | to       | guard                                       |
|----------|----------|----------|---------------------------------------------|
| measure  | ramped   | measured | `[[claim.lift.tests_registered]]`           |
| test     | measured | tested   | `[[claim.lift.lift_tested]]`                |
| record   | tested   | recorded | `window elapsed without retraction`         |
| fail     | ramped   | failed   | `¬[[claim.lift.tests_registered]]`          |

## Properties

| id              | kind | derives_from                     | generator                    | tested_by | predicate                                                       |
|-----------------|------|-----------------------------------|------------------------------|-----------|-----------------------------------------------------------------|
| claims_resolvable | unit | [[claim.lift.tests_registered]] | `experiments_with_registered_tests()` | ctr_t  | `every tested_by name resolves to a declared ## StatTests row`   |
| ctr_claim        | empirical.statistic | [[claim.lift.lift_tested]]       | `experiments_with_lift_claims()` | ctr_t     | `the CTR lift claim stands at the registered significance — **alpha:** the claim is evaluated at the test's declared alpha; **window:** the claim holds over the declared observation window` |
| lift_recorded   | unit | [[claim.lift.lift_tested]]       | `experiments_with_lift_claims()` | conv_t | `every lift claim carries its registered test source`            |
| retraction_emitted | unit | [[claim.lift.lift_retracted]] | `experiments_with_failing_tests()` | ctr_t | `the failure carries the lift_retracted label with detail`       |
| retraction_named | unit | [[claim.lift.retraction_surfaced]] | `experiments_with_failing_tests()` | conv_t | `the operator message names the offending test and window`       |
| significance_cases | law | [[claim.lift.tests_registered]] | `experiments_with_statistic_claims()` | ctr_t | `the lift claim holds in every declared case — **identity:** a claim tested against itself stands; **associativity:** composing tests is order-independent; **alpha:** significance is evaluated at the test's declared alpha; **window:** the claim holds only over the declared observation window` |
```

Nothing pack-specific in the layers themselves — `traces_to`, guards,
`emits`, coverage: all core-format machinery, taught properly in the
[generic worked example](worked-example.md).

## First full lint — a dangling reference, then the fix

The author's first full draft carries one typo: the `lift_tested`
constraint traces to `[[claim.lifst]]` — letters transposed, the same
classic as the reservation example. Lint the file:

```sh
spk lint claim-lift.md
```

```text
lint: 1 file(s) linted, 1 finding(s), 1 advisory warning(s)
  claim-lift.md [linter.total_refs] dangling reference `[[claim.lifst]]` from constraints.traces_to — target not defined in any spec file
  advisory claim.lift [linter.observability] pack `empirical.registry` activated for `claim.lift` (vocabulary match: empirical.statistic, StatTests, tested_by) — empty checked-set (no violations to report)
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

Exit 1 — a finding, never a silent pass. Two things to read here:

1. **The finding and the advisory ride different channels.** The pack
   activated (its vocabulary is all over the file — note the match list
   names every token that triggered it: the fiber kind, the section
   name, the reference field), but activation lives on the *warnings*
   channel and never gates. The dangling reference lives on the
   *findings* channel and does.
2. **`empty checked-set (no violations to report)`** — the pack's
   declared checkers ran and honestly report an empty violation set.
   More on what that does and does not mean below.

Fix the typo — `[[claim.lifst]]` → `[[claim.lift]]` — and lint again:

```text
lint: 1 file(s) linted, 0 finding(s), 1 advisory warning(s)
  advisory claim.lift [linter.observability] pack `empirical.registry` activated for `claim.lift` (vocabulary match: empirical.statistic, StatTests, tested_by) — empty checked-set (no violations to report)
→ Run: specodelic graph
```

Zero findings, exit 0 — and now the advisory is the *whole* output.
Read it as the meter it is: an exit-0 lint alone proves nothing (a spec
using none of the pack vocabulary also exits 0, silently); **the
activation line naming `empirical.registry` is the observable** that
the pack was discovered, matched, and engaged. The pipeline's next
stage runs pack-aware too:

```sh
spk graph claim-lift.md
```

```text
graph: 1 file(s), 19 node(s), 22 edge(s), 0 dangling, 0 typing violation(s), 0 supersedes cycle(s), 0 external boundary(ies)
→ Run: specodelic lint
```

## What a missing pack looks like

The demo depends on the pack file being *discovered*. Copy the finished
spec into a directory with no `kind: profile` file in it — the
machinery goes inert:

```text
lint: 1 file(s) linted, 2 finding(s), 0 advisory warning(s)
  claim.lift [linter.property_kind_closed] property `ctr_claim` has kind `empirical.statistic` — outside the closed set {unit, law} (kinds.md property_row_shape)
  claim.lift [linter.orphan_vocabulary] orphan vocabulary: pack-qualified token(s) `empirical.statistic` used by `claim.lift`, but no `kind: profile` pack in namespace `empirical` is discovered — candidate pack `empirical.*` (prefix-derived: only the namespace is known); remediations: add/enable a `kind: profile` pack file declaring the vocabulary, or fix the vocabulary (un-qualify or retype the token)
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

Different from the sibling packs' story, and worth pausing on: this
spec does *not* lint silently without its pack, because it uses the
pack's fiber kind in a base-table kind column. A fiber kind's
typeability *is* the pack — with no active declaring pack, the
`empirical.statistic` cell is outside the base closed set
(`linter.property_kind_closed`) and the token itself is orphan
vocabulary (`linter.orphan_vocabulary`, naming the candidate
`empirical.*` and both remediations: enable a pack declaring the
vocabulary, or fix the token). The byte-identical guarantee still holds
exactly as specified — it covers files using *none* of the pack
vocabulary. Retype the one fiber-kind cell back to `unit` in that same
pack-less directory and the file goes quiet again:

```text
lint: 1 file(s) linted, 0 finding(s), 0 advisory warning(s)
→ Run: specodelic graph
```

No advisory, no findings — a spec with no pack vocabulary is markdown
the pack machinery never interprets, by design: a pack can never gate
a file that did not opt in. The reliable diagnostic either way: a spec
you *meant* to be pack-enabled but lint with no activation advisory is
a spec whose pack was not discovered.

## What the linter does not catch (yet)

Honesty check — the failures this page has *not* shown are the two you
might expect first, because the pack declares a checker for each.

**A `tested_by` edge that dangles.** Change `ctr_t` to `ghost_t` in the
`tested_by` column and lint the pack-enabled workspace:

```text
lint: 1 file(s) linted, 0 finding(s), 1 advisory warning(s)
  advisory claim.lift [linter.observability] pack `empirical.registry` activated for `claim.lift` (vocabulary match: empirical.statistic, StatTests, tested_by) — empty checked-set (no violations to report)
→ Run: specodelic graph
```

Byte-for-byte the clean output — exit 0, activation advisory, nothing
else. That is not the linter missing the dangle by accident:
`empirical.stat_test_closed` ships as a **declaration**.

**Floor labels dropped.** Strip the **`alpha:`** and **`window:`** case
labels out of `ctr_claim`'s predicate and lint:

```text
lint: 1 file(s) linted, 0 finding(s), 1 advisory warning(s)
  advisory claim.lift [linter.observability] pack `empirical.registry` activated for `claim.lift` (vocabulary match: empirical.statistic, StatTests, tested_by) — empty checked-set (no violations to report)
→ Run: specodelic graph
```

Identical again. `empirical.statistic_labels` is the same kind of
declaration.

In v1, pack checkers are honest-empty — activation and the empty
checked-set are reported truthfully, violation *evaluation* is not
implemented yet (the [guide](../using-packs.md) documents this, and
[verification boundaries](../verification-boundaries.md) places it on
the guarantee ladder). The format's answer to the gap is the one this
spec already embodies: each unenforced claim lives where the evidence
plan lives — the dangling-edge claim in the `tests_registered`
invariant and its `claims_resolvable` property, the floor in the
`ctr_claim` and `significance_cases` predicates' enumerated cases —
each waiting on a real executable predicate. When the checkers gain
evaluation, the declarations are already in place; the spec does not
change.

## The finished spec

Every block above assembles into this one file, `claim-lift.md` —
exactly as it lints at 0 findings, 1 activation advisory:

```markdown
---
id: claim.lift
kind: intent
statement: "WHEN an experiment concludes THE system SHALL record every lift claim it holds together with the stat test and evidence window that supports it."
---

# Experiment lift claims

A small empirical registry for one growth team: each experiment measures
click-through and conversion, and every stated lift claim is an
empirically held statistic backed by a registered significance test over
a declared observation window.

## StatTests

| name   | metric     | alpha | window |
|--------|------------|-------|--------|
| ctr_t  | ctr        | 0.05  | 14d    |
| conv_t | conversion | 0.01  | 30d    |

## Constraints

| id              | kind      | expr                                                                                      | traces_to      | observes                    |
|-----------------|-----------|-------------------------------------------------------------------------------------------|----------------|-----------------------------|
| tests_registered| invariant | `every stat test named by a claim is declared in the ## StatTests table of this file`      | [[claim.lift]] |                             |
| lift_tested     | invariant | `every empirically held lift claim names exactly one registered test as its tested_by source` | [[claim.lift]] |                          |
| lift_retracted  | effect    | `claim.lift.lift_retracted(detail)`                                                       | [[claim.lift]] |                             |
| retraction_surfaced | invariant | `every retracted claim reaches the operator with the offending test and window named` | [[claim.lift]] | [[claim.lift.lift_retracted]] |

## Model

### States

- `ramped`
- `measured`
- `tested`
- `recorded`
- `failed` (emits: `[[claim.lift.lift_retracted]]`)

### Transitions

| id       | from     | to       | guard                                       |
|----------|----------|----------|---------------------------------------------|
| measure  | ramped   | measured | `[[claim.lift.tests_registered]]`           |
| test     | measured | tested   | `[[claim.lift.lift_tested]]`                |
| record   | tested   | recorded | `window elapsed without retraction`         |
| fail     | ramped   | failed   | `¬[[claim.lift.tests_registered]]`          |

## Properties

| id              | kind | derives_from                     | generator                    | tested_by | predicate                                                       |
|-----------------|------|-----------------------------------|------------------------------|-----------|-----------------------------------------------------------------|
| claims_resolvable | unit | [[claim.lift.tests_registered]] | `experiments_with_registered_tests()` | ctr_t  | `every tested_by name resolves to a declared ## StatTests row`   |
| ctr_claim        | empirical.statistic | [[claim.lift.lift_tested]]       | `experiments_with_lift_claims()` | ctr_t     | `the CTR lift claim stands at the registered significance — **alpha:** the claim is evaluated at the test's declared alpha; **window:** the claim holds over the declared observation window` |
| lift_recorded   | unit | [[claim.lift.lift_tested]]       | `experiments_with_lift_claims()` | conv_t | `every lift claim carries its registered test source`            |
| retraction_emitted | unit | [[claim.lift.lift_retracted]] | `experiments_with_failing_tests()` | ctr_t | `the failure carries the lift_retracted label with detail`       |
| retraction_named | unit | [[claim.lift.retraction_surfaced]] | `experiments_with_failing_tests()` | conv_t | `the operator message names the offending test and window`       |
| significance_cases | law | [[claim.lift.tests_registered]] | `experiments_with_statistic_claims()` | ctr_t | `the lift claim holds in every declared case — **identity:** a claim tested against itself stands; **associativity:** composing tests is order-independent; **alpha:** significance is evaluated at the test's declared alpha; **window:** the claim holds only over the declared observation window` |
```

To reproduce every block on this page: copy
`packs/empirical-registry.md` from the repo into a fresh git workspace,
save the file above as `claim-lift.md`, and run
`spk lint claim-lift.md`.

## Where to go next

- The mechanism, end to end: [using packs — the practical
  guide](../using-packs.md) — discovery, activation, honest-empty
  semantics, the full CLI loop.
- The pack's normative home: [packs — domain-pack
  mechanism](../specs/packs/spec.md), and the pack definition itself —
  [`packs/empirical-registry.md`](https://github.com/charly-vibes/specodelic/tree/main/packs/empirical-registry.md)
  is the source of truth for the vocabulary this page used.
- The capability spec behind the pack:
  [empirical-registry-pack](../specs/empirical-registry-pack/spec.md).
- The same journey without packs: [worked example —
  reservation/order](worked-example.md), and with the sibling packs:
  [data.lineage](data-lineage.md) and
  [numeric.predicates](numeric-predicates.md).
- Rule semantics offline: `spk explain packs` and
  `spk explain lint-rules`.
