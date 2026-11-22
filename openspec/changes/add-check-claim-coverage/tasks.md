# Tasks — add-check-claim-coverage

> TDD order: each numbered section starts with the failing test (RED), then the
> implementation (GREEN), then the tidy pass. No section may land without its
> RED evidence in the commit that carries the GREEN.

## 1. Report assembly — the claim-coverage section

1.1 RED: a `check` run over a file whose declared claims are partly
    unevidenced produces a report whose coverage section lists every declared
    invariant claim — including one with no evidence row (`src/check.rs`
    report builder; fixture: a spec with two claims and one evidence row).
1.2 GREEN: the report builder derives the section from the parsed spec's
    declared claims and the rows the run produced, indexed by the
    file-qualified claim id (`<file id>.<claim id>`).
1.3 TIDY: the section's ordering follows the corpus's declaration order; no
    hash-map iteration order leaks into the serialization.

## 2. Claim keys — file-qualified, never abbreviated

2.1 RED: a row for a claim id that also exists in another capability file
    carries the file-qualified key, and the two files' rows stay distinct
    (fixture: two files declaring `process_lifecycle`).
2.2 GREEN: the row key carries the file-qualified claim id; the existing
    producer-shaped key (`verify:prop:*`, `verify:model`) is preserved
    unchanged — report interoperability depends on it.
2.3 TIDY: no claim is matched by name, slug, filename, or similarity; a
    near-miss produces no row (test: a sidecar-style slug resembling a claim).

## 3. Classes — reuse audit's closed set

3.1 RED: the coverage section's class values are exactly audit's closed set;
    a claim with no evidence row carries `unchecked`; the run's
    `scripts/check_evidence_axes.py` gate passes with no new documented value
    row.
3.2 GREEN: map each claim to its row's audit class, or `unchecked` when no row
    corresponds.
3.3 TIDY: no new enum value, no new vocabulary, no boolean "covered" flag
    (the corpus forbids stored booleans — derived facts are computed).

## 4. Non-gating

4.1 RED: a run whose rows are all current and which declares an unchecked
    claim exits 0; the unchecked count appears in every output view.
4.2 GREEN: the section contributes counts, never an exit-code contribution.
4.3 TIDY: the count is surfaced beside the existing evidence counts, not in a
    separate summary block that a reader might miss.

## 5. Residuals — rows that place no claim

5.1 RED: a row that corresponds to no declared claim of the judged file (for
    example a model-level row) is accounted for in the section's residual
    bucket rather than dropped.
5.2 GREEN: the residual bucket lists such rows.
5.3 TIDY: the residual bucket reuses the existing class set; it introduces no
    new spelling.

## 6. Docs and gates

6.1 `docs/src/graded-evidence.md` — document the section's shape, the class
    mapping, and the rule that the section never gates.
6.2 `scripts/check_evidence_axes.py` — passes with no new value row (assert
    in the change's own gate run).
6.3 `just ci` — fmt, clippy `-D warnings`, tests, release build.
6.4 `spk lint openspec` — the corpus lints clean; the change's layer
    sidecar lints clean on its own (`spk lint
    openspec/changes/add-check-claim-coverage/specs/check/layer.md`).

## 7. Archive readiness

7.1 The plain delta (`specs/check/spec.md`) carries the two ADDED
    requirements; the layer sidecar (`specs/check/layer.md`) carries the three
    new constraints and three new properties.
7.2 `just archive-change id=add-check-claim-coverage` deploys the layer
    verbatim and fail-closes if the sidecar is missing.
7.3 The `## Requirements` layer-sidecar contract requirement is present in
    the sidecar.
