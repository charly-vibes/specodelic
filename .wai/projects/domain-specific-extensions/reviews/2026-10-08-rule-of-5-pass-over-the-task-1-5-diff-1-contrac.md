---
reviews: 2026-10-08-refactor-specodelic-gre-3-task-1-5-none-needed.md
verdict: pass
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-3-native-dot-mermaid-projections-task-1-5, pipeline-step:ro5u-review]
---

rule-of-5 pass over the task 1.5 diff: (1) contract — flags wired per D8 (raw text, overrides --json/--human, byte-stable, zero crates, no --render), grammar styles mapped guard=dashed emits=penwidth=2 traces_to/derives_from=dotted, violations red dashed annotated, dangling red dashed notes, canonical ids via the shared edge_projection core; (2) format consistency — zero-file exit-0-empty follows the landed --format edges contract (1.1), envelope graph keeps its specodelic-6pi exit-2 path untouched; (3) tests — RED observed (clap invalid value, exit 2), 13 cases green, pins characterize the deterministic sort order; (4) seams — hardening applied: render_mermaid now names.get()s endpoints instead of indexing (a pure renderer must not panic on a degenerate row); dedup keys on the rendered line, styles impossible to collide; (5) simplicity — one projection core, no second normalization path, dead struct removed. Findings fixed in-commit; tests re-run 64/64 green.
