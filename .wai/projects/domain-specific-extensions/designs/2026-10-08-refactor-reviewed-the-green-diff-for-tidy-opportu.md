---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-4-wiring-view-task-1-6, pipeline-step:refactor]
---

REFACTOR: reviewed the GREEN diff for tidy opportunities — wiring_projection already shares the kind-index/ownership path with edge_projection; renderers are three small pure fns over one WiringRow type (no duplication worth extracting: the no_wiring branch is format-specific by design); naming is consistent with the 1.5 renderers (render_wiring_*). One structural improvement made during GREEN: renderer byte pins live at unit level (src/graph.rs) where the pure functions are testable without the CLI, and CLI tests pin flag composition only — this also kept tests/cli/parse_misc.rs under the pretender shrink-only file_lines ratchet (2300) without raising any threshold. No behavior change; cargo test wiring_/graph:: green after review.
