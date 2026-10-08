---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-efb-acset-instance-panic-on-member-path-label-qualified-link-targets, pipeline-step:orient]
---

ORIENT: specodelic-efb — resolve() member arm returns label-qualified 'file.row (member)' (pinned by specodelic-njh unit tests); graph::build records that raw target as the edge 'to', but Instance::from_specs interns only defined ids (file.row) so intern_of panics at instance.rs:126 on plain 'spk graph'. Single production caller of resolve() is resolve_link (graph.rs:273) — the shared derivation both consumers see. Fix decision: normalize via graph::canonical_id at resolve_link (single derivation point), not at resolve() (would churn pinned contract for no benefit) and not only in from_specs (would break adapter_graph_equivalent parity vs raw build edges).
