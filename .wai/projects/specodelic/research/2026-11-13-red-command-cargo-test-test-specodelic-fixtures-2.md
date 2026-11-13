---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-3632-lint-revision-seams-for-the-four-blocked-rows, pipeline-step:red]
---

RED: command=cargo test --test specodelic_fixtures expected failure=the new seams do not exist yet — error[E0433]: cannot find 'revision' in 'lint' (specodelic::lint::revision::check_pair / ChangeKind) at tests/specodelic_fixtures.rs:496,509,512,526; the fixture-backed assertions (the extra Constraint column / extra State field / silent subkind growth / shrunk id-set / declared growth) and the no-row-left-on-the-placeholder test are red for the same reason: no rule linter.row_shape_closed, no fixture seam, and the four corpus rows still carry todo_predicate!.
