# Change: Verify runner seams — tool dependency, fixture API, properties-only verdict

## Why

The lf3 scoped run (2026-11-13) measured that **0 of 537** corpus property
rows are honestly expressible as executable `**rust:**` fragments today
(`.wai/projects/specodelic/research/2026-11-13-lf3-feasibility-summary.md`).
Three seams are the blocker, and none of them is the runner's execution
machinery — fragments DO execute today (M3). The blockers are:

- **M1 — the scratch crate cannot see the tool.** `src/verify.rs`'s
  `SCRATCH_MANIFEST` declares `proptest` and nothing else, so a fragment
  cannot call the linter, the compiler or the graph builder — the very
  things the corpus's property rows assert. A fragment that re-implements
  the checker inside itself is a tautology (M6), not evidence.
- **M2 — a named generator yields a constant.** `spec_gen::<name>()` is
  `Just("<name>".into())`: one input per property, and the input is the
  generator's own name. The corpus's generators name FIXTURES
  (`spec_file_with(transition.to_not_in_states: true)`,
  `pack_pinned_to_older_revision()`), and nothing turns such a name into a
  concrete spec.
- **M3' — the verdict hides the properties stage.** `spk verify` reports
  the first blocking stage per file, so a file whose properties pass but
  whose model run is missing reports `missing_model_run` — a scoped CI
  recipe cannot tell "properties passed" from "properties were never
  reached".

Without these three seams, no corpus property cell can be migrated
honestly, and the `verify-specs` CI gate cannot be wired.

## What Changes

- **P1 — the tool-dependency seam.** The scratch cargo crate the verify
  runner materializes gains a path dependency on the specodelic lib
  (`[dependencies] specodelic = { path = "<crate root>" }`) exactly when
  the staged artifact references `specodelic::`. A property fragment may
  therefore call the REAL checker (`specodelic::lint::lint_corpus`,
  `specodelic::compile::properties_to_proptest`). The runner contract is
  unchanged: the same `PropertiesRunner` trait, the same report shape.
  Artifacts that do not reference the tool keep today's manifest
  byte-for-byte — no library rebuild, no path resolution, no behavior
  change.
- **P2 — the fixture API.** A new public, documented
  `specodelic::fixtures` registry maps a fixture name to a concrete spec
  text. The emitted `spec_gen::<name>()` strategy materializes a
  fixture-named generator through the registry (a real concrete input)
  instead of `Just("<name>")`; unknown names keep today's emission
  byte-for-byte. Every fixture is a real spec file the real parser and the
  real checkers consume — no test doubles, no re-implemented checkers.
- **P3 — the properties-only scope verdict.** `verify::properties_only_verdict`
  reports `properties_failed` exactly when a compiled block failed, never
  for a missing or stale model run, and never reports `verified` — the
  full `both_gates_required` semantics are untouched. The shipped CLI
  surface is the envelope's per-file `properties.state` (already exactly
  "failed" ⟺ a block failed) plus a `just` recipe that gates on it.
- **Demonstration.** ONE property row of `openspec/specs/compile/spec.md`
  (`proptest_block_per_property`) is migrated to a `**rust:**` fragment
  that calls the real compiler on a materialized fixture; the bulk
  migration is lf3's next run.

## Impact

- Affected specs: `verify` (a new properties-only scope constraint,
  property and requirement) and `compile` (one migrated property row and
  its mirrored scenario). No other capability's requirements change.
- Affected code: `src/verify.rs` (runner seam + scoped verdict),
  `src/lib.rs` (the `fixtures` module and the `scope_verdict` module —
  the latter for the `pretender.toml` file-lines ratchet, see design D3),
  `src/compile.rs` (the emitted
  strategy for a fixture-named generator).
- Not in scope (owned elsewhere): real generator strategies
  (specodelic-jwjj), law-case fragment bodies (a format Revision),
  the generator-adequacy gate (specodelic-l8l.10), the bulk corpus
  migration (specodelic-lf3), and CLI flag wiring in `src/main.rs` /
  `src/commands/**` (outside this ticket's allowed file set — the
  envelope field plus the recipe is the shipped surface).
