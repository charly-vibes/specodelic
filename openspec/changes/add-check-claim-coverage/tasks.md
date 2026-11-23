# Tasks — add-check-claim-coverage

> TDD order: each numbered section starts with the failing test (RED), then the
> implementation (GREEN), then the tidy pass. No section may land without its
> RED evidence in the commit that carries the GREEN.

## 0. Preconditions (not this change's work — the change depends on them)

0.1 `spk check` must be able to judge a corpus capability file at all.
    Measured 2026-11-24: 0 of 50 corpus capability files are judged — 19
    refuse at `artifact_gate` (`src/check.rs:242` calls the corpus-less
    `conform::gate`; the corpus-aware `conform::gate_in_corpus` is wired for
    `spk conform` only, `src/commands/corpus.rs:888-925`), and 31 refuse at
    `verify_model` for an absent model-check report. Tracked as its own bead;
    this change neither fixes nor depends on the gate's behaviour, but its
    self-hosting exercise (task 6.5) cannot run until it lands.
0.2 The coverage section's RED tests use fixtures, not the corpus — a fixture
    is a spec file with two declared claims, one property deriving from one of
    them, and evidence rows that place one and not the other. The corpus is
    the self-hosting exercise, never the unit test.

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

## 2b. Placement — which row places which claim

2b.1 RED: a run whose evidence row names a property whose `derives_from`
    names a declared invariant claim lists that claim in the coverage section,
    and a row naming a property that derives from no declared claim of the
    judged file places none.
2b.2 GREEN: the section resolves a placing row to its claim through the
    property's `derives_from` column — the corpus's declared edge — and a row
    that names neither a claim nor such a property lands in the residual
    bucket.
2b.3 TIDY: the resolution reads the parsed spec's own rows; no second
    resolution path, no name matching, no slug table.

## 2c. The fold — several rows, one class

2c.1 RED: a claim placed by both a current row and a contradicted row is
    listed once, with class `contradicted`; a claim placed only by a vacuous
    binding row carries `binds_nothing`; a claim placed by no row carries
    `unchecked`.
2c.2 GREEN: the class is the strongest class among the placing rows, in the
    order `contradicted` > `current` > `binds_nothing` > `unchecked`.
2c.3 TIDY: the fold is one named function with the order in one place; a
    new class needs one edit there, not one per call site.

## 2d. Where the section lives, and what it says about itself

2d.1 RED: the emitted report's audit payload carries the audit report's
    schema version and field set unchanged, the coverage section rides the
    check envelope, and the section's rendered text states that its classes
    describe the evidence this run's producers produced.
2d.2 GREEN: the section is assembled into the check envelope beside the
    report; `AuditReport` gains no field; the scope statement is part of the
    section's rendering.
2d.3 TIDY: the scope statement is one named constant, rendered by the human
    and JSON views alike — never written twice.

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
    separate summary block that a reader might miss — and on the verdict line
    of the human view, so a count that says "nothing covered" cannot be read
    as a pass.

## 5. Residuals — rows that place no claim

5.1 RED: a row that corresponds to no declared claim of the judged file (for
    example a model-level row) is accounted for in the section's residual
    bucket rather than dropped.
5.2 GREEN: the residual bucket lists such rows.
5.3 TIDY: the residual bucket reuses the existing class set; it introduces no
    new spelling.

## 6. Docs and gates

6.1 `docs/src/graded-evidence.md` — document the section's shape, the class
    mapping (with the fold order), the placement rule, and the rule that the
    section never gates.
6.2 `scripts/check_evidence_axes.py` — passes with no new value row (assert
    in the change's own gate run).
6.3 `just ci` — fmt, clippy `-D warnings`, tests, release build.
6.4 `spk lint openspec` — the corpus lints clean, and the change's layer
    sidecar resolves within that tree (the sidecar alone cannot pass:
    `total_refs` resolves against the linted set only, so linting
    `layer.md` by itself reports dangling cross-file references by
    construction — always lint the containing tree).
6.5 Self-hosting (blocked on task 0.1): the section is exercised against one
    of this repo's own capabilities. The measured baseline stands on the
    record — of 408 invariant claims, 76 are placeable by this repo's own
    producers and 202 are reachable through the sibling contract surface the
    tool may not read; 470 of 572 properties are still `todo_predicate!`
    stubs. A capability whose properties are stubs produces a refusal, not a
    coverage list, and that is the honest reading.

## 7. Archive readiness

7.1 The plain delta (`specs/check/spec.md`) carries the two ADDED
    requirements; the layer sidecar (`specs/check/layer.md`) carries the three
    new constraints and three new properties.
7.2 `just archive-change id=add-check-claim-coverage` deploys the layer
    verbatim and fail-closes if the sidecar is missing.
7.3 The `## Requirements` layer-sidecar contract requirement is present in
    the sidecar.
