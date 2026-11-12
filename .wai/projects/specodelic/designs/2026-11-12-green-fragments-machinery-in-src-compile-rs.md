---
tags: [pipeline-run:tdd-ro5-2026-11-12-specodelic-ez9a-1-add-shared-fragment-helpers, pipeline-step:green]
---

GREEN: ## Fragments machinery in src/compile.rs (~280 lines): FragmentDef + fragment_defs parser (labeled on malformed/duplicate defs), helper_refs/rewrite_helper_refs (@name token scanner — boundary-aware, Rust pattern @ never matches), HelperExpander (memoized block-expression rewrite ({ items name }), visiting-stack cycle detection), validate_fragment_helpers (collision vs spec_gen/todo_predicate/block-fns/v[0-9]+, inherited FRAGMENT_BANNED hygiene, unknown refs), helper_warnings (liveness closure from fragment cells), Compiled.warnings carried via write_artifacts splice onto the existing warnings channel; spec.rs captures fragments_body verbatim. Emission hooks: properties_to_proptest block rewrite + extract_model_ir invariant rewrite (lenient, same contract as fragment_of). All 43 test targets green; pretender/specodelic gates pass.
