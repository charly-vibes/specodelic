---
tags: [pipeline-step:select-ticket, pipeline-run:epic-orchestrator-2026-11-13-specodelic-lf3-migrate-packs-properties, pipeline-run:epic-orchestrator-2026-11-13-specodelic-lf3-migrate-compile-properties]
---

TICKET: specodelic-lf3 run 3 (owner-authorized scoped-first, next unit) - scope: openspec/specs/packs/spec.md (16 property rows, 0 law rows; packs_props.rs currently 17 todo_predicate!). The u7kc seams are landed and proven on the compile file (35/35 migrated, verify-specs gate wired scoped to compile, properties-only verdict). Target: migrate all 16 packs rows to **rust:** predicates calling the real tool via the fixtures registry (builders for the corpus/pack fixtures the generator cells name), regenerate packs_props.rs + packs.check.json, extend the verify-specs recipe scope to [compile, packs], prove the gate still exits 1 on a broken fragment; any honestly-inexpressible row is reported and left todo (file then out of scope until resolved). No other corpus files.
