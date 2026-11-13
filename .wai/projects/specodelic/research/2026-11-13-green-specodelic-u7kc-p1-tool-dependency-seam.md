---
tags: [pipeline-run:epic-orchestrator-2026-11-13-specodelic-u7kc-verify-runner-seams, pipeline-step:spawn-subagent]
---

GREEN (specodelic-u7kc): P1 tool-dependency seam — src/verify.rs adds the scratch crate's path dependency on the tool (specodelic = { path = <crate root> }) exactly when the staged artifact contains 'specodelic::'; the tool root is env!(CARGO_MANIFEST_DIR) with the SPECODELIC_TOOL_PATH override, and the scratch lockfile is seeded from the tool's own Cargo.lock. Proof: tests/verify_runner_seams.rs::scratch_artifact_calling_the_tool_compiles_and_executes (RED before: E0433 unresolved crate specodelic; GREEN now: the fragment calls the real lint_corpus and observes the real rule id linter.model_present).
