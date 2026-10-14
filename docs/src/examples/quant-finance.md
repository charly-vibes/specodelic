# Worked example — the quant.finance pack, failures included

The [generic worked example](worked-example.md) builds a spec with the
core format alone, and the [data.lineage](data-lineage.md),
[numeric.predicates](numeric-predicates.md),
[empirical.registry](empirical-registry.md), and
[bioimage.data](bioimage-data.md) pages run the same journey pack by
pack. This page does it for
**[`quant.finance`](https://github.com/charly-vibes/specodelic/tree/main/packs/quant-finance.md)**
— the mechanism's *restraint instance*: most of what a quant desk would
ask for already ships in the packs it requires (tolerance semantics in
`numeric.predicates`, statistics in `empirical.registry`, backtest
fixtures and datasets in `data.lineage`), so this pack declares only the
vocabulary no other pack carries — a typed `## Limits` section for
bounded risk measures and pricing invariants, the `quant.risk` and
`quant.pricing` kinds, and the `capped_by` outbound-leaf edge. Unit
systems stay outside: ISO4217 currency codes and market-data conventions
remain authoritative external standards, bridged through the opaque
`unit` column, never absorbed into any closed set. You will see a small
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

A prop trading desk keeps a risk register: an overnight VaR limit, the
expected-shortfall limit that reconciles against it, and a NAV
consistency bound. The spec's job is to record that register — every
limit with its unit and bound, and the cap edge between the two risk
limits. One capability, one file, named `desk-risks.md`; the format's
naming law maps `-` to the namespace dot, so the frontmatter id is
`desk.risks`.

Deliberately out of scope: *which* external standard each unit comes
from and *where* the market data pricing the bounds lives. `USD` is an
ISO4217 code, `bps` is a market convention — the pack treats both as
opaque strings in the `unit` column and never interprets them.

## The setup

Packs are discovered by scanning the workspace, not by any config file:
every `kind: profile` file in the git repository toplevel of the linted
spec is a candidate pack. So the demo workspace needs two files — the
pack definition (copied from this repo's `packs/` directory) and the
spec under test:

```sh
mkdir demo && cd demo && git init
cp /path/to/specodelic/packs/quant-finance.md .
```

That is the whole enablement story. The pack's `## Requires` table also
declares `numeric.predicates` and `data.lineage` — but those are
*complementary* vocabulary, not prerequisites for this example: backtest
fixtures and datasets are `## Data` rows (`data.lineage`), tolerance
semantics live in `numeric.predicates`, and the spec below uses neither.
Copying just the pack above is enough; the discovery and activation
rules (corpus-scan anchors, word-boundary vocabulary matching, the
byte-identical guarantee) are in [the guide](../using-packs.md).

## Layer 1 — Intent

```markdown
---
id: desk.risks
kind: intent
statement: "WHEN the desk books trades against its risk limits THE system SHALL record every bounded risk measure and pricing invariant together with the cap edges between them."
---

# Desk risk register

A small risk register for one prop trading desk: overnight VaR, its
expected-shortfall companion, and the NAV consistency bound — each
stated with its unit and bound, and the cap edges between them.
```

Lint it as it stands:

```sh
spk lint desk-risks.md
```

```text
lint: 1 file(s) linted, 1 finding(s), 0 advisory warning(s)
  desk-risks.md [linter.model_present] Model section incomplete: 0 states, 0 transitions — both sections must exist (empty-but-present beats absent)
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

`linter.model_present` — a spec with an intent but no Model section is
incomplete by construction, same as in the generic worked example. Note
what is *missing* from this output: no pack advisory. The file uses none
of the pack's vocabulary yet, so the byte-identical guarantee holds —
the pack machinery is invisible. That changes with the next section.

## Layer 2 — the `## Limits` section

This is the pack's namespaced section, declared in the pack's
`## Sections` manifest with the fixed row shape
`| name | kind | unit | bound |`:

```markdown
## Limits

| name      | kind          | unit | bound   |
|-----------|---------------|------|---------|
| var_1d    | quant.risk    | USD  | 250_000 |
| es_1d     | quant.risk    | USD  | 400_000 |
| nav_drift | quant.pricing | bps  | 25      |
```

Each column, defined for a cold read:

- **name** — the row's id inside this file; cap edges point at it.
- **kind** — a short label closed *per pack* to `risk` and `pricing`
  (a risk measure vs. a valuation invariant). These are the row-level
  form of the pack's `quant.risk`/`quant.pricing` kinds — and note the
  boundary: the mechanism **never checks row-kind cells**. The closure
  is the pack's own prose law; what the linter enforces is the
  pack-qualified *kind tokens* in the Properties layer, below.
- **unit** — opaque. `USD` is an ISO4217 code, `bps` a market
  convention; the pack declares no syntax for either and the linter
  never interprets the column. The external standards stay
  authoritative — the pack bridges them, it does not absorb them.
- **bound** — the limit's numeric value. Also opaque prose; the
  mechanism never parses it.

Cap edges are the pack's reference field, declared in its
`## References` manifest — `capped_by`, resolving to a `## Limits` row
of the same file. It rides as a column on a second table under the same
section:

```markdown
| row    | capped_by |
|--------|-----------|
| es_1d  | var_1d    |
```

Read the edge aloud: `es_1d` is capped by `var_1d` — the
expected-shortfall limit names the VaR limit it reconciles against.

The pack also declares two checkers in its `## Checkers` manifest —
`quant.limit_closed` (*every `capped_by` name resolves to a `## Limits`
row declared in the same file*) and `quant.risk_labels` (*`quant.risk`
laws enumerate their horizon and confidence case labels per the pack's
floor*). Both ship as declarations: the mechanism names them, it does
not yet execute them — keep reading, the page shows exactly what that
means. The same manifest style declares the pack's floor: law-kind
properties about `quant.risk` rows must carry **`horizon:`** and
**`confidence:`** case labels. The base law floor is untouched — the
mandatory `identity:`/`associativity:` labels ride base `law` rows (see
[the numeric.predicates page](numeric-predicates.md)); this floor is
*kind-dependent* and rides the fiber kind, as the Properties layer
below pays it.

Lint the file with the section in place, Model still missing:

```text
lint: 1 file(s) linted, 1 finding(s), 1 advisory warning(s)
  desk-risks.md [linter.model_present] Model section incomplete: 0 states, 0 transitions — both sections must exist (empty-but-present beats absent)
  advisory desk.risks [linter.observability] pack `quant.finance` activated for `desk.risks` (vocabulary match: quant.risk, quant.pricing, Limits, capped_by) — empty checked-set (no violations to report)
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

There is the activation advisory, and the match list names every token
that triggered it: both fiber kinds, the section name, and the
reference field.

## Layer 3 — Constraints, Model, Properties

The rest of the file is the ordinary four-layer shape — the pack adds
vocabulary, it does not change the layers. Two invariants state the
register's claims, an `effect` constraint labels the failure output, an
`observes` edge ties the observer to the effect, and the model is a
four-state day lifecycle whose failure terminal emits:

```markdown
## Constraints

| id              | kind      | expr                                                                                   | traces_to      | observes                     |
|-----------------|-----------|----------------------------------------------------------------------------------------|----------------|------------------------------|
| caps_closed     | invariant | `every capped_by name resolves to a ## Limits row declared in this file`                 | [[desk.risks]] |                              |
| limit_breach    | effect    | `desk.risks.limit_breach(detail)`                                                       | [[desk.risks]] |                              |
| breach_surfaced | invariant | `every limit breach reaches the risk officer with the offending row and its bound named` | [[desk.risks]] | [[desk.risks.limit_breach]]  |

## Model

### States

- `booked`
- `checked`
- `published`
- `failed` (emits: `[[desk.risks.limit_breach]]`)

### Transitions

| id      | from    | to        | guard                              |
|---------|---------|-----------|------------------------------------|
| check   | booked  | checked   | `[[desk.risks.caps_closed]]`       |
| publish | checked | published | `end-of-day run written`           |
| fail    | booked  | failed    | `¬[[desk.risks.caps_closed]]`      |

## Properties

| id             | kind       | derives_from                   | generator                    | predicate                                                                                                                                                    |
|----------------|------------|--------------------------------|------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------|
| caps_resolve   | unit       | [[desk.risks.caps_closed]]     | `registers_with_cap_edges()` | `every capped_by name resolves to a declared ## Limits row`                                                                                                   |
| breach_emitted | unit       | [[desk.risks.limit_breach]]    | `registers_with_breaches()`  | `the failure carries the desk.risks.limit_breach label with detail`                                                                                           |
| breach_named   | unit       | [[desk.risks.breach_surfaced]] | `registers_with_breaches()`  | `the risk-officer message names the offending ## Limits row and its bound`                                                                                     |
| es_reconciles  | quant.risk | [[desk.risks.caps_closed]]     | `registers_with_es_limits()` | `the ES limit reconciles against the VaR limit in every declared case — **horizon:** both limits are stated over the same 1-day horizon; **confidence:** both limits assume the same 99% confidence level` |
```

One pack-specific row: `es_reconciles` is a law whose kind is the fiber
kind `quant.risk` — the Properties layer paying the pack's floor, its
predicate enumerating **`horizon:`** and **`confidence:`** as named
cases. Everything else — `traces_to`, guards, `emits`, coverage — is
core-format machinery, taught properly in the
[generic worked example](worked-example.md).

## First full lint — a dangling reference, then the fix

The author's first full draft carries one typo: the `caps_closed`
constraint traces to `[[desk.riss]]` — misremembered, the same classic
as the reservation example. Lint the file:

```sh
spk lint desk-risks.md
```

```text
lint: 1 file(s) linted, 1 finding(s), 1 advisory warning(s)
  desk-risks.md [linter.total_refs] dangling reference `[[desk.riss]]` from constraints.traces_to — target not defined in any spec file
  advisory desk.risks [linter.observability] pack `quant.finance` activated for `desk.risks` (vocabulary match: quant.risk, quant.pricing, Limits, capped_by) — empty checked-set (no violations to report)
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

Exit 1 — a finding, never a silent pass. Two things to read here:

1. **The finding and the advisory ride different channels.** The pack
   activated (its vocabulary is all over the file — note the match list
   names every token that triggered it), but activation lives on the
   *warnings* channel and never gates. The dangling reference lives on
   the *findings* channel and does.
2. **`empty checked-set (no violations to report)`** — the pack's
   declared `quant.limit_closed` checker ran and honestly reports an
   empty violation set. More on what that does and does not mean below.

Fix the typo — `[[desk.riss]]` → `[[desk.risks]]` — and lint again:

```text
lint: 1 file(s) linted, 0 finding(s), 1 advisory warning(s)
  advisory desk.risks [linter.observability] pack `quant.finance` activated for `desk.risks` (vocabulary match: quant.risk, quant.pricing, Limits, capped_by) — empty checked-set (no violations to report)
→ Run: specodelic graph
```

Zero findings, exit 0 — and now the advisory is the *whole* output.
Read it as the meter it is: an exit-0 lint alone proves nothing (a spec
using none of the pack vocabulary also exits 0, silently); **the
activation line naming `quant.finance` is the observable** that the
pack was discovered, matched, and engaged. The pipeline's next stage
runs pack-aware too:

```sh
spk graph desk-risks.md
```

```text
graph: 1 file(s), 15 node(s), 17 edge(s), 0 dangling, 0 typing violation(s), 0 supersedes cycle(s), 0 external boundary(ies)
→ Run: specodelic lint
```

## What a missing pack looks like

The demo depends on the pack file being *discovered*. Lint the same
finished spec in a **fresh git workspace** with no `kind: profile` file
in it — not a subdirectory of the first workspace, whose repo toplevel
the discovery scan still anchors to — and the machinery goes inert…
almost. First, the truly silent case: drop the one fiber-kind row (the
`es_reconciles` law, whose kind cell is pack vocabulary) and lint:

```text
lint: 1 file(s) linted, 0 finding(s), 0 advisory warning(s)
→ Run: specodelic graph
```

No advisory, no findings — the byte-identical guarantee in action. The
`## Limits` table is now just markdown the linter does not interpret,
and that is by design: a pack can never gate a file that did not opt
in. But inert cuts both ways, and the linter tells you when vocabulary
*expects* a pack. Restore the law row and lint in that same pack-less
workspace:

```text
lint: 1 file(s) linted, 2 finding(s), 0 advisory warning(s)
  desk.risks [linter.property_kind_closed] property `es_reconciles` has kind `quant.risk` — outside the closed set {unit, law} (kinds.md property_row_shape)
  desk.risks [linter.orphan_vocabulary] orphan vocabulary: pack-qualified token(s) `quant.risk` used by `desk.risks`, but no `kind: profile` pack in namespace `quant` is discovered — candidate pack `quant.*` (prefix-derived: only the namespace is known); remediations: add/enable a `kind: profile` pack file declaring the vocabulary, or fix the vocabulary (un-qualify or retype the token)
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

`linter.orphan_vocabulary` — the finding always names the candidate
(namespace-derived, `quant.*` here) and states **both** remediations:
enable a pack declaring the vocabulary, or fix the token. Copy the pack
file back into the workspace and the pair clears — with the pack
active, the walkers accept a fiber kind in a kind column
(base ∪ active-pack-fiber), so that mis-typing only surfaces when the
pack is missing. The reliable diagnostic is the one this section
started with: a spec you *meant* to be pack-enabled but lint with no
activation advisory is a spec whose pack was not discovered.

## What the linter does not catch (yet)

Honesty check — the failures this page has *not* shown are the ones you
might expect first. Change `var_1d` to `ghost_row` in the `capped_by`
column and lint:

```text
lint: 1 file(s) linted, 0 finding(s), 1 advisory warning(s)
  advisory desk.risks [linter.observability] pack `quant.finance` activated for `desk.risks` (vocabulary match: quant.risk, quant.pricing, Limits, capped_by) — empty checked-set (no violations to report)
→ Run: specodelic graph
```

Byte-for-byte the clean output — exit 0, activation advisory, nothing
else. That is not the linter missing the dangle by accident:
`quant.limit_closed` ships as a **declaration**. In v1, pack checkers
are honest-empty — activation and the empty checked-set are reported
truthfully, violation *evaluation* is not implemented yet (the
[guide](../using-packs.md) documents this, and
[verification boundaries](../verification-boundaries.md) places it on
the guarantee ladder). The same applies to the floor: drop the
**`horizon:`** and **`confidence:`** case labels from `es_reconciles`
and the lint output is identical — `quant.risk_labels` is also named,
not executed.

The format's answer to the gap is the one this spec already embodies:
the dangling-edge claim lives where the evidence plan lives — the
`caps_closed` invariant, its `caps_resolve` property, and eventually a
real executable predicate. When the checkers gain evaluation, the
declarations are already in place; the spec does not change.

## The finished spec

Every block above assembles into this one file, `desk-risks.md` —
exactly as it lints at 0 findings, 1 activation advisory:

```markdown
---
id: desk.risks
kind: intent
statement: "WHEN the desk books trades against its risk limits THE system SHALL record every bounded risk measure and pricing invariant together with the cap edges between them."
---

# Desk risk register

A small risk register for one prop trading desk: overnight VaR, its
expected-shortfall companion, and the NAV consistency bound — each
stated with its unit and bound, and the cap edges between them.

## Limits

| name      | kind          | unit | bound   |
|-----------|---------------|------|---------|
| var_1d    | quant.risk    | USD  | 250_000 |
| es_1d     | quant.risk    | USD  | 400_000 |
| nav_drift | quant.pricing | bps  | 25      |

| row    | capped_by |
|--------|-----------|
| es_1d  | var_1d    |

## Constraints

| id              | kind      | expr                                                                                   | traces_to      | observes                     |
|-----------------|-----------|----------------------------------------------------------------------------------------|----------------|------------------------------|
| caps_closed     | invariant | `every capped_by name resolves to a ## Limits row declared in this file`                 | [[desk.risks]] |                              |
| limit_breach    | effect    | `desk.risks.limit_breach(detail)`                                                       | [[desk.risks]] |                              |
| breach_surfaced | invariant | `every limit breach reaches the risk officer with the offending row and its bound named` | [[desk.risks]] | [[desk.risks.limit_breach]]  |

## Model

### States

- `booked`
- `checked`
- `published`
- `failed` (emits: `[[desk.risks.limit_breach]]`)

### Transitions

| id      | from    | to        | guard                              |
|---------|---------|-----------|------------------------------------|
| check   | booked  | checked   | `[[desk.risks.caps_closed]]`       |
| publish | checked | published | `end-of-day run written`           |
| fail    | booked  | failed    | `¬[[desk.risks.caps_closed]]`      |

## Properties

| id             | kind       | derives_from                   | generator                    | predicate                                                                                                                                                    |
|----------------|------------|--------------------------------|------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------|
| caps_resolve   | unit       | [[desk.risks.caps_closed]]     | `registers_with_cap_edges()` | `every capped_by name resolves to a declared ## Limits row`                                                                                                   |
| breach_emitted | unit       | [[desk.risks.limit_breach]]    | `registers_with_breaches()`  | `the failure carries the desk.risks.limit_breach label with detail`                                                                                           |
| breach_named   | unit       | [[desk.risks.breach_surfaced]] | `registers_with_breaches()`  | `the risk-officer message names the offending ## Limits row and its bound`                                                                                     |
| es_reconciles  | quant.risk | [[desk.risks.caps_closed]]     | `registers_with_es_limits()` | `the ES limit reconciles against the VaR limit in every declared case — **horizon:** both limits are stated over the same 1-day horizon; **confidence:** both limits assume the same 99% confidence level` |
```

To reproduce every block on this page: copy
`packs/quant-finance.md` from the repo into a fresh git workspace, save
the file above as `desk-risks.md`, and run `spk lint desk-risks.md`.

## Where to go next

- The mechanism, end to end: [using packs — the practical
  guide](../using-packs.md) — discovery, activation, honest-empty
  semantics, the full CLI loop.
- The pack's normative home: [packs — domain-pack
  mechanism](../specs/packs/spec.md), and the pack definition itself —
  [`packs/quant-finance.md`](https://github.com/charly-vibes/specodelic/tree/main/packs/quant-finance.md)
  is the source of truth for the vocabulary this page used.
- The capability spec behind the pack:
  [quant-finance-pack](../specs/quant-finance-pack/spec.md).
- The packs this one requires for the rest of a quant desk's needs:
  [the numeric.predicates pack](numeric-predicates.md) (tolerance
  semantics) and [the data.lineage pack](data-lineage.md) (backtest
  fixtures and datasets) — or the journey without packs at all:
  [worked example — reservation/order](worked-example.md).
- Rule semantics offline: `spk explain packs` and
  `spk explain lint-rules`.
