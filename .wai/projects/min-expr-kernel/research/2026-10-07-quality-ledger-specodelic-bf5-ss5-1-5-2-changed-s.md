---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-bf5-ss5-1-5-2-opaque-kernel-binding-extraction-and-claim-carrier, pipeline-step:quality-ledger]
---

QUALITY LEDGER specodelic-bf5 §5.1-5.2.
Changed: src/compile.rs (ConstraintToml.binding Option + kernel_binding_of_row helper — opaque extraction, invariant-kind only, fence per kernel_cell_content precedent, None-skipped for pure widening); tests/kernel_binding.rs (new — 7 tests incl. the ordinary/arbitrary/empty hard gate, never-interpreted garbage fixture, no-column byte-identity, non-invariant scope); tests/cli/parse_misc.rs (CLI-surface test — binding verbatim in {stem}.toml via real compile invocation, minimal kernel pack per orphan_vocabulary remediation); openspec/changes/add-min-expr-kernel/tasks.md (§5.1/§5.2 checkboxes only).
Verified: cargo test --test kernel_binding 7/7; tests/cli binary 194 passed; just test clean ×2 consecutive; just test-smart 480 selected green; cargo fmt clean; just lint clean; pretender check green; ah check no issues; openspec validate --all --strict 25/25; just lint-specs green (no new advisory class).
Review: RO5U done — 0 critical, 0 high, 1 medium deferred out-of-ticket (kernel.binding vocab-orphan interplay — filed as bd issue), 4 low no-fix (correct-by-precedent/format-constraint).
Risks: (1) vocab-orphan interplay — files using the column need kernel-namespace pack until the bd-tracked decision lands; (2) clippy --all-targets -D warnings red on HEAD pre-existing (dead-code consts in tests/common/kernel_fixtures.rs) — not the just ci gate ('cargo clippy --'), but the orchestrator should know; (3) one flake observed once in citation_resolution::qualified_citation_without_its_file_in_the_invocation_is_unknown, clean on 3 subsequent runs.
Next: commit §5.1-5.2 (RED+GREEN one commit), then orchestrator verify.
