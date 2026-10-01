# Design: update-law-named-cases

Decisions of record for making law-row named cases machine-checkable.

## D1 — Capability targeting (confirmed per the ticket's instruction)

The ticket says: *MODIFIED requirement on the capability owning law-row
checking (confirm: linter-coverage vs linter-schema_shape, via openspec
spec list)*. Confirmation performed 2026-10-01:

- `openspec list --specs` has **no** `linter-coverage` and **no**
  `linter-schema_shape` capability — the domain checker files
  (`specs/linter-*.md`) are not mirrored as capabilities.
- Domain-side ownership is settled by the Checker Ownership table
  (`specs/specodelic.md` line 98): `linter-coverage.md` owns
  `coverage` **and** `law_requires_cases`.
- The openspec capability that *owns law-row checking* is **compile**:
  the pipeline row (`specs/specodelic.md` line 76) gates compile on
  `[[specodelic.coverage]] ∧ [[specodelic.law_requires_cases]]`, and
  compile's "Properties table compiles to proptest blocks" requirement
  is the only deployed requirement whose behavior reads law-row cases.

→ Deltas target `compile` (law-case behavior) and `spec-integration`
(MODIFIED-delta mirror widening). The linter rule itself is specified
domain-side in `specs/linter-coverage.md`, like every other checker.

## D2 — The machine form is the form compile already parses

No new syntax. `compile.rs::required_law_cases` (regex
`\*\*([a-zA-Z][a-zA-Z _-]*?):\*\*`) is the de-facto standard: corpus
law rows already use `**identity:**` / `**associativity:**` /
`**naturality:**` labels, USAGE §2.5 documents the shape, and the
compile gate expands one proptest block per label. The change ratifies
this regex as the format's machine-findable case form and enforces it
at lint time — *before* the silent fallback can mask anything.

Rejected alternative: the ticket's sketch `case "associativity": expr`
— inventing a second syntax when one is already deployed corpus-wide
would churn every law row for zero gained checkability.

## D3 — Rule shape: `law_cases`, append-only

- `RULE_TABLE` gains `("law_cases", "every law-kind property must
  enumerate its required cases as **name:** labels in its own
  predicate — the identity and associativity floor is mandatory, extra
  named cases are checkable declarations")`. The catalog is
  append-only; `rule_id` is `linter.law_cases`.
- Finding message names each missing floor case and carries the
  remediation hint (write `**identity:** … **associativity:** …`
  labels). The spec-side error contract stays
  `linter.coverage.law_case_failure` (owned by
  `specs/linter-coverage.md`'s effect row — not re-declared in the
  deltas, keeping the file-id-namespaced error-label law intact).
- Floor matching is on the sanitized label (trim + lowercase), so
  `**Identity:**` counts; a label like `**left_identity:**` satisfies
  nothing by itself — the floor cases must be present *by name*, and
  `left_identity`/`right_identity` remain extra named cases (USAGE
  §2.5's monad example already pairs the floor with them explicitly).

## D4 — Shared case-label helper (refactor commit, separate)

`required_law_cases` is private to `compile.rs`; lint needs the same
parsing. Lift the label extraction into one `pub(crate)` helper (exact
home decided at implementation — `spec.rs` is the natural neighbor of
the other shared parsing seams) and have both call sites use it, so
the linter and the compiler cannot disagree on what a case is. Tidy
First: the lift lands as its own refactor commit before the rule's
green commit.

## D5 — First `## MODIFIED Requirements` delta: widen the mirror rules

Every archived delta so far used `## ADDED Requirements`; the
dual-format rules key on that header:

- `requirement_drift` / `dual_format_valid` (`src/lint.rs`): compare
  and fire on `## ADDED Requirements` only.
- `scripts/check_section_sync.py`: compares the
  `ADDED Requirements` ↔ `Requirements` sections; a MODIFIED-carrying
  file is simply skipped.

Widening is **additive**: treat `## MODIFIED Requirements` exactly like
`## ADDED Requirements` in all three places (a file carrying either
header declares `id: spec`, pairs the mirror, and drift fails CI).
Capability specs (plain `## Requirements`, no delta section) keep
failing the capability-format check exactly as before. ADDED-keyed
tests stay green unchanged (D: added_rules_unchanged).

## D6 — Live corpus catch: `topo_sort_naturality`

`specs/linter-graph_shape.md` line 54's law row carries only
`**naturality:**`. Once `law_cases` ships, `just lint-specs` fails
here — the fix follows the cxq precedent (`coverage_naturality`'s
decision of record): instantiate the floor at the row's observation
point with rename-instantiation cases,
`**identity:** topo_sort(rename(I, a, a)) == topo_sort(I)` and
`**associativity:** topo_sort(rename(rename(I, a, b), b, c)) ==
topo_sort(rename(I, a, c))`. This fix is part of implementation (the
corpus must be lint-clean under the new rule before gates run), not a
separate ticket.

## D7 — Demo promotion

`/tmp/resume-specs/batch-resume.md` (Rule-of-5 reviewed 2026-09-30)
moves to `docs/src/examples/batch-resume.md` with its law rows
relabeled to the machine form — its current `associativity: …` plain
labels are exactly the prose shape `law_cases` now rejects, which makes
the promoted example a before/after demonstration of the rule.
`docs/src/SUMMARY.md` gains the entry (the SUMMARY-completeness
checker requires every spec file listed; no absolute URLs, per
specodelic-j0m).

## D8 — Archive caution

Archive with the dual-format recipe: `spk archive-companion
update-law-named-cases` (shipped in specodelic-fzo). Default
`openspec archive` regenerates capability specs and destroys the
specodelic layer — the exact failure this repo's archive-companion
capability exists to prevent.

## Explicit non-goals

- No new property kinds: `property_kind_closed = {unit, law}` stands.
- No `## Operations` section: Constraints + law rows already carry the
  algebra (USAGE §2 preface).
- Compile's unlabeled fallback is unchanged: for lint-clean input it
  is unreachable (the precondition gate runs the new rule first); it
  remains defense-in-depth for direct API callers that skip the gate.