# Change: update-law-named-cases — make law-row named cases machine-checkable

## Why

Law rows are monoid witnesses. `law_requires_cases`
(`specs/specodelic.md` line 27) mandates associativity + identity as "a
floor, not a ceiling", and extra named cases (commutativity,
idempotence, naturality) are what drive derived-parallelism claims
(`reduce_sound_iff_monoid` in the reviewed demo). But *where* cases
appear in the predicate is free text: the linter cannot distinguish a
real case from prose merely mentioning one.

Two live catches make the gap non-hypothetical:

- `specs/linter-graph_shape.md`'s `topo_sort_naturality` law row
  carries only `**naturality:**` — the `law_requires_cases` floor
  (identity, associativity) is missing, yet `just lint-specs` is green
  because the floor is unenforced.
- The reviewed demo (`/tmp/resume-specs/batch-resume.md`,
  `specodelic-9qw` motivation) writes its law rows as
  `associativity: merge(merge(a, b), c) == …` — plain labels no
  machine can tell apart from prose.

The de-facto machine form already exists:
`compile.rs`'s `required_law_cases` (the `**name:**` case-label regex)
parses exactly this shape to expand law rows into one proptest block
per case. The format never ratified it, and nothing enforces it before
compile — compile's silent floor fallback even *masks* unlabeled rows
for lint-clean input's siblings.

## What Changes

- **Format (domain corpus):** `specs/specodelic.md` gains **Revision
  13** — the `law_requires_cases` row now names the machine-findable
  form: required cases are enumerated as `**name:**` case labels in the
  law row's own predicate; the identity + associativity floor is a
  minimum; extra named cases are first-class checkable declarations.
- **Linter (enforcement):** new append-only rule `law_cases`
  (`rule_id: linter.law_cases`) — a law-kind Property row whose
  predicate lacks the floor labels in machine form is rejected, naming
  each missing case with a remediation hint. Spec-side, this is
  `linter-coverage.md`'s `every_law_has_cases` / the
  `linter.coverage.law_case_failure` error contract, finally executed.
- **Compile capability (MODIFIED requirement):** "Properties table
  compiles to proptest blocks" is updated — law rows expand to exactly
  one block per label-enumerated case; the unlabeled fallback stays
  only as defense-in-depth for input that skipped the compile
  precondition. `required_law_cases`' case-label parsing is lifted into
  a shared helper so lint and compile cannot diverge on the form.
- **spec-integration capability (MODIFIED requirements):** this is the
  repo's first `## MODIFIED Requirements` delta, so the dual-format
  mirror rules (`requirement_drift`, `dual_format_valid`,
  `scripts/check_section_sync.py`) are widened additively: a file
  carrying `## MODIFIED Requirements` is treated exactly like one
  carrying `## ADDED Requirements` (`id: spec`, mirror checked, drift
  CI-failed).
- **Companions (non-format):**
  - `specs/USAGE.md` §2.8 pattern entry: *derived parallelism — the
    accumulator's algebra licenses the architecture* (the demo's tier
    table: deterministic → map; monoid → chunked reduce; +commutative
    → order-free workers; semilattice → free concurrent checkpointing).
  - The demo is promoted out of `/tmp` to a durable worked example:
    `docs/src/examples/batch-resume.md` (law rows relabeled to machine
    form), linked from `docs/src/SUMMARY.md`.

**Explicitly out of scope:** new property kinds
(`property_kind_closed = {unit, law}` is correct); any `## Operations`
section (Constraints + law rows already carry the algebra); changing
compile's fallback behavior for non-preconditioned input.

## Impact

- **Capabilities:** `compile` (1 MODIFIED requirement),
  `spec-integration` (2 MODIFIED requirements: Dual-format delta,
  Section sync).
- **Domain corpus:** `specs/specodelic.md` (Revision 13),
  `specs/linter-coverage.md` (`every_law_has_cases` wording),
  `specs/linter-graph_shape.md` (`topo_sort_naturality` floor fix),
  `specs/USAGE.md` (§2.8), plus `src/guide.rs` `FORMAT_REVISION`
  bump (drift guard).
- **Code:** `src/lint.rs` (rule + shared case-label helper),
  `src/compile.rs` (helper lift, no behavior change),
  `scripts/check_section_sync.py` (MODIFIED headers), mdBook docs
  (new examples page).
- **Dogfood:** corpus law rows already comply except
  `topo_sort_naturality` (fixed here); artifacts regenerate only if
  the graph-shape file changes ripple into compiled artifacts.
- **Archive caution:** this change must be archived with the
  dual-format recipe — `spk archive-companion update-law-named-cases`
  (specodelic-fzo). Default `openspec archive` destroys the
  specodelic layer.
- **Approval gate:** no implementation before this proposal is
  approved.