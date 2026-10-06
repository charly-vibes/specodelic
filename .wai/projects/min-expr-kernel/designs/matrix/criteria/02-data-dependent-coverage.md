# data-dependent-coverage

Does the approach give the data-dependent tier (~204 equational cells at `028fc50`, e.g. `refund_amount == paid_amount`, `refund_issued_at - cancelled_at <= 5 business days`) an expression form that is not one runtime's syntax? Today the only executable form is a verbatim `**rust:**` blob (100% coupling) or prose (0% checking) — no middle tier.
