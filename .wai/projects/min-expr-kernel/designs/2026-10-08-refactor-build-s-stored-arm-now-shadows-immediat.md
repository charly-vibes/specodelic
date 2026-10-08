---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-efb-acset-instance-panic-on-member-path-label-qualified-link-targets, pipeline-step:refactor]
---

REFACTOR: build's Stored arm now shadows  immediately before record_edge — makes the ordering explicit (typing_violation above consults the raw spelling; the record canonicalizes after the guard). No behavior change; acset_parity (18), cli member_path (2), lib graph (16) still green.
