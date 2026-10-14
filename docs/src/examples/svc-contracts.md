# Worked example — the svc.contracts pack, failures included

The [generic worked example](worked-example.md) builds a spec with the
core format alone, and the [data.lineage](data-lineage.md),
[numeric.predicates](numeric-predicates.md),
[empirical.registry](empirical-registry.md),
[bioimage.data](bioimage-data.md), and
[quant.finance](quant-finance.md) pages run the same journey pack by
pack. This page does it for
**[`svc.contracts`](https://github.com/charly-vibes/specodelic/tree/main/packs/svc-contracts.md)**
— the pack for services whose published contracts live in different
repositories: a typed `## Interfaces` registry section (name, kind,
repo, binding, evolution), the `svc.compat` compatibility law kind, and
the `published_as` outbound-leaf reference field. It adds no cross-file
edge and no consumer listing: the base format already carries those
(`extension_point` + `satisfies` for request contracts, `effect` +
`observes` for events), and `USAGE.md` §2.6 forbids a publisher from
enumerating its conformers. External contracts — OpenAPI, AsyncAPI,
protobuf, JSON Schema — stay authoritative through an opaque `binding`
column the pack never parses or absorbs.

> **Lifecycle: DRAFT.** `svc.contracts` is the one standard pack still
> at draft — its pack file's Model names the single state `draft`
> ([pack definition](https://github.com/charly-vibes/specodelic/tree/main/packs/svc-contracts.md),
> proposed by the archived `add-svc-contracts-pack` change). What that
> means for you today: the pack activates by vocabulary match only,
> everything it produces rides the advisory (warnings) channel with
> exit 0, and the activation advisory itself carries the suffix
> `— draft status: findings are advisory-first and name it`. Nothing a
> draft pack does can block a lint. Its vocabulary already parses (the
> fiber kinds extend the closed kind sets), while its checkers are
> honest-empty declarations — the same v1 semantics every pack ships.
> Every lint block below shows where the draft status appears; this
> page is a **draft preview**, not a promise of published-pack
> behavior, and promotion is a maintainer decision gated on a second
> independent consumer.

You will see a small spec use the pack's vocabulary end to end, the
pack-activation advisory that proves the pack actually engaged (draft
suffix included), genuinely failing variants with their real labeled
findings, and — just as important — the failures the linter does *not*
catch yet, shown honestly.

How packs are enabled and what a pack adds mechanically is the job of
[using packs — the practical guide](../using-packs.md); read that for
the mechanism, this page for the hands-on loop. Every lint and graph
block below is real output from `specodelic 0.10.0` on the file as it
stood at that moment. On a TTY `spk` prints the human-readable text
shown here; piped, it emits a JSON envelope instead.

## The story

A checkout team publishes three contracts, and they do not live in one
place: the cart request contract and the order-placed event are owned
by the backend API repo, while the order-summary schema is owned by the
web repo. The spec's job is to register those three contracts — which
repo owns each, which authoritative external artifact each binds to,
how each is allowed to evolve — and to state each change's consumer
compatibility as a law. One capability, one file, named
`checkout-api.md`; the format's naming law maps `-` to the namespace
dot, so the frontmatter id is `checkout.api`.

Deliberately out of scope: *what any contract's consumers look like*
and *what the OpenAPI/AsyncAPI/JSON Schema files say inside*. The
publisher never lists its conformers (the base `satisfies`/`observes`
edges carry that direction, derived from the graph), and the binding
column stays opaque — `openapi://…`, `asyncapi://…`,
`json-schema://…` are pointers the linter never interprets.

## The setup

Packs are discovered by scanning the workspace, not by any config file:
every `kind: profile` file in the git repository toplevel of the linted
spec is a candidate pack. So the demo workspace needs two files — the
pack definition (copied from this repo's `packs/` directory) and the
spec under test:

```sh
mkdir demo && cd demo && git init
cp /path/to/specodelic/packs/svc-contracts.md .
```

That is the whole enablement story. The discovery and activation rules
(corpus-scan anchors, word-boundary vocabulary matching, the
byte-identical guarantee) are in [the guide](../using-packs.md); this
page just runs the loop.

## Layer 1 — Intent

```markdown
---
id: checkout.api
kind: intent
statement: "WHEN the checkout team publishes or changes a service contract THE system SHALL register every published contract with its owning repo, its authoritative external binding, its declared evolution mode, and the consumer compatibility law each change must satisfy."
---

# Checkout service contracts

A contract registry for one checkout team: a cart request contract, an
order-placed event, and an order-summary schema — each registered with
the repo that owns it and the authoritative external artifact it binds
to.
```

Lint it as it stands:

```sh
spk lint checkout-api.md
```

```text
lint: 1 file(s) linted, 1 finding(s), 0 advisory warning(s)
  checkout-api.md [linter.model_present] Model section incomplete: 0 states, 0 transitions — both sections must exist (empty-but-present beats absent)
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

`linter.model_present` — a spec with an intent but no Model section is
incomplete by construction, same as in the generic worked example.
Note what is *missing* from this output: no pack advisory. The file
uses none of the pack's vocabulary yet, so the byte-identical guarantee
holds — the pack machinery is invisible. That changes with the next
section.

## Layer 2 — the `## Interfaces` section

This is the pack's namespaced section, declared in the pack's
`## Sections` manifest with the fixed row shape
`| name | kind | repo | binding | evolution |`:

```markdown
## Interfaces

| name          | kind    | repo              | binding                               | evolution |
|---------------|---------|-------------------|---------------------------------------|-----------|
| cart_api      | request | acme/checkout-api | openapi://specs/cart-v2.yaml          | additive  |
| order_placed  | event   | acme/checkout-api | asyncapi://specs/order-placed-v1.yaml | versioned |
| order_summary | schema  | acme/checkout-web | json-schema://order-summary-v3.json   | frozen    |
```

Each column, defined for a cold read:

- **name** — the row's id inside this file; contract rows point at it
  through `published_as` (Layer 3).
- **kind** — a short label closed *per pack* to `request` (a consumer
  conforms to it via the base `satisfies` edge), `event` (consumers
  watch it via the base `observes` edge), and `schema` (a shared data
  shape). Boundary note, same as every pack: the mechanism **never
  checks row-kind cells** — the closure is the pack's own prose law.
- **repo** — opaque. The repo that owns and publishes the contract;
  the pack declares no syntax for it and the linter never parses it.
- **binding** — opaque. The pointer to the authoritative external
  artifact — an OpenAPI or AsyncAPI document, a protobuf definition, a
  JSON Schema file. The pack bridges to those standards, it never
  absorbs them: the `.yaml`/`.proto`/`.json` file stays the system of
  record, and nothing in this spec is allowed to contradict it.
- **evolution** — closed *per pack* to `additive` (only compatible
  additions), `versioned` (breaking changes ship as new versions), and
  `frozen` (no changes at all). Also unchecked by the mechanism — the
  closure is prose.

The pack declares one reference field in its `## References` manifest —
`published_as`, resolving to a `## Interfaces` row *of the declaring
file* — and two checkers, `svc.interface_closed` (*every `published_as`
name resolves to an `## Interfaces` row declared in the same file*) and
`svc.compat_cases` (*`svc.compat` laws enumerate their added and
removed case labels per the pack's floor*). Both ship as declarations:
the mechanism names them, it does not yet execute them — keep reading,
the page shows exactly what that means. The same manifest style
declares the pack's floor: law-kind properties of kind `svc.compat`
must carry **`added:`** and **`removed:`** case labels — the two cases
that decide whether existing consumers keep working. The base law floor
is untouched — the mandatory `identity:`/`associativity:` labels ride
base `law` rows (see [the numeric.predicates page](numeric-predicates.md));
this floor is *kind-dependent* and rides the fiber kind, as the
Properties layer below pays it.

Lint with the section in place, Model still missing:

```text
lint: 1 file(s) linted, 1 finding(s), 1 advisory warning(s)
  checkout-api.md [linter.model_present] Model section incomplete: 0 states, 0 transitions — both sections must exist (empty-but-present beats absent)
  advisory checkout.api [linter.observability] pack `svc.contracts` activated for `checkout.api` (vocabulary match: Interfaces) — empty checked-set (no violations to report) — draft status: findings are advisory-first and name it
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

There is the activation advisory, draft suffix and all: `— draft
status: findings are advisory-first and name it` is the pack's
lifecycle speaking. The match list names every vocabulary token that
triggered it — so far just the section name. The finding on the
findings channel and the advisory on the warnings channel are
independent: the draft status cannot change the exit code, ever.

## Layer 3 — Constraints, Model, Properties

The rest of the file is the ordinary four-layer shape — the pack adds
vocabulary, it does not change the layers. The two *contract rows* are
base-format rows: a request contract is an `extension_point` (consumers
`satisfies` it, from their own files), the event is an `effect`
(consumers `observes` it). The pack's `published_as` field rides as a
column on those rows, tying each contract to the `## Interfaces` row
that registers it. Two invariants state the registry's claims, a
failure `effect` labels the breach output, and the model is a
four-state publish lifecycle whose failure terminal emits:

```markdown
## Constraints

| id                 | kind            | expr                                           | traces_to        | observes                       | published_as |
|--------------------|-----------------|------------------------------------------------|------------------|--------------------------------|--------------|
| cart_contract      | extension_point | `a consumer conforms to the cart request contract` | [[checkout.api]] |                              | cart_api     |
| contracts_closed   | invariant       | `every published_as name on a contract row resolves to a ## Interfaces row declared in this file` | [[checkout.api]] |                              |              |
| order_placed_event | effect          | `checkout.api.order_placed_event(payload)`     | [[checkout.api]] |                                | order_placed |
| compat_breach      | effect          | `checkout.api.compat_breach(detail)`           | [[checkout.api]] |                                |              |
| breach_surfaced    | invariant       | `every compatibility breach reaches the consumer team with the interface and change case named` | [[checkout.api]] | [[checkout.api.compat_breach]] |              |
| event_on_bus       | invariant       | `every order event reaches the message bus topic its binding declares` | [[checkout.api]] | [[checkout.api.order_placed_event]] |          |

## Model

### States

- `registered`
- `checked`
- `published`
- `failed` (emits: `[[checkout.api.compat_breach]]`)

### Transitions

| id    | from       | to        | guard                             |
|-------|------------|-----------|-----------------------------------|
| check | registered | checked   | `[[checkout.api.contracts_closed]]`  |
| ship  | checked    | published | `all published_as bindings written` |
| fail  | registered | failed    | `¬[[checkout.api.contracts_closed]]` |

## Properties

| id             | kind       | derives_from                        | generator                        | predicate                                                                                                                                                                                                  |
|----------------|------------|-------------------------------------|----------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| order_emitted  | unit       | [[checkout.api.order_placed_event]] | `checkout_runs_with_orders()`    | `every order publication carries the order_placed payload`                                                                                                                                                 |
| bus_written    | unit       | [[checkout.api.event_on_bus]]       | `checkout_runs_with_orders()`    | `every order event is written to the topic its binding declares`                                                                                                                                           |
| contracts_resolve | unit    | [[checkout.api.contracts_closed]]   | `contract_rows_with_published_as()` | `every published_as name resolves to a declared ## Interfaces row`                                                                                                                                      |
| breach_emitted | unit       | [[checkout.api.compat_breach]]      | `contract_changes_with_breaches()` | `the failure carries the checkout.api.compat_breach label with detail`                                                                                                                                   |
| breach_named   | unit       | [[checkout.api.breach_surfaced]]    | `contract_changes_with_breaches()` | `the consumer-team message names the interface and the change case`                                                                                                                                      |
| cart_compat    | svc.compat | [[checkout.api.cart_contract]]      | `cart_contracts_with_changes()`  | `every cart contract change keeps existing consumers working in every declared case — **added:** optional fields and new endpoints keep existing clients valid; **removed:** a removal never ships inside an additive binding — it ships as a new versioned binding` |
```

One pack-specific row: `cart_compat` is a law whose kind is the fiber
kind `svc.compat` — the Properties layer paying the pack's floor, its
predicate enumerating **`added:`** and **`removed:`** as named cases
(the two directions a published contract can break in). Everything
else — `traces_to`, guards, `emits`, coverage, the `observes` edges
that keep every effect somebody's watched output — is core-format
machinery, taught properly in the [generic worked
example](worked-example.md). And note what the Constraints table
*cannot* grow: a `consumers` column. The pack declares no
provider-to-consumer field — that direction is the graph's job
(`satisfies` from the consumer's file, `observes` likewise), never the
publisher's authored list.

## First full lint — a dangling reference, then the fix

The author's first full draft carries one typo: `cart_contract`'s
`traces_to` cell says `[[checkout.apt]]` — misremembered, the same
classic as the reservation example. Lint the file:

```sh
spk lint checkout-api.md
```

```text
lint: 1 file(s) linted, 2 finding(s), 1 advisory warning(s)
  checkout-api.md [linter.total_refs] dangling reference `[[checkout.apt]]` from constraints.traces_to — target not defined in any spec file
  checkout.api [linter.single_root_reachable] 2 row(s) unreachable from any intent row — an orphaned island (traces_to/derives_from/guard/from-to/emits): checkout.api.cart_compat, checkout.api.cart_contract
  advisory checkout.api [linter.observability] pack `svc.contracts` activated for `checkout.api` (vocabulary match: svc.compat, Interfaces, published_as) — empty checked-set (no violations to report) — draft status: findings are advisory-first and name it
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

Exit 1 — findings, never a silent pass. Three things to read here:

1. **One typo, two labeled findings.** The dangling `traces_to` is the
   direct hit; `linter.single_root_reachable` is the cascade — with the
   edge broken, `cart_contract` and the law that derives from it
   (`cart_compat`) become an island no intent can reach. Fixing the
   typo clears both.
2. **The finding and the advisory ride different channels.** The pack
   activated (its vocabulary is all over the file — the match list
   names all three tokens now: the kind, the section, the field), but
   activation lives on the *warnings* channel and never gates, draft
   status or not. The dangling reference lives on the *findings*
   channel and does.
3. **`empty checked-set (no violations to report)`** — the pack's
   declared `svc.interface_closed` checker ran and honestly reports an
   empty violation set. More on what that does and does not mean below.

Fix the typo — `[[checkout.apt]]` → `[[checkout.api]]` — and lint
again:

```text
lint: 1 file(s) linted, 0 finding(s), 1 advisory warning(s)
  advisory checkout.api [linter.observability] pack `svc.contracts` activated for `checkout.api` (vocabulary match: svc.compat, Interfaces, published_as) — empty checked-set (no violations to report) — draft status: findings are advisory-first and name it
→ Run: specodelic graph
```

Zero findings, exit 0 — and now the advisory is the *whole* output.
Read it as the meter it is: an exit-0 lint alone proves nothing (a spec
using none of the pack vocabulary also exits 0, silently); **the
activation line naming `svc.contracts` is the observable** that the
pack was discovered, matched, and engaged. The pipeline's next stage
runs pack-aware too:

```sh
spk graph checkout-api.md
```

```text
graph: 1 file(s), 20 node(s), 23 edge(s), 0 dangling, 0 typing violation(s), 0 supersedes cycle(s), 1 external boundary(ies)
  external boundary: checkout.api
→ Run: specodelic lint
```

The `1 external boundary` is not a defect — it is this pack's whole
point, derived for free. A file hosting an `extension_point` row *is*
an external boundary: the checkout team's spec is the seam between
their repo and every consumer's repo, and the graph says so without the
publisher ever listing a consumer.

## What a missing pack looks like

The demo depends on the pack file being *discovered*. Lint the same
finished spec in a **fresh git workspace** with no `kind: profile` file
in it — not a subdirectory of the first workspace, whose repo toplevel
the discovery scan still anchors to — and the machinery turns from
advisory into findings:

```text
lint: 1 file(s) linted, 2 finding(s), 0 advisory warning(s)
  checkout.api [linter.property_kind_closed] property `cart_compat` has kind `svc.compat` — outside the closed set {unit, law} (kinds.md property_row_shape)
  checkout.api [linter.orphan_vocabulary] orphan vocabulary: pack-qualified token(s) `svc.compat` used by `checkout.api`, but no `kind: profile` pack in namespace `svc` is discovered — candidate pack `svc.*` (prefix-derived: only the namespace is known); remediations: add/enable a `kind: profile` pack file declaring the vocabulary, or fix the vocabulary (un-qualify or retype the token)
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

`linter.orphan_vocabulary` — the finding always names the candidate
(namespace-derived, `svc.*` here) and states **both** remediations:
enable a pack declaring the vocabulary, or fix the token. Copy the pack
file back into the workspace and the pair clears — with the pack
active, the walkers accept a fiber kind in a kind column
(base ∪ active-pack-fiber), so that mis-typing only surfaces when the
pack is missing. The reliable diagnostic is the one this section
started with: a spec you *meant* to be pack-enabled but lint with no
activation advisory is a spec whose pack was not discovered.

For the other direction — proof the guarantee still runs *toward*
silence — strip every pack-vocabulary token from the same file (drop
the `## Interfaces` section, the `published_as` column and the prose
mentions of it, and the `svc.compat` law row), then lint in that same
pack-less workspace:

```text
lint: 1 file(s) linted, 0 finding(s), 0 advisory warning(s)
→ Run: specodelic graph
```

No advisory, no findings — the byte-identical guarantee in action. What
is left is an ordinary base-format spec, and a pack can never gate a
file that did not opt in.

## What the linter does not catch (yet)

Honesty check — the failures this page has *not* shown are the ones you
might expect first. Change `cart_api` to `ghost_row` in the
`published_as` column of the finished spec and lint:

```text
lint: 1 file(s) linted, 0 finding(s), 1 advisory warning(s)
  advisory checkout.api [linter.observability] pack `svc.contracts` activated for `checkout.api` (vocabulary match: svc.compat, Interfaces, published_as) — empty checked-set (no violations to report) — draft status: findings are advisory-first and name it
→ Run: specodelic graph
```

Byte-for-byte the clean output — exit 0, activation advisory, nothing
else. That is not the linter missing the dangle by accident:
`svc.interface_closed` ships as a **declaration**. In v1, pack checkers
are honest-empty — activation and the empty checked-set are reported
truthfully, violation *evaluation* is not implemented yet (the
[guide](../using-packs.md) documents this, and
[verification boundaries](../verification-boundaries.md) places it on
the guarantee ladder). The same applies to the floor: drop the
**`added:`** and **`removed:`** case labels from `cart_compat`'s
predicate and the lint output is identical — `svc.compat_cases` is also
named, not executed.

The format's answer to the gap is the one this spec already embodies:
the dangling-edge claim lives where the evidence plan lives — the
`contracts_closed` invariant, its `contracts_resolve` property, and
eventually a real executable predicate. When the checkers gain
evaluation, the declarations are already in place; the spec does not
change. Until then, remember what draft means here twice over: the
pack's *findings* are advisory-first, *and* its checkers are
declarations — this page is a preview of a mechanism still being
filled in, not a compatibility guarantee you can test today.

## The finished spec

Every block above assembles into this one file, `checkout-api.md` —
exactly as it lints at 0 findings, 1 activation advisory:

```markdown
---
id: checkout.api
kind: intent
statement: "WHEN the checkout team publishes or changes a service contract THE system SHALL register every published contract with its owning repo, its authoritative external binding, its declared evolution mode, and the consumer compatibility law each change must satisfy."
---

# Checkout service contracts

A contract registry for one checkout team: a cart request contract, an
order-placed event, and an order-summary schema — each registered with
the repo that owns it and the authoritative external artifact it binds
to.

## Interfaces

| name          | kind    | repo              | binding                               | evolution |
|---------------|---------|-------------------|---------------------------------------|-----------|
| cart_api      | request | acme/checkout-api | openapi://specs/cart-v2.yaml          | additive  |
| order_placed  | event   | acme/checkout-api | asyncapi://specs/order-placed-v1.yaml | versioned |
| order_summary | schema  | acme/checkout-web | json-schema://order-summary-v3.json   | frozen    |

## Constraints

| id                 | kind            | expr                                           | traces_to        | observes                       | published_as |
|--------------------|-----------------|------------------------------------------------|------------------|--------------------------------|--------------|
| cart_contract      | extension_point | `a consumer conforms to the cart request contract` | [[checkout.api]] |                              | cart_api     |
| contracts_closed   | invariant       | `every published_as name on a contract row resolves to a ## Interfaces row declared in this file` | [[checkout.api]] |                              |              |
| order_placed_event | effect          | `checkout.api.order_placed_event(payload)`     | [[checkout.api]] |                                | order_placed |
| compat_breach      | effect          | `checkout.api.compat_breach(detail)`           | [[checkout.api]] |                                |              |
| breach_surfaced    | invariant       | `every compatibility breach reaches the consumer team with the interface and change case named` | [[checkout.api]] | [[checkout.api.compat_breach]] |              |
| event_on_bus       | invariant       | `every order event reaches the message bus topic its binding declares` | [[checkout.api]] | [[checkout.api.order_placed_event]] |          |

## Model

### States

- `registered`
- `checked`
- `published`
- `failed` (emits: `[[checkout.api.compat_breach]]`)

### Transitions

| id    | from       | to        | guard                             |
|-------|------------|-----------|-----------------------------------|
| check | registered | checked   | `[[checkout.api.contracts_closed]]`  |
| ship  | checked    | published | `all published_as bindings written` |
| fail  | registered | failed    | `¬[[checkout.api.contracts_closed]]` |

## Properties

| id             | kind       | derives_from                        | generator                        | predicate                                                                                                                                                                                                  |
|----------------|------------|-------------------------------------|----------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| order_emitted  | unit       | [[checkout.api.order_placed_event]] | `checkout_runs_with_orders()`    | `every order publication carries the order_placed payload`                                                                                                                                                 |
| bus_written    | unit       | [[checkout.api.event_on_bus]]       | `checkout_runs_with_orders()`    | `every order event is written to the topic its binding declares`                                                                                                                                           |
| contracts_resolve | unit    | [[checkout.api.contracts_closed]]   | `contract_rows_with_published_as()` | `every published_as name resolves to a declared ## Interfaces row`                                                                                                                                      |
| breach_emitted | unit       | [[checkout.api.compat_breach]]      | `contract_changes_with_breaches()` | `the failure carries the checkout.api.compat_breach label with detail`                                                                                                                                   |
| breach_named   | unit       | [[checkout.api.breach_surfaced]]    | `contract_changes_with_breaches()` | `the consumer-team message names the interface and the change case`                                                                                                                                      |
| cart_compat    | svc.compat | [[checkout.api.cart_contract]]      | `cart_contracts_with_changes()`  | `every cart contract change keeps existing consumers working in every declared case — **added:** optional fields and new endpoints keep existing clients valid; **removed:** a removal never ships inside an additive binding — it ships as a new versioned binding` |
```

To reproduce every block on this page: copy
`packs/svc-contracts.md` from the repo into a fresh git workspace, save
the file above as `checkout-api.md`, and run
`spk lint checkout-api.md`.

## Where to go next

- The mechanism, end to end: [using packs — the practical
  guide](../using-packs.md) — discovery, activation, lifecycle and
  draft-status semantics, honest-empty checkers, the full CLI loop.
- The pack's normative home: [packs — domain-pack
  mechanism](../specs/packs/spec.md), and the pack definition itself —
  [`packs/svc-contracts.md`](https://github.com/charly-vibes/specodelic/tree/main/packs/svc-contracts.md)
  is the source of truth for the vocabulary this page used.
- The capability spec behind the pack:
  [svc-contracts-pack](../specs/svc-contracts-pack/spec.md) — including
  the design decision that keeps `published_as` an intra-file outbound
  leaf.
- The published sibling packs, for the stable versions of the same
  journey: [the data.lineage pack](data-lineage.md) (registry rows for
  datasets) or [the quant.finance pack](quant-finance.md) (registry
  rows for limits) — or the journey without packs at all: [worked
  example — reservation/order](worked-example.md).
- Rule semantics offline: `spk explain packs` and
  `spk explain lint-rules`.
