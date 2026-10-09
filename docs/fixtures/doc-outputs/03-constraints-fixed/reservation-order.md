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
