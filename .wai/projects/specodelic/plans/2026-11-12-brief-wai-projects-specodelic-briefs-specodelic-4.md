---
tags: [pipeline-step:write-brief, pipeline-run:epic-orchestrator-2026-11-12-specodelic-q7uo-compile-pipe-unescape, pipeline-run:epic-orchestrator-2026-11-12-specodelic-q7uo-compile-pipe-unescape, pipeline-step:write-brief]
---

BRIEF: .wai/projects/specodelic/briefs/specodelic-q7uo.md - RED: fixture with escaped pipe in an executable fragment asserts literal || in emitted _props.rs (compiles as Rust) + byte-stability for \|-free fragments; GREEN: single unescape helper at fragment extraction so all four consumers (props.rs, toml, tla comment, scratch crate) get unescaped text; packs.rs:154-161 forbidden; targeted tests only, cargo fmt before commit, orchestrator owns full gates.
