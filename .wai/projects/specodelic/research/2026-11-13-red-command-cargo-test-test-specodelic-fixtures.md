---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-lf3-migrate-specodelic-kinds-properties, pipeline-step:red]
---

RED: command=cargo test --test specodelic_fixtures expected failure=the migrated corpus rows' generators do not resolve to any fixture — the compiler would bind specodelic_spec_file(), spec_repo_with_traces_cycle(), kinds_rename_id() and the rest to the placeholder string, and the specodelic/kinds property predicates were still prose (check(row) == failed) with no executable fragment at all. The red was observed before the fixtures module existed: the corpus generators were unregistered, so the emitted spec_gen strategies carried literals.
