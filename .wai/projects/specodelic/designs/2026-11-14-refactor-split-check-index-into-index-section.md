---
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-oeuz-gate-the-format-summary-prose-across-docs-copies, pipeline-step:refactor]
---

REFACTOR: split _check_index into _index_section / _table_layer_labels / _check_layer_labels — the single function had cyclomatic 18 (pretender red); each helper is now well under the ratchet, tests and the real-tree gate stay green (6/6 unittest; gate exit 0; pretender check exit 0).
