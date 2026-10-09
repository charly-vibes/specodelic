# Worked example — a reservation/order spec, failures included

This page builds one small spec from nothing: a reservation service for a
web shop. You will see the whole four-layer journey — **Intent →
Constraints → Model → Properties** — and then the part most tutorials
skip: the mistakes. Every lint finding below is real output from
`specodelic 0.7.0` on the file as it stood at that moment; the fixes are
the fixes that were actually made. The finished spec lives at
[`examples/reservation-order.md`](reservation-order.md) and lints clean —
you can check that yourself with `spk lint docs/src/examples/reservation-order.md`
from the repo root.

The machine truth behind every rule quoted here is the tool itself:
`spk explain format`, `spk explain ears`, `spk explain lint-rules` serve
the distilled format guide from inside the binary. This page teaches the
format through one example; the commands appear only where they earn
their place.

On a TTY `spk` prints human-readable text (shown throughout); piped, it
emits a JSON envelope instead.

## The story

A customer submits an order for several items. The service holds stock
for every line item and accepts the order — or rejects it outright,
holding nothing, charging nothing. Accepted orders are charged exactly
once, fulfilled, or released by cancellation or expiry. Deliberately out
of scope: pricing, the payment gateway itself, and multi-order races for
the last unit (see [What the spec does not decide](#unresolved-behaviors)).

One capability, one file. The file is named `reservation-order.md`;
the format's naming law says `-` maps to the namespace dot, so the
frontmatter id is `reservation.order` (`_` would be literal).

## Layer 1 — Intent

The frontmatter carries the whole intent as one EARS sentence; the
opening prose is free-form and is never read by the linter.

```markdown
---
id: reservation.order
kind: intent
statement: "WHEN a customer submits an order THE reservation service SHALL hold stock for every line item and accept the order, or reject it with nothing held and nothing charged."
---

# Reservation & Order

A small order pipeline: submitting an order holds stock for every line
item or rejects the order outright; accepted orders are charged exactly
once, fulfilled, or released by cancellation or expiry.
```

The statement is Event-Driven (`WHEN <trigger> THE <system> SHALL
<response>`) — one of the five EARS patterns; a statement with no `SHALL`
or no pattern match fails `linter.ears_syntax`. `spk explain ears` has
the full list.

Now lint the file as it stands:

```sh
spk lint reservation-order.md
```

```text
lint: 1 file(s) linted, 1 finding(s), 0 advisory warning(s)
  reservation-order.md [linter.model_present] Model section incomplete: 0 states, 0 transitions — both sections must exist (empty-but-present beats absent)
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

`linter.model_present` fires because the Model section does not exist
yet — a spec with an intent but no model is incomplete by construction,
and the checker says so from the first pass. Keep the finding in mind;
the next two layers exist to resolve it.

## Layer 2 — Constraints

The Constraints table holds the invariants the design actually rests on.
Each row is `id | kind | expr | traces_to`; `traces_to` is a **typed
wiki-link** pointing at this file's own intent row — bare text does not
resolve. Here is the first draft, warts included: the author wrote the
last constraint's `traces_to` against a misremembered id
(`reservation.orders`, plural).

```markdown
| id                             | kind      | expr                                                                                                                | traces_to           |
|--------------------------------|-----------|----------------------------------------------------------------------------------------------------------------------|---------------------|
| all_or_nothing                 | invariant | `the order is accepted only if every line item's requested quantity is available; otherwise no unit is held and nothing is charged` | [[reservation.orders]] |
| held_or_converted              | invariant | `every unit held for an accepted order is either converted to a fulfillment or released; no held unit outlives both` | [[reservation.order]] |
| release_is_total               | invariant | `when a hold is released, no unit remains held for that order; releasing an already-released hold changes nothing`   | [[reservation.order]] |
| charge_iff_fulfilled           | invariant | `the customer is charged if and only if the order reaches fulfilled`                                                  | [[reservation.order]] |
| cancel_refunds_exactly_charged | invariant | `IF a charged order is cancelled, THEN the refund equals exactly the amount charged — never more, never less`         | [[reservation.order]] |
| expiry_releases_holds          | invariant | `an unfulfilled reservation expires after its TTL; expiry releases the hold without charging the customer`            | [[reservation.order]] |
```

```sh
spk lint reservation-order.md
```

```text
lint: 1 file(s) linted, 9 finding(s), 0 advisory warning(s)
  reservation-order.md [linter.model_present] Model section incomplete: 0 states, 0 transitions — both sections must exist (empty-but-present beats absent)
  reservation-order.md [linter.total_refs] dangling reference `[[reservation.orders]]` from constraints.traces_to — target not defined in any spec file
  reservation.order [linter.coverage] constraint `all_or_nothing` has no deriving property — ∃ property.derives_from == `reservation.order.all_or_nothing` is required — write `[[reservation.order.all_or_nothing]]` in the property's derives_from cell
  reservation.order [linter.coverage] constraint `cancel_refunds_exactly_charged` has no deriving property — ∃ property.derives_from == `reservation.order.cancel_refunds_exactly_charged` is required — write `[[reservation.order.cancel_refunds_exactly_charged]]` in the property's derives_from cell
  reservation.order [linter.coverage] constraint `charge_iff_fulfilled` has no deriving property — ∃ property.derives_from == `reservation.order.charge_iff_fulfilled` is required — write `[[reservation.order.charge_iff_fulfilled]]` in the property's derives_from cell
  reservation.order [linter.coverage] constraint `expiry_releases_holds` has no deriving property — ∃ property.derives_from == `reservation.order.expiry_releases_holds` is required — write `[[reservation.order.expiry_releases_holds]]` in the property's derives_from cell
  reservation.order [linter.coverage] constraint `held_or_converted` has no deriving property — ∃ property.derives_from == `reservation.order.held_or_converted` is required — write `[[reservation.order.held_or_converted]]` in the property's derives_from cell
  reservation.order [linter.coverage] constraint `release_is_total` has no deriving property — ∃ property.derives_from == `reservation.order.release_is_total` is required — write `[[reservation.order.release_is_total]]` in the property's derives_from cell
  reservation.order [linter.single_root_reachable] 1 row(s) unreachable from any intent row — an orphaned island (traces_to/derives_from/guard/from-to/emits): reservation.order.all_or_nothing
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

Stop and read what one typo bought. **Three different rules fired on one
wrong cell**, and they say different things:

1. `linter.total_refs` — the reference dangles: nothing defines
   `reservation.orders`. The target id is `reservation.order`.
2. `linter.coverage` — every constraint must have a deriving property
   (the fourth layer's job; six of these findings stay until then).
3. `linter.single_root_reachable` — `all_or_nothing` is now an
   *orphaned island*: with its only link broken, it cannot reach the
   file's own intent row, and neither can anything hanging off it.

That last one is the format's structural conscience. Fix the typo:

```markdown
| all_or_nothing                 | invariant | `the order is accepted only if every line item's requested quantity is available; otherwise no unit is held and nothing is charged` | [[reservation.order]] |
```

and the dangling-reference and orphaned-island findings clear — one
cell, two findings. (The coverage finding for `all_or_nothing` itself
stays: it fires because nothing derives from the constraint yet, not
because of the typo — all six coverage rows stand until the next layer.)
What remains is `linter.model_present` plus the six coverage rows; those
are next — the count dropped from 9 to 7, all of it the typo's:

```text
lint: 1 file(s) linted, 7 finding(s), 0 advisory warning(s)
  reservation-order.md [linter.model_present] Model section incomplete: 0 states, 0 transitions — both sections must exist (empty-but-present beats absent)
  reservation.order [linter.coverage] constraint `all_or_nothing` has no deriving property — ∃ property.derives_from == `reservation.order.all_or_nothing` is required — write `[[reservation.order.all_or_nothing]]` in the property's derives_from cell
  reservation.order [linter.coverage] constraint `cancel_refunds_exactly_charged` has no deriving property — ∃ property.derives_from == `reservation.order.cancel_refunds_exactly_charged` is required — write `[[reservation.order.cancel_refunds_exactly_charged]]` in the property's derives_from cell
  reservation.order [linter.coverage] constraint `charge_iff_fulfilled` has no deriving property — ∃ property.derives_from == `reservation.order.charge_iff_fulfilled` is required — write `[[reservation.order.charge_iff_fulfilled]]` in the property's derives_from cell
  reservation.order [linter.coverage] constraint `expiry_releases_holds` has no deriving property — ∃ property.derives_from == `reservation.order.expiry_releases_holds` is required — write `[[reservation.order.expiry_releases_holds]]` in the property's derives_from cell
  reservation.order [linter.coverage] constraint `held_or_converted` has no deriving property — ∃ property.derives_from == `reservation.order.held_or_converted` is required — write `[[reservation.order.held_or_converted]]` in the property's derives_from cell
  reservation.order [linter.coverage] constraint `release_is_total` has no deriving property — ∃ property.derives_from == `reservation.order.release_is_total` is required — write `[[reservation.order.release_is_total]]` in the property's derives_from cell
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

## Layer 3 — Model

The Model section is a finite state machine: a `### States` list and a
`### Transitions` table (`id | from | to | guard`). The first draft most
people write is wrong in instructive ways. This draft has a missing
guard, a misspelled state name, a state declared but never used — and it
draws the happy path while leaving the failure path mute:

```markdown
### States

- `submitted`
- `reserved`
- `charged`
- `fulfilled`
- `cancelled`
- `released`
- `expired`
- `reserve_failed`

### Transitions

| id            | from      | to             | guard                                                                                              |
|---------------|-----------|----------------|-----------------------------------------------------------------------------------------------------|
| accept        | submitted | reserved       | [[reservation.order.all_or_nothing]]                                                                 |
| reject_order  | submitted | reserve_failed | stock unavailable at acceptance time                                                                 |
| charge        | reserved  | charged        |                                                                                                      |
| fulfill       | charged   | shipped        | payment settled                                                                                      |
| cancel_paid   | charged   | cancelled      | `customer cancelled ∧ ¬payment settled ∧ [[reservation.order.cancel_refunds_exactly_charged]]`        |
| cancel_unpaid | reserved  | cancelled      | `customer cancelled ∧ [[reservation.order.held_or_converted]]`                                       |
| expire        | reserved  | released       | `reservation TTL elapsed ∧ [[reservation.order.expiry_releases_holds]]`                              |
```

```sh
spk lint reservation-order.md
```

```text
lint: 1 file(s) linted, 13 finding(s), 0 advisory warning(s)
  reservation-order.md [linter.guard_required] transition `charge` has an empty guard — every guard must be non-null
  reservation-order.md [linter.every_state_used] state `fulfilled` is declared but no transition enters or leaves it — every state must appear as from or to in at least one transition
  reservation-order.md [linter.every_state_used] state `expired` is declared but no transition enters or leaves it — every state must appear as from or to in at least one transition
  reservation-order.md [linter.every_transition_valid] transition `fulfill` has to `shipped` — not a declared state (states: ["cancelled", "charged", "expired", "fulfilled", "released", "reserve_failed", "reserved", "submitted"])
  reservation-order.md [linter.terminal_states_emit] failure terminal `reserve_failed` emits nothing — a mute failure terminal is a finding; add an emits edge to a file-owned effect Constraint: `- reserve_failed (emits: [[reservation.order.<label>]])`
  reservation-order.md [linter.guard_negation_total] failure transition `reject_order` cites nothing and is not on the carve-out list (orchestrate's stage-fails) — cite the union of its success siblings' citation sets, or record a carve-out in linter-failure_shape.md
  reservation.order [linter.coverage] constraint `all_or_nothing` has no deriving property — ∃ property.derives_from == `reservation.order.all_or_nothing` is required — write `[[reservation.order.all_or_nothing]]` in the property's derives_from cell
  reservation.order [linter.coverage] constraint `cancel_refunds_exactly_charged` has no deriving property — ∃ property.derives_from == `reservation.order.cancel_refunds_exactly_charged` is required — write `[[reservation.order.cancel_refunds_exactly_charged]]` in the property's derives_from cell
  reservation.order [linter.coverage] constraint `charge_iff_fulfilled` has no deriving property — ∃ property.derives_from == `reservation.order.charge_iff_fulfilled` is required — write `[[reservation.order.charge_iff_fulfilled]]` in the property's derives_from cell
  reservation.order [linter.coverage] constraint `expiry_releases_holds` has no deriving property — ∃ property.derives_from == `reservation.order.expiry_releases_holds` is required — write `[[reservation.order.expiry_releases_holds]]` in the property's derives_from cell
  reservation.order [linter.coverage] constraint `held_or_converted` has no deriving property — ∃ property.derives_from == `reservation.order.held_or_converted` is required — write `[[reservation.order.held_or_converted]]` in the property's derives_from cell
  reservation.order [linter.coverage] constraint `release_is_total` has no deriving property — ∃ property.derives_from == `reservation.order.release_is_total` is required — write `[[reservation.order.release_is_total]]` in the property's derives_from cell
  reservation.order [linter.single_root_reachable] 2 row(s) unreachable from any intent row — an orphaned island (traces_to/derives_from/guard/from-to/emits): reservation.order.expired, reservation.order.fulfilled
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

Thirteen findings, four distinct lessons:

**Shape mistakes.** `charge` has an empty guard
(`linter.guard_required` — a transition without a guard is not a
decision, it is an accident). `fulfill` points at `shipped`, which is
not declared (`linter.every_transition_valid`). And `expired` was
declared but never wired into any transition
(`linter.every_state_used`).

**A typo cascades.** Notice that `fulfilled` — a state the author fully
intended to reach — is *also* reported unused. That is the `shipped`
typo's doing: with `fulfill` pointing elsewhere, nothing enters
`fulfilled`, so the state became an unreachable island, dragging itself
and `expired` into one `linter.single_root_reachable` finding. Fixing
the typo clears both.

**The failure path is not optional.** The two most interesting findings
name the failure path:

- `linter.terminal_states_emit`: `reserve_failed` is a terminal state
  with inbound transitions and nothing to say — a *mute failure
  terminal*. In this format a failure may not be silent: it must emit
  exactly one labeled effect Constraint owned by this file.
- `linter.guard_negation_total`: `reject_order`'s guard cites nothing.
  Its success sibling (`accept`) cites
  `[[reservation.order.all_or_nothing]]`, so the failure transition must
  cite exactly the negated disjunction of that citation set. The rule
  makes "rejection happens for the same reason acceptance does not" a
  machine-checked shape, not a hope.

Fixing all of this touches four places: delete the redundant `expired`
state (TTL expiry releases the hold — `released` is that state), give
`charge` a guard, point `fulfill` at `fulfilled`, and give the failure
terminal something to emit. Emitting needs a new **effect** row — the
fourth constraint kind the file meets for the first time:

```markdown
| stock_exhausted                | effect    | `reservation.order.stock_exhausted(detail)`                                                                           | [[reservation.order]] |
```

and the states list gains the emit edge:

```markdown
- `reserve_failed` (emits: `[[reservation.order.stock_exhausted]]`)
```

while the transitions become:

```markdown
| id            | from      | to             | guard                                                                                              |
|---------------|-----------|----------------|-----------------------------------------------------------------------------------------------------|
| accept        | submitted | reserved       | [[reservation.order.all_or_nothing]]                                                                 |
| reject_order  | submitted | reserve_failed | `¬[[reservation.order.all_or_nothing]]`                                                              |
| charge        | reserved  | charged        | `payment authorized ∧ [[reservation.order.charge_iff_fulfilled]]`                                    |
| fulfill       | charged   | fulfilled      | `payment settled ∧ [[reservation.order.charge_iff_fulfilled]]`                                       |
| cancel_paid   | charged   | cancelled      | `customer cancelled ∧ ¬payment settled ∧ [[reservation.order.cancel_refunds_exactly_charged]]`        |
| cancel_unpaid | reserved  | cancelled      | `customer cancelled ∧ [[reservation.order.held_or_converted]]`                                       |
| expire        | reserved  | released       | `reservation TTL elapsed ∧ [[reservation.order.expiry_releases_holds]]`                              |
```

```sh
spk lint reservation-order.md
```

```text
lint: 1 file(s) linted, 7 finding(s), 1 advisory warning(s)
  reservation.order [linter.coverage] constraint `all_or_nothing` has no deriving property — ∃ property.derives_from == `reservation.order.all_or_nothing` is required — write `[[reservation.order.all_or_nothing]]` in the property's derives_from cell
  reservation.order [linter.coverage] constraint `cancel_refunds_exactly_charged` has no deriving property — ∃ property.derives_from == `reservation.order.cancel_refunds_exactly_charged` is required — write `[[reservation.order.cancel_refunds_exactly_charged]]` in the property's derives_from cell
  reservation.order [linter.coverage] constraint `charge_iff_fulfilled` has no deriving property — ∃ property.derives_from == `reservation.order.charge_iff_fulfilled` is required — write `[[reservation.order.charge_iff_fulfilled]]` in the property's derives_from cell
  reservation.order [linter.coverage] constraint `expiry_releases_holds` has no deriving property — ∃ property.derives_from == `reservation.order.expiry_releases_holds` is required — write `[[reservation.order.expiry_releases_holds]]` in the property's derives_from cell
  reservation.order [linter.coverage] constraint `held_or_converted` has no deriving property — ∃ property.derives_from == `reservation.order.held_or_converted` is required — write `[[reservation.order.held_or_converted]]` in the property's derives_from cell
  reservation.order [linter.coverage] constraint `release_is_total` has no deriving property — ∃ property.derives_from == `reservation.order.release_is_total` is required — write `[[reservation.order.release_is_total]]` in the property's derives_from cell
  reservation.order [linter.coverage] constraint `stock_exhausted` has no deriving property — ∃ property.derives_from == `reservation.order.stock_exhausted` is required — write `[[reservation.order.stock_exhausted]]` in the property's derives_from cell
  advisory reservation.order [linter.observability] effect `reservation.order.stock_exhausted` has no `observes` reference targeting it — a declared output nobody observes (advisory, exit 0) [hint: add an `observes` column on the row that consumes this output, or remove the effect if it is unintentional]
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

The model shape is clean now; what remains is coverage (next layer) —
plus one **advisory warning**: the new effect is a firehose nobody
attached a hose to. The warning asks, in the linter's own words, "who is
this output for?" It never gates (exit 0) — but a rejection reason that
is part of the contract deserves an observer, so the finished spec adds
one. An observer is a second column on the row that consumes the output:

```markdown
| reject_reason_surfaced         | invariant | `every rejection reaches the caller with the exhausted line items named`                                              | [[reservation.order]] | [[reservation.order.stock_exhausted]] |
```

## Layer 4 — Properties

Every constraint — including the `effect` row — needs a deriving
property. The Properties table is the evidence plan: `unit` rows pair a
generator with a predicate; `law` rows would additionally require
`**identity:**` and `**associativity:**` case labels (`linter.law_cases`)
— none of this file's claims are algebraic, so it needs none.

```markdown
## Properties

| id                        | kind | derives_from                                    | generator                                        | predicate                                                        |
|---------------------------|------|--------------------------------------------------|---------------------------------------------------|--------------------------------------------------------------------|
| no_charge_on_rejection    | unit | [[reservation.order.all_or_nothing]]             | `orders_with_mixed_availability()`                | `nothing is charged for any rejected order`                        |
| holds_all_converted       | unit | [[reservation.order.held_or_converted]]          | `arbitrary_closed_orders()`                       | `every held unit is converted or released when the order closes`   |
| double_release_noop       | unit | [[reservation.order.release_is_total]]           | `order_released_twice()`                          | `the second release changes nothing`                               |
| fulfilled_implies_charged | unit | [[reservation.order.charge_iff_fulfilled]]       | `arbitrary_accepted_orders()`                     | `charged(order) == (outcome(order) == fulfilled)`                  |
| refund_bounded            | unit | [[reservation.order.cancel_refunds_exactly_charged]] | `charged_orders_cancelled_before_settlement()` | `refund(order) == amount_charged(order)`                           |
| expiry_free               | unit | [[reservation.order.expiry_releases_holds]]      | `orders_past_ttl()`                               | `nothing is charged and no unit remains held after expiry`         |
| rejection_labels_emitted  | unit | [[reservation.order.reject_reason_surfaced]]     | `orders_with_exhausted_items()`                   | `the rejection message names exactly the exhausted line items`     |
| stock_exhausted_emitted   | unit | [[reservation.order.stock_exhausted]]            | `rejections_over_sold_out_items()`                | `the rejection carries the stock_exhausted label with the item detail` |
```

With the observer row and the property table in place:

```sh
spk lint reservation-order.md
```

```text
lint: 1 file(s) linted, 0 finding(s), 0 advisory warning(s)
→ Run: specodelic graph
```

Zero findings, zero warnings. The spec is structurally complete. Now
walk the rest of the pipeline (the exact commands and their shape are in
[Installation & Quick Start](../installation.md)); the essentials:

```sh
spk graph reservation-order.md
```

```text
graph: 1 file(s), 31 node(s), 39 edge(s), 0 dangling, 0 typing violation(s), 0 supersedes cycle(s), 0 external boundary(ies)
→ Run: specodelic lint
```

```sh
spk compile reservation-order.md --out-dir specodelic/
```

```text
compile: 1 compiled, 0 failed
  reservation-order.md [reservation.order] → specodelic/reservation-order.toml, specodelic/reservation-order_props.rs, specodelic/reservation-order.tla
→ Run: specodelic verify (consumes the *_props.rs artifacts)
```

Compile emits three artifacts byte-stably: the constraints as TOML, the
properties as proptest scaffolding, the model as a TLA+ module.

```sh
spk model-check reservation-order.md
```

```text
model-check: 1 checked, 0 failed
  reservation-order.md [reservation.order]: exploration_only (7 states explored) → specodelic/reservation-order.check.json
    claims: 0 evaluated, 7 unchecked, 0 blocking
      unchecked: all_or_nothing, cancel_refunds_exactly_charged, charge_iff_fulfilled, expiry_releases_holds, held_or_converted, reject_reason_surfaced, release_is_total
→ Run: specodelic verify (consumes the *.check.json reports)
```

Read that verdict carefully: **`exploration_only` is not a clean bill of
health.** All seven states were explored and no executable invariant was
violated — but none of the seven claims was *evaluated*, because
prose-predicate claims cannot be executed. The tool refuses to dress an
unexplored claim as a verified one.

## A counterexample — the checker catches the author

That honesty has a payoff. Suppose the author, wanting a stronger
verdict, hardens one claim into an executable invariant — and while at
it, over-claims. Working on a scratch copy of the finished file, they
add:

```markdown
| reserve_never_fails            | invariant | `**rust:** state != "reserve_failed"` — surely the reject path never fires once the flow is written down | [[reservation.order]] |                                       |
```

plus its coverage property:

```markdown
| reserve_never_fails_holds | unit | [[reservation.order.reserve_never_fails]]        | `arbitrary_orders()`                              | `no order ever reaches reserve_failed`                             |
```

The `**rust:**` marker makes the invariant executable: the model-check
backend evaluates it as a Rust boolean over the program counter, the
machine's current state name. Lint is happy — coverage holds, shape
holds:

```sh
cp reservation-order.md /tmp/reservation/
cd /tmp/reservation
spk lint reservation-order.md
```

```text
lint: 1 file(s) linted, 0 finding(s), 0 advisory warning(s)
→ Run: specodelic graph
```

**Lint cannot catch this mistake** — it is structural, and this file is
structurally impeccable. The claim itself is simply false, and the next
stage proves it:

```sh
spk compile reservation-order.md --out-dir specodelic/
spk model-check reservation-order.md
```

```text
compile: 1 compiled, 0 failed
  reservation-order.md [reservation.order] → specodelic/reservation-order.toml, specodelic/reservation-order_props.rs, specodelic/reservation-order.tla
→ Run: specodelic verify (consumes the *_props.rs artifacts)
model-check: 1 checked, 0 failed
  reservation-order.md [reservation.order]: counterexample_found (3 states explored) → specodelic/reservation-order.check.json
    claims: 1 evaluated, 7 unchecked, 1 blocking
      blocking: reserve_never_fails (rust, counterexample)
      unchecked: all_or_nothing, cancel_refunds_exactly_charged, charge_iff_fulfilled, expiry_releases_holds, held_or_converted, reject_reason_surfaced, release_is_total
model-check findings: the claim aggregate is not clean (see .data.checked)
→ Run: fix the refuted claim(s) named in .data.checked — verify's model gate accepts only a no_counterexample aggregate
```

`counterexample_found` — and the report names the violated invariant and
the shortest trace that refutes it (from the emitted
`reservation-order.check.json`):

```json
{
  "outcome": "counterexample_found",
  "violated_invariant_id": "reserve_never_fails",
  "trace": ["submitted", "reserve_failed"],
  "states_explored": 3,
  "invariants_checked": ["reserve_never_fails"]
}
```

The model checker explored the reachable states — `states_explored: 3`,
the two successors of `submitted` and the root — and reported the
two-step trace ending in the very state the author swore was
unreachable. Rejection is reachable *by design* —
`all_or_nothing` can fail; that is what the failure terminal is for. The
executable invariant claimed otherwise, so the invariant was the bug.

The fix is to delete the over-claiming row and its property (the true
claim — *why* rejection happens — is already carried by the transition's
guard). Re-running `spk model-check` on the restored file returns
exactly the `exploration_only` verdict shown above. The checker was the
design critic; the counterexample was the design lesson.

This is also where the format's honesty boundary shows: the program
counter model carries **no data binding** — an executable invariant sees
only the machine's current state name, never the order's contents or its
history. Claims like "no charge without fulfillment" live in the
Constraints table as prose invariants and their Properties plans; they
get executed only when someone writes a real `**rust:**` fragment in a
backend that can evaluate them.

## What verify refuses to pretend

The last stage executes the property scaffolding:

```sh
spk verify reservation-order.md
```

```text
verify: 0 verified, 1 blocked
  reservation-order.md [reservation.order]: properties_failed — 8 of 8 compiled blocks failed — a partial pass is a failure of the property
      unchecked: all_or_nothing, cancel_refunds_exactly_charged, charge_iff_fulfilled, expiry_releases_holds, held_or_converted, reject_reason_surfaced, release_is_total
verify blocked — the blocking gate, message and hint ride .data.blocked
→ Run: fix the failing predicates (see each block's detail for the shrunk failing input)
```

Eight of eight blocks fail — because every predicate in this file is
still prose. Compile emitted them as `todo_predicate!` placeholders that
compile fine and fail the moment `verify` executes them. That is the
format refusing to pretend: a claim that was never executed is never
reported as passing, and a partial pass is a failure. The next rung on
the progressive-formalization ladder is opting individual predicates
into `**rust:**` fragments — one property at a time, only where a real
assertion can be written.

## Unresolved behaviors

A finished spec is not a finished design. This file deliberately leaves
open:

- **Payment semantics.** `payment authorized` and `payment settled` are
  prose guards; the compiled TLA+ module carries them as comments and
  stutters at terminals rather than inventing semantics. The spec does
  not model the gateway.
- **The value of the TTL.** `expire` fires "after its TTL" — no duration
  is decided, and the schema has no column that could carry one.
- **Refund timing.** `cancel_refunds_exactly_charged` bounds the
  *amount*; when the refund lands is not decided.
- **Concurrent orders.** The model is one order's program counter. Two
  orders racing for the last unit is a different capability — its own
  file, its own intent — not a row bolted onto this one.
- **Partial fulfillment.** `all_or_nothing` was a design decision, not
  an accident of scope: no partial accept is modeled, deliberately.
- **Unexecuted predicates.** All eight predicates are prose. The claims
  are as strong as their evidence plan, and the evidence has not been
  written yet — see `spk verify` above.

Each of these would land in a named place if decided: a constraint row,
a property row, a transition guard, or a new file — the format refuses
to let them remain ambient.

## The finished file

The result of every fix above is
[`examples/reservation-order.md`](reservation-order.md) — the standalone
spec exactly as it lints at 0 findings, 0 warnings. For a second worked
example that goes deeper on one mechanism (how the accumulator's algebra
licenses parallel architectures), see
[batch-resume — derived parallelism, worked end to end](batch-resume.md).
