---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-6-per-file-state-diagrams-tasks-1-7-render-2-3-corpus-2-4, pipeline-step:red]
---

RED: command='python3 -m unittest discover -s scripts -p test_graph_views.py' → 37 tests, 4 failures + 18 errors, all AttributeError on missing graph_views.render_states/OutOfScopeRefused/EdgesTsvInvalid or CLI 'usage: graph_views.py schema <export.json>' exit 2 — the missing states-view behavior, not setup breakage. Rust side: 'cargo test --test cli fixture_' → 5 passed (producer-side characterization pins over the checked-in fixture corpora; existing binary behavior, so green by design). Fixtures checked in: tests/fixtures/lint_dirty/dirty.md (guard_required finding verified live).
