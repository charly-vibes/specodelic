---
tags: [pipeline-run:tdd-ro5-2026-11-24-claim-coverage, pipeline-step:green]
---

REFACTOR: tidy pass over the green implementation — no behavior change, no expansion. (1) collapsed the nested if-let in resolve_claim (clippy collapsible_if, -D warnings); (2) cargo fmt over the new files. Tidy checklist from tasks.md verified in place: the fold order lives in ONE named function (claim_coverage::class_rank, tasks 2c.3); the scope statement is ONE named constant (claim_coverage::SCOPE_STATEMENT) rendered by both views (tasks 2d.3); claims list in declaration order from the parsed spec, no hash-map order leaks (tasks 1.3, byte-stability test pins it); no new enum value, no boolean flag — AuditClass reused as-is (tasks 3.3); counts ride the verdict line beside the evidence counts (tasks 4.3); residuals carry the existing class set (tasks 5.3). Verification: cargo clippy --all-targets -D warnings exit 0; cargo fmt --check clean; cargo test --test check_claim_coverage 12/12.
