---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-9-docs-graphs-wiring-and-primer-tasks-3-1-3-4, pipeline-step:green]
---

GREEN: implemented docs-graphs recipe (justfile, bash body: graph edges/--json + lint --json + guide --schema --json envelopes from one cargo-run binary into target/docs-graphs, four views via graph_views.py + --view wiring into gitignored docs/src/views/, git-status-clean assertion scoped to OUT), .gitignore docs/src/views/, docs-build: docs-graphs dep, docs/src/graph-views.md with 4 {{#include views/*.md}} + philosophy sentence + format_revision note, SUMMARY Graph Views section, guide.rs additive topic (GRAPH_VIEWS_BODY const + slice_topic fallthrough; TOPICS append; existing count pin 8→9 per the test's own bump-on-append comment). Narrow results: cargo test --test ci_wiring 13/13 ok; cargo test --lib guide:: 23/23 ok; just docs-graphs → regenerated 4 views, git status scoped to views dir empty.
