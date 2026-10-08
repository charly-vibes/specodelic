---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-9-docs-graphs-wiring-and-primer-tasks-3-1-3-4, pipeline-step:red]
---

RED: command='cargo test --test ci_wiring' → 5 new tests failed for the expected missing artifacts (no docs-graphs recipe, no gitignore entry, no docs page, no SUMMARY link, no docs-build dep); command='cargo test --lib guide::' → graph_views_topic_is_appended_last + graph_views_topic_documents_taxonomy_flags_and_recipes failed because the topic does not exist yet. Failures are missing-behavior, not setup breakage (all 21 existing guide tests + 8 existing ci_wiring tests passed).
