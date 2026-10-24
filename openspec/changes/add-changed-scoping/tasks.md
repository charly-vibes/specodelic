# Tasks: add-changed-scoping

## 0. Prerequisite (blocked-by)

- [ ] 0.1 specodelic-523w — compile: corpus runner scope guard (later
      same-stem file silently wins). This change reuses the guard.

## 1. Red — failing tests first

- [ ] 1.1 Fixture: corpus of 3 files, one changed — test asserting
      compile/model-check/verify execute only for the changed path while
      lint still reports over the corpus.
- [ ] 1.2 Totality test: a changed file's cross-file references still lint
      corpus-wide (no dangling-ref regression from scoping).
- [ ] 1.3 Same-stem shadowing test: two same-stem files in scope produce the
      labeled failure the 523w guard defines — never a silent win.

## 2. Green — make the tests pass

- [ ] 2.1 Path filter for artifact-local stages; lint unaffected.
- [ ] 2.2 Summary marks which stage ran scoped vs corpus-wide.
- [ ] 2.3 Decision-guide row documenting the lint corpus-wide rationale.

## 3. Refactor — separate commit

- [ ] 3.1 Stage-runner unification (scoped/corpus-wide share one runner).
- [ ] 3.2 `just ci` green.

## 4. Dual-format layer (fail-closed gate)

- [ ] 4.1 Extend the `orchestrate.layer` sidecar and validate via
      `just archive-change id=add-changed-scoping`.
