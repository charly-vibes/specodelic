# Change: `audit --before` never retires a contradiction (specodelic-f1by)

## Why

`spk audit --before <report.json>` overlays a diff-currency label
(`current` / `stale` / `new`) onto every row. The overlay overwrites the
row's evidence class, so a live contradiction disappears from the report
and from the exit code in two ways:

- **Sustained contradiction** — prior `contradicted`, current
  `contradicted`: `overlay_class` compares ranks, `2 > 2` is false, and the
  row is relabelled `current`. The report says nothing is wrong and the run
  exits 0.
- **Unseen contradiction** — the prior report never saw the row: the class
  becomes `new`, which the exit predicate ignores. Exit 0 again.

`check` already works around this by counting contradicted rows before the
overlay (`src/check.rs` `Judgement::contradicted`), which is the existence
proof that the diff itself is the wrong place to hide a failure. An audit
that reports zero contradictions while a contradiction persists is the
empty-pass failure the graded-evidence discipline exists to prevent.

## What Changes

- `audit::diff_with_prior` SHALL preserve `contradicted`: a record whose
  pre-overlay class is `contradicted` keeps that class whether the prior
  report held the same contradiction or never saw the row. Every other row
  keeps today's overlay semantics (`held`/improved ⇒ `current`, worsened ⇒
  `stale`, unseen ⇒ `new`).
- The documented consequences follow from the existing predicates with no
  new machinery: `exit_code` returns 1, `class_counts` counts the
  contradiction, and the human view names it.
- `src/check.rs` keeps its pre-overlay count (the exit predicate now agrees
  either way) and its comment stops claiming the overlay demotes
  contradictions.

## Impact

- Affected capability: `audit` (one MODIFIED requirement, one new layer
  constraint and its property).
- No report-schema change, no new audit class, no vocabulary change: prior
  reports written before this change remain valid `--before` priors, and
  `check`'s report shape is untouched.
- Behavior change is confined to `--before` runs that carry a live
  contradiction: they now exit 1 instead of 0 — the bug's whole point.

## Non-goals

- No currency axis added beside the class field (schema v2 is not needed to
  stop retiring failures).
- No change to the exit contract for `stale`, `unchecked`, `binds_nothing`
  or `new` rows.
- No upstream `ah` work; no contract runner, adapter registry, or
  independence claim.