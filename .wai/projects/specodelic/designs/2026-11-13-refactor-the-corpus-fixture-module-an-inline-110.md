---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-3632-lint-revision-seams-for-the-four-blocked-rows, pipeline-step:refactor]
---

REFACTOR: the corpus fixture module (an inline 1100-line block in src/lib.rs) moved verbatim to src/fixtures/specs.rs in its own commit (ec2d4af) — the module path, every item and every call site are unchanged, so the move is behavior-preserving and the feature diff reads against a stable crate root. The new lint seams live in their own modules (src/lint/row_shape.rs, src/lint/revision.rs) rather than inside src/lint/mod.rs. Tests stay green after the move.
