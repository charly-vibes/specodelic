---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-9-docs-graphs-wiring-and-primer-tasks-3-1-3-4, pipeline-step:refactor]
---

REFACTOR: extracted slice_topic(source, topic) helper so GUIDE_MD and the appended GRAPH_VIEWS_BODY share one slicing path (fill_placeholders applied uniformly). No behavior change. Verified pre-existing clippy warning (tests/cli/model_check.rs:1640 while_let_on_iterator) is untouched-by-me and outside the ci clippy gate (cargo clippy -- -D warnings covers lib+bins). cargo fmt clean; python unittest scripts 97 OK.
