---
tags: [pipeline-run:tdd-ro5-2026-10-06-add-min-expr-kernel-phase-3-specodelic-7ga-tier-b-kernel-core, pipeline-step:quality-ledger]
---

QUALITY LEDGER specodelic-7ga (phase 3, Tier B kernel core):

COMMITS (this session):
1. feat(kernel): Tier B kernel core — closed grammar + row-typing + **kernel:** marker opt-in (§3.1-3.2) — src/kernel.rs, src/lib.rs, src/compile.rs, tests/kernel_grammar.rs + RED/plan/orient artifacts.
2. feat(kernel): v0 atomics grounded per-atomic + three-valued status chain end-to-end (§3.3-3.4) — tests/kernel_grounding.rs, tests/kernel_status.rs, parser dot-split fix.
3. refactor(kernel): predicate-registry seam pinned — D8 decidability gate, widening purity (§3.5) — tests/kernel_registry.rs.
4. test(kernel): RO5U review fixes — row-typing and compile-leg edge cases pinned labeled — tests/kernel_grammar.rs stage-4 pins + tests/kernel_compile_edge.rs.

RO5U: PASS (converged stage 4; 0 critical/0 high open; F-1 HIGH + F-2 MEDIUM fixed during green; 4 low edge gaps pinned; 2 low perf notes deferred of record). Artifact: reviews/ro5u-phase3-7ga.md.

GATES: just test green (all suites; 4 new suites, 31 new tests); just lint-specs exit 0; uv run spk lint on kernel+compile delta specs = 0 issues; ah check = pre-existing debt only; openspec validate add-min-expr-kernel valid; cargo fmt/clippy clean on touched files; pretender file ratchets respected — src/model_check.rs UNTOUCHED (2282/2300 shrink-only), no pinned file grew past its ratchet.

PRE-EXISTING DEBTS (NOT this session's — listed, not fixed):
- ah check --changes add-min-expr-kernel: 23 no-toml findings + 1 overlay-conflict (compile/mid-span-occurrence-is-a-mention, from the scaffold commit).
- Contract TOMLs for change scenarios are authored at ARCHIVE time (phase 7); 5 slice-1 TOMLs stashed at /tmp/txo-contracts/*.toml.
- clippy --all-targets failures in 3 out-of-scope test files block just ci (not just test); plus 2 unused-import warnings in src/lint/mod.rs cfg(test) code — untouched files.
- Known pre-existing advisory linter warnings (single_root_reachable cross-file naturality rows, observability effects) — corpus facts, unchanged.

DESIGN CORRECTION of record: kernel opt-in = explicit **kernel:** marker in fragment position (marker-free whole-cell opt-in broke pure widening against the corpus's existing ∀-led prose cells — F-1).

KNOWN LIMITS (deferred of record): Proj evaluation scans morphism_values linearly per lookup; parse_kernel_str rebuilds schema::canonical() per call — termination holds (D8), perf tidies later; grammar-level widening (registered pack predicates entering the closed parse set) rides the widening law in a later change — the seam stores entries without editing this Revision's grammar.
