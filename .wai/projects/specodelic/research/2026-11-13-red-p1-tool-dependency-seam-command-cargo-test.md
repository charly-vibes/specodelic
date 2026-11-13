---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-u7kc-verify-runner-seams, pipeline-step:red]
---

RED (P1, tool-dependency seam): command=cargo test --test verify_runner_seams; expected failure=the compiled artifact's **rust:** fragment cannot name the tool — 'error[E0433]: cannot find module or crate `specodelic` in this scope' — so the properties gate reports Uncompilable instead of Pass. Observed: test scratch_artifact_calling_the_tool_compiles_and_executes FAILED with state Uncompilable("cannot find module or crate `specodelic`") (4.56s). The spec-first record (openspec/changes/add-verify-runner-seams/{proposal,design,tasks}.md) is written: D1 conditional tool dependency, D2 the specodelic::fixtures registry, D3 the properties-only scope verdict + envelope-field recipe, D4 the one-row compile.md demonstration, D5 the dual-format contract change.
