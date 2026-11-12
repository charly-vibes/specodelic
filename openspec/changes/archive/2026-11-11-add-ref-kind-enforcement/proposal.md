# Change: Enforce Reference Typing (ref_kind_compatible) as a lint rule

## Why

The five-field (now nine-row) Reference Typing table in
`openspec/specs/specodelic/spec.md` is promised by the rule catalog
(`guard_required`'s semantics defer target typing to
`ref_kind_compatible`), by `src/guide.rs` (`REFERENCE_TYPING` const), and
by `spk graph` (which already reports typing violations from the same
schema-as-data table) — but no lint rule checks target kinds. Verified on
the v0.5.2 binary (2026-10-07, beads specodelic-d8c): a guard citing an
`advisory` constraint, a state `emits`-ing an `invariant` constraint, and
a constraint `traces_to`-ing another constraint all lint clean. The
checker file that owns the check
(`openspec/specs/linter-referential_integrity/spec.md`) declares
`ref_kind_compatible` as an invariant constraint with a declared
property (`wrong_target_kind`) — the linter never fires it. The
`linter lint` gate therefore admits kind-incompatible references that
`spk graph` labels as violations on the very same corpus.

## What Changes

- New lint rule `ref_kind_compatible` (rule id `linter.ref_kind_compatible`)
  in the linter's rule catalog, firing on the issues channel (error
  severity — it gates the checker's `accept` transition) when a resolved
  `[[link]]` in a Reference-Typing-typed column targets a row of a
  kind the table forbids.
- The decision is read from the Schema as data
  (`acset::schema::canonical()` via `typing_violation`) — no per-field
  match arms — so the next Reference Typing row addition needs no
  checker-code change (the constraint's own expr demands exactly this).
- Resolution reuses the linter's corpus-wide index and shares the graph's
  semantics: typed columns only, metasyntactic skip, dangling targets stay
  `total_refs`' beat. Member-arm targets (`file.row.member`) canonicalize
  to the row and are typed as the row — matching the independent oracle
  (`tests/reference_oracle.rs`), which is stricter than the graph here.
- `from`/`to` are NOT duplicated: their State typing is already enforced
  by `every_transition_valid` (same-file State membership).
- Corpus conformance verified by probe before implementation: `spk graph`
  reports zero typing violations over `openspec/specs` (2026-11-12) — no
  corpus rows need conforming and no typing-table row needs widening.
- Affected specs: `linter-referential_integrity` (ADDED requirement).
  No modification to `specodelic`'s Reference Typing table (append-only,
  nothing widens) and no change to `guide.rs`'s wording.

## Impact

- Affected specs: `openspec/specs/linter-referential_integrity/spec.md`
  (deployed only via the `just archive-change` companion recipe).
- Affected code: `src/lint/mod.rs` (RULE_TABLE row + wiring),
  `src/lint/graph.rs` (the check), tests (`src/lint` fixture corpus +
  `tests/cli/lint.rs` CLI fixtures + `tests/reference_oracle.rs`
  agreement stays green).
- Behavior change: corpora that previously linted clean with
  kind-incompatible typed references now fail with labeled findings —
  that is the point (enforce, not demote: demotion would change
  documented behavior across three surfaces and is rejected).
