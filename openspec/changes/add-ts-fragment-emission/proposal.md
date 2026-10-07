# Change: Execute TypeScript properties with shared generator semantics

## Why

Existing ticket specodelic-aby names a TypeScript follow-up but has no
executable plan. After the Python change establishes generator IR and
multi-language result accounting, TypeScript can reuse those contracts
without multiplying constant-name generators or creating a runner registry.

## What Changes

- Emit deterministic TypeScript unit-property artifacts using fast-check
  and a selected project-local test toolchain, with row-level attribution.
- Map the established generator IR to the same value domains, including
  Unicode scalar semantics and explicit singleton domains.
- Integrate bounded execution, shrinking, staleness and complete mixed-
  language accounting through PropertiesRunner. Do not add model-check
  guard execution or automatic Rust/Python/TypeScript predicate translation.

## Impact and dependencies

Pending approval; owner specodelic-aby. Depends on add-py-fragment-emission
(specodelic-l8l), its generator/report contracts, and claim-gate policy.
New capability typescript-properties; full compile snapshot rebased only
AFTER the Python compile delta deploys. No simultaneous archive of the
Python and TypeScript compile overlays. New emitter/runner modules, shared
compile/verify wiring, CLI fixtures and release docs are implementation scope.
No runtime source changes now. Separate feature and refactoring tickets
are created from tasks.md after approval; do not duplicate aby.
