# Tasks: update-law-named-cases

## 1. Spec deltas (this proposal — done at scaffold)

- [x] 1.1 `specs/compile/spec.md` dual-format delta: MODIFIED
  "Properties table compiles to proptest blocks" (machine-form case
  enumeration, mandatory floor, one block per case) + specodelic layer.
- [x] 1.2 `specs/spec-integration/spec.md` dual-format delta: MODIFIED
  "Dual-format delta" + "Section sync" (ADDED-or-MODIFIED mirror
  widening) + specodelic layer.
- [x] 1.3 `openspec validate update-law-named-cases --strict` passes;
  both deltas `spk lint` clean.

## 2. Format ratification (domain corpus)

- [ ] 2.1 RED (docs-text test): `specs/specodelic.md` Revision 13
  heading exists; the `law_requires_cases` row names the machine
  form (`**name:**` case labels, floor = identity + associativity,
  extras = checkable declarations). `src/guide.rs` `FORMAT_REVISION`
  bumped to `Revision 13` — the drift-guard test must stay green
  (corpus revision and embedded revision move together).
- [ ] 2.2 GREEN: write both edits.
- [ ] 2.3 `specs/linter-coverage.md`: `every_law_has_cases` constraint
  row wording updated to the machine form; new unit property rows:
  `unlabeled_law_rejected` (prose-mention-only predicate →
  `check(file) == failed`) and `extra_case_is_declaration`
  (extra named case → one block per case, per the compile delta).
  File must stay lint-clean under the new rule (its
  `coverage_naturality` law row already carries the floor labels).

## 3. Linter rule `law_cases` (TDD)

- [ ] 3.1 RED: fixture file with a law row whose predicate mentions
  identity/associativity in prose (no `**name:**` labels) → finding
  `rule_id == "linter.law_cases"`, message names both missing cases.
  The distinguishing case: today this file lints clean.
- [ ] 3.2 RED: law row missing exactly one floor case
  (`**identity:**` present, no associativity) → finding names the
  missing one only.
- [ ] 3.3 RED: law row with floor labels + extra named case
  (`**commutativity:**`) → zero findings; labeled-but-drifted
  spelling (`**identiy:**`) → finding.
- [ ] 3.4 RED: unit rows (kind != law) never trigger the rule, even
  with case-like labels in the predicate.
- [ ] 3.5 GREEN: implement — extract the floor check from the shared
  case-label helper; append the rule to `RULE_TABLE` (the
  fixture-coverage catalog test pins rule count + semantics).
- [ ] 3.6 Refactor (separate commit): lift `required_law_cases`' label
  regex out of `compile.rs` into the shared `pub(crate)` helper (D4);
  both call sites consume it; corpus compile artifacts byte-identical
  (no fn-name churn — sanitize_ident untouched).

## 4. MODIFIED-delta mirror widening (TDD)

- [ ] 4.1 RED (lint): dual-format fixture carrying
  `## MODIFIED Requirements` + drifted `## Requirements` mirror →
  `requirement_drift` fires naming the requirement (today: silent).
- [ ] 4.2 RED (lint): fixture carrying `## MODIFIED Requirements` with
  a non-`spec` id → `dual_format_valid` fires (today: silent).
- [ ] 4.3 RED (script): `check_section_sync.py` over a MODIFIED-carrying
  fixture with drifted mirror → exit 1 naming the file and requirement
  (today: skipped). ADDED fixtures unchanged (additive widening, D5).
- [ ] 4.4 GREEN: widen `requirement_drift`, `dual_format_valid` in
  `src/lint.rs` + the section-sync script; all existing ADDED tests
  pass unchanged.

## 5. Corpus dogfood

- [ ] 5.1 Fix `specs/linter-graph_shape.md` `topo_sort_naturality`:
  add the floor labels per D6 (rename-instantiation identity +
  associativity at the topo_sort observation point); note in the file
  that the catch came from the rule shipping.
- [ ] 5.2 `just lint-specs` → 0 issues; `spk graph specs` → 0 dangling;
  regenerate compiled corpus artifacts if the graph-shape edit ripples
  (`spk compile` artifacts are committed and byte-stable).

## 6. Companions (non-format)

- [ ] 6.1 `specs/USAGE.md` §2.8: *Derived parallelism — the
  accumulator's algebra licenses the architecture* — the demo's tier
  table (deterministic/replayable → map over any partition; monoid →
  chunked reduce; +commutative → order-free workers; semilattice →
  free concurrent checkpointing; non-monoid → reduce is sequential),
  citing `map_is_parallel_unconditional` /
  `reduce_sound_iff_monoid` from the worked example.
- [ ] 6.2 Promote the demo: `cp /tmp/resume-specs/batch-resume.md
  docs/src/examples/batch-resume.md`, relabel its law rows to the
  machine form (`**identity:**` / `**associativity:**` /
  `**idempotence:**`), keep it lint-clean (`spk lint
  docs/src/examples/batch-resume.md` — it is an `id: batch.resume`
  spec file).
- [ ] 6.3 `docs/src/SUMMARY.md`: link the new example page (relative
  path; the summary checker fails absolute URLs per specodelic-j0m).

## 7. Gates + ship

- [ ] 7.1 `just ci` (fmt, clippy `-D warnings`, tests, release build)
  + `just lint-specs` + `just openspec-validate` (`--strict`) +
  `just sync-sections` — all green.
- [ ] 7.2 CHANGELOG entry (#91+), `spk explain` topics untouched (no
  new topic needed — the floor semantics live in `dual-format`'s and
  `lint-rules`' existing bodies; verify the rule table renders).
- [ ] 7.3 Rule-of-5 review of the change set; fix findings.
- [ ] 7.4 Commit + push; at approval-archive time use
  `spk archive-companion update-law-named-cases` (D8 — never default
  archive).