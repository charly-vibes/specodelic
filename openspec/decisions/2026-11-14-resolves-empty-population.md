# Decision: `resolves(m)` over an empty morphism population is vacuously verified

*Decision note, 2026-11-14 (specodelic-9ilm; 8mta Rule-of-5 review M4).*

## What is decided

`Atomic::Resolves(m)` means **every defined value of `m` resolves**. When
the instance has no values for `m` — a corpus with no `uses` edges, for
example — the universal is vacuously true and the atomic evaluates
`verified`, not `unknown`.

- This is the intended reading, not a defect: the claim asserts a property
  of a relation; a corpus that never exercises the relation has no
  counterexample.
- `unknown` stays reserved for claims the evaluator **cannot discharge**:
  an id literal naming no declared id (`specodelic-8mta` F1), a
  `reachable` seed or target that is not an id of the instance, an
  unsupported backend.
  A vacuous universal *is* discharged.
- The reading is documented in `openspec/specs/kernel/spec.md`'s Notes and
  pinned by
  `tests/kernel_status.rs::resolves_over_an_empty_morphism_population_is_vacuously_verified`.
- No lint advisory in v1. A claim over a relation the corpus never
  exercises is a modeling smell, not an error; if dogfood friction
  demonstrates otherwise, an advisory on the warnings channel is a
  separate, later change.

## Why

Three-valued honesty requires `unknown` to mean "cannot decide", not
"decided, but uncomfortable". Coercing vacuous truth to `unknown` would
make `resolves` unusable over corpora that legitimately do not exercise a
relation — and would be a coercion from a decided status, the same class
of dishonesty `specodelic-8mta` removed on the literal side.

## Consequences and limits

- A claim can be `verified` without the corpus exercising the relation.
  That is now stated and test-pinned, not implicit.
- Morphism names are still parse-validated against the schema
  (`morphism_arg`), so a typo is a labeled grammar failure — not this
  case.
- No behavior, grammar, or report change ships with this decision.