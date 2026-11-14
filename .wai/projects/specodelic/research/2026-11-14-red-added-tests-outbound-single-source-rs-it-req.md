---
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-mlc-consolidate-the-outbound-leaf-carve-out-into-one-normative-statement, pipeline-step:red]
---

RED: added tests/outbound_single_source.rs. It requires the specodelic.md marker 'Outbound-leaf carve-out (Revision 22)' inside ### Reference Typing, and at most one prose mention of 'outbound' outside specodelic.md (identifier spellings ignored, linter-failure_shape's model-edge sense exempt), and that mention must name specodelic. Run cargo test --test outbound_single_source fails first on the missing marker; the current pack/observability/linter files also restate the rule multiple times. Red confirmed.
