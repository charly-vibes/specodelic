---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-lf3-migrate-compile-properties, pipeline-step:orient]
---

ORIENT: specodelic-lf3 run 2 — scoped migration of openspec/specs/compile/spec.md (35 unit property rows, zero law rows). Runner seams landed in 449aa5b0: fixtures registry (src/lib.rs), tool-dependency seam (scratch crate path dep on specodelic), scope_verdict::properties_only_verdict. Generator names in the property table are regex-captured as name( — so each row's generator cell names a fixture the registry must carry. Plan: add 30+ real spec-text fixtures to src/lib.rs fixtures registry, migrate each predicate cell to a **rust:** fragment that calls the real tool (specodelic::compile::compile_spec / properties_to_proptest / constraints_to_toml / model_to_tla / lint::lint_corpus / spec::parse_str), regenerate specodelic/compile_props.rs + compile.check.json, then wire a just verify-specs recipe scoped to openspec/specs/compile/spec.md.
