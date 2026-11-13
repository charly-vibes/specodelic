---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-u7kc-verify-runner-seams, pipeline-step:plan]
---

TDD plan — specodelic-u7kc verify runner seams

DESIRED BEHAVIOR
- P1 (tool-dependency seam): the scratch cargo crate the verify runner materializes gains a path dependency on the specodelic lib exactly when the compiled artifact references `specodelic::`; artifacts that do not reference the tool keep today's manifest byte-for-byte (no lib rebuild, no path resolution risk). A property predicate fragment may therefore call the REAL checker (specodelic::lint::lint_corpus / specodelic::compile::properties_to_proptest). Runner contract unchanged (PropertiesRunner trait + report shape).
- P2 (fixture API): a new public, documented `specodelic::fixtures` registry (name -> concrete spec text). The emitted `spec_gen::<name>()` strategy materializes a fixture-named generator through the registry (a real concrete input) instead of `Just("<name>")`; unknown names keep today's Just(name) emission byte-for-byte.
- P3 (properties-only verdict): `verify::properties_only_verdict` (scoped) reports `properties_failed` exactly when a compiled block failed, never for a missing/stale model run, and never reports `verified` (both_gates_required semantics unchanged); a just recipe gates a run on the envelope's per-file `properties.state == "failed"`.

OUT OF SCOPE
- Real generator strategies (specodelic-jwjj), law-case fragment bodies (format Revision), generator-adequacy gate (specodelic-l8l.10), bulk corpus migration (lf3), CLI flag wiring in src/main.rs + src/commands/** (outside the ticket's allowed file set -> the envelope field + recipe is the shipped surface), corpus fixture sections (a format Revision).

TEST CASES (RED first)
- tests/verify_runner_seams.rs::scratch_artifact_calling_the_tool_compiles_and_executes — a property predicate fragment calling specodelic::lint::lint_corpus: today the properties gate is Uncompilable (unresolved crate); after P1 it is Pass.
- tests/verify_runner_seams.rs::fixture_named_generator_materializes_a_concrete_spec — specodelic::fixtures::materialize(<fixture>) is Some(spec text) that parses; the emitted strategy calls the registry, not Just(name).
- tests/verify_runner_seams.rs::properties_only_verdict_reports_failed_only_for_a_failed_block — Failed -> properties_failed; Pass -> a non-verified status; model Missing/Stale never -> properties_failed.
- src/verify.rs unit tests: properties_verdict extraction + scoped verdict reuse.

NARROW TEST COMMAND: cargo test --test verify_runner_seams ; cargo test --lib verify::
FULL VERIFICATION: just ci (fmt-check, clippy -D warnings, tests, release build) + ah check + spk lint openspec/specs
FILES TOUCHED: src/verify.rs, src/lib.rs (fixtures module only), src/compile.rs (emission only), tests/verify_runner_seams.rs, openspec/changes/add-verify-runner-seams/**, openspec/specs/{verify,compile}/spec.md (only via just archive-change), .espectacular/{verify,compile}/*.toml (new contracts only), justfile (properties-only recipe)
