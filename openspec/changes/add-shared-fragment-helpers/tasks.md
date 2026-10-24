# Tasks: add-shared-fragment-helpers

## 1. Red — failing tests first

- [ ] 1.1 Fixture: spec with a `## Fragments` section defining one helper,
      referenced from two property fragments — compile currently ignores the
      section; emission test asserting the helper body is inlined at both use
      sites fails.
- [ ] 1.2 Collision test: helper named like a generated symbol fails labeled.
- [ ] 1.3 Byte-preservation test: identical file without the section compiles
      byte-identically before/after the change.
- [ ] 1.4 Hygiene test: a helper body containing a banned token fails labeled
      with the same rule as fragments.

## 2. Green — make the tests pass

- [ ] 2.1 Extraction: parse `## Fragments` (`def <name>:` + fenced body),
      resolve references by name, fail labeled on unknown reference.
- [ ] 2.2 Emission: inline bodies at use sites; dead-helper warning on the
      warnings channel.
- [ ] 2.3 Docs: format summary section + a bowling-style before/after size
      note in the case study (Wave 1 docs ticket lands the page).

## 3. Refactor — separate commit

- [ ] 3.1 Unify fragment-body extraction for inline and helper bodies.
- [ ] 3.2 `just ci` green.

## 4. Dual-format layer (fail-closed gate)

- [ ] 4.1 Extend the `compile.layer` sidecar (full post-change layer) and
      validate via `just archive-change id=add-shared-fragment-helpers`.
