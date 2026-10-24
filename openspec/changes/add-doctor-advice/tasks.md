# Tasks: add-doctor-advice

## 1. Red — failing tests first

- [ ] 1.1 Fixture A (control-dominated): 6-state traffic-light-style model,
      guards executable or simple — advice test asserts "run the model-check
      loop" routing and no data warning.
- [ ] 1.2 Fixture B (data-dominated): product-growth states + guards citing
      claims with no executable fragment — advice test asserts the
      data-dimension signal and the oracle/differential routing hint.
- [ ] 1.3 Advisory-only test: advice never changes exit codes for either
      fixture; corpus mutation test asserts doctor writes nothing.

## 2. Green — make the tests pass

- [ ] 2.1 Compute the three signals from parsed spec data (no re-parse).
- [ ] 2.2 Emit advice records with signal evidence inline (each advice names
      the numbers that triggered it).
- [ ] 2.3 Decision-guide routing row + FAQ cross-link.

## 3. Refactor — separate commit

- [ ] 3.1 Signal extraction into its own module, unit-tested in isolation.
- [ ] 3.2 `just ci` green.

## 4. Dual-format layer (fail-closed gate)

- [ ] 4.1 Extend the `doctor.layer` sidecar and validate via
      `just archive-change id=add-doctor-advice`.
