---
id: reservation.order
kind: intent
statement: "WHEN a customer submits an order THE reservation service SHALL hold stock for every line item and accept the order, or reject it with nothing held and nothing charged."
---

# Reservation & Order

A small order pipeline: submitting an order holds stock for every line
item or rejects the order outright; accepted orders are charged exactly
once, fulfilled, or released by cancellation or expiry.
