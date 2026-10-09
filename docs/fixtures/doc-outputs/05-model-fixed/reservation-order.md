---
id: reservation.order
kind: intent
statement: "WHEN a customer submits an order THE reservation service SHALL hold stock for every line item and accept the order, or reject it with nothing held and nothing charged."
---

# Reservation & Order

A small order pipeline: submitting an order holds stock for every line
item or rejects the order outright; accepted orders are charged exactly
once, fulfilled, or released by cancellation or expiry.

## Constraints

| id                             | kind      | expr                                                                                                                | traces_to           |
|--------------------------------|-----------|----------------------------------------------------------------------------------------------------------------------|---------------------|
| all_or_nothing                 | invariant | `the order is accepted only if every line items requested quantity is available; otherwise no unit is held and nothing is charged` | [[reservation.order]] |
| held_or_converted              | invariant | `every unit held for an accepted order is either converted to a fulfillment or released; no held unit outlives both` | [[reservation.order]] |
| release_is_total               | invariant | `when a hold is released, no unit remains held for that order; releasing an already-released hold changes nothing`   | [[reservation.order]] |
| charge_iff_fulfilled           | invariant | `the customer is charged if and only if the order reaches fulfilled`                                                  | [[reservation.order]] |
| cancel_refunds_exactly_charged | invariant | `IF a charged order is cancelled, THEN the refund equals exactly the amount charged — never more, never less`         | [[reservation.order]] |
| expiry_releases_holds          | invariant | `an unfulfilled reservation expires after its TTL; expiry releases the hold without charging the customer`            | [[reservation.order]] |
| stock_exhausted                | effect    | `reservation.order.stock_exhausted(detail)`                                                                           | [[reservation.order]] |

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
