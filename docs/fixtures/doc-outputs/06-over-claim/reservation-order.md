---
id: reservation.order
kind: intent
statement: "WHEN a customer submits an order THE reservation service SHALL hold stock for every line item and accept the order, or reject it with nothing held and nothing charged."
---

# Reservation & Order

A small order pipeline: submitting an order holds stock for every line
item or rejects the order outright; accepted orders are charged exactly
once, fulfilled, or released by cancellation or expiry. The subject of
this spec is the reservation service itself — nothing here constrains or
mentions any checking tool.

## Constraints

| id                             | kind      | expr                                                                                                                | traces_to           | observes                              |
|--------------------------------|-----------|----------------------------------------------------------------------------------------------------------------------|---------------------|---------------------------------------|
| all_or_nothing                 | invariant | `the order is accepted only if every line item's requested quantity is available; otherwise no unit is held and nothing is charged` | [[reservation.order]] |                                       |
| held_or_converted              | invariant | `every unit held for an accepted order is either converted to a fulfillment or released; no held unit outlives both` | [[reservation.order]] |                                       |
| release_is_total               | invariant | `when a hold is released, no unit remains held for that order; releasing an already-released hold changes nothing`   | [[reservation.order]] |                                       |
| charge_iff_fulfilled           | invariant | `the customer is charged if and only if the order reaches fulfilled`                                                  | [[reservation.order]] |                                       |
| cancel_refunds_exactly_charged | invariant | `IF a charged order is cancelled, THEN the refund equals exactly the amount charged — never more, never less`         | [[reservation.order]] |                                       |
| expiry_releases_holds          | invariant | `an unfulfilled reservation expires after its TTL; expiry releases the hold without charging the customer`            | [[reservation.order]] |                                       |
| stock_exhausted                | effect    | `reservation.order.stock_exhausted(detail)`                                                                           | [[reservation.order]] |                                       |
| reject_reason_surfaced         | invariant | `every rejection reaches the caller with the exhausted line items named`                                              | [[reservation.order]] | [[reservation.order.stock_exhausted]] |
| reserve_never_fails            | invariant | `**rust:** state != "reserve_failed"` — surely the reject path never fires once the flow is written down | [[reservation.order]] |                                       |

## Model

### States

- `submitted`
- `reserved`
- `charged`
- `fulfilled`
- `cancelled`
- `released`
- `reserve_failed` (emits: `[[reservation.order.stock_exhausted]]`)

### Transitions

| id            | from      | to             | guard                                                                                              |
|---------------|-----------|----------------|-----------------------------------------------------------------------------------------------------|
| accept        | submitted | reserved       | [[reservation.order.all_or_nothing]]                                                                 |
| reject_order  | submitted | reserve_failed | `¬[[reservation.order.all_or_nothing]]`                                                              |
| charge        | reserved  | charged        | `payment authorized ∧ [[reservation.order.charge_iff_fulfilled]]`                                    |
| fulfill       | charged   | fulfilled      | `payment settled ∧ [[reservation.order.charge_iff_fulfilled]]`                                       |
| cancel_paid   | charged   | cancelled      | `customer cancelled ∧ ¬payment settled ∧ [[reservation.order.cancel_refunds_exactly_charged]]`        |
| cancel_unpaid | reserved  | cancelled      | `customer cancelled ∧ [[reservation.order.held_or_converted]]`                                       |
| expire        | reserved  | released       | `reservation TTL elapsed ∧ [[reservation.order.expiry_releases_holds]]`                              |

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
| reserve_never_fails_holds | unit | [[reservation.order.reserve_never_fails]]        | `arbitrary_orders()`                              | `no order ever reaches reserve_failed`                             |
