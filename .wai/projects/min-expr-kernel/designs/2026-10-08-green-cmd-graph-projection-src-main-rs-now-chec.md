---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-0zk-graph-projection-exit-code-semantics-f3-f8, pipeline-step:green]
---

GREEN: cmd_graph_projection (src/main.rs) now checks named roots when specs are empty — all roots unresolved (fs::metadata err) emits a labeled Output::failure envelope routed to stderr (stdout stays the raw projection channel) and exits 2, matching cmd_graph's JSON-mode refusal; violations from graph::build flip the exit to 1 with text unchanged (annotation rows ride along, D3). 4 pre-existing tests pinning old exit 0 updated to the new contract. Narrow 'cargo test --test cli' 269 passed. Fixture note: typing_violations carries 7 violations, not 6 as the meter said.
