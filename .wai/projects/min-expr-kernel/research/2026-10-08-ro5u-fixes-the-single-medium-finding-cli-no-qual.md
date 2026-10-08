---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-efb-acset-instance-panic-on-member-path-label-qualified-link-targets, pipeline-step:fix-review]
---

RO5U-FIXES: the single Medium finding (CLI no-qualifier assertion scanned all TSV columns incl. reason text — would false-positive on a legit reason containing ' (') was fixed during the review pass: assertion now checks only the endpoint columns r[0]/r[3] (tests/cli/graph_views.rs member_path_link_graphs_without_panicking). Tests re-run: --test cli member_path 2 passed. No Critical/High findings existed; no deferrals.
