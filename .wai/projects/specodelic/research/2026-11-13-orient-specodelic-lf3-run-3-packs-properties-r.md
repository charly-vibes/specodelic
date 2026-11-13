---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-lf3-migrate-packs-properties, pipeline-step:orient]
---

ORIENT: specodelic-lf3 run 3 (packs properties). Read packs/spec.md (16 unit property rows, 0 law), the lf3 feasibility packs research (all 16 need tool-call predicates), run-2 precedent 861491e1 (compile file: fixtures registry in src/lib.rs + **rust:** fragments + verify-specs recipe scoped to compile). Landed seams: P1 SPECODELIC_TOOL_PATH path-dep, P2 fixtures registry builders, P3 properties-only scope verdict. Plan: add fixtures::packs module (real pack-file/consumer builders + an on-disk git-anchored corpus harness) to src/lib.rs, migrate the 16 predicate cells to **rust:** fragments calling the real tool, regenerate specodelic/packs_props.rs + packs.check.json, extend the verify-specs recipe scope to compile+packs.
