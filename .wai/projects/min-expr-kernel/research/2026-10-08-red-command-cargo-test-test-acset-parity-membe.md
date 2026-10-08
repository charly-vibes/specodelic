---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-efb-acset-instance-panic-on-member-path-label-qualified-link-targets, pipeline-step:red]
---

RED: command='cargo test --test acset_parity member && cargo test --test cli member_path' expected failure=the member-path panic. Observed: member_path_link_projects_canonical_edge and dangling_member_path_stays_dangling_on_labeled_path panic at src/acset/instance.rs:126:28 'uninterned node id: v.row (deep)'; both CLI tests (tests/cli/graph_views.rs) see exit 101 with the same panic on stderr. Fixtures tests/fixtures/member_path (resolving) and tests/fixtures/member_path_dangling (v.norow undefined). Failure is the expected missing behavior, not setup breakage.
