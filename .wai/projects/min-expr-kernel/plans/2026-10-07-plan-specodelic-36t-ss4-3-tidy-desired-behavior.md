---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-36t-ss4-3-shared-fixture-module-named-case-law-promotion-path, pipeline-step:plan]
---

PLAN specodelic-36t §4.3 TIDY — DESIRED BEHAVIOR: single home for the kernel fixture corpora under tests/common/; kernel_grounding.rs and kernel_status.rs import their corpus constants instead of defining them; zero change to fixture text, ids, assertions, or statuses. OUT OF SCOPE: tests/kernel_agreement.rs (36n property — untouched), unifying agreement-vs-grounding corpus ids (would change assertion meaning), src/, tests/cli/*, openspec/specs/, .espectacular/. TEST CASES (characterization-first, pure move): baseline 
running 3 tests
test py_seat_is_honest_pending_l8l ... ok
test supporting_backends_agree_on_every_cell ... ok
test every_cell_matches_its_expected_status_on_the_rust_backend ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 8 tests
test every_v0_atomic_has_a_grounding_entry ... ok
test comparisons_agree_with_bounded_evaluation ... ok
test reachable_agrees_with_forward_closure ... ok
test unknown_seed_is_unknown_never_a_fabricated_verdict ... ok
test bounded_quantifiers_agree_over_the_instance ... ok
test resolves_agrees_with_the_dangling_report ... ok
test unique_agrees_with_injectivity_of_defined_values ... ok
test acyclic_agrees_with_graph_cycle_detection ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 7 tests
test counterexample_claim_travels_the_whole_chain ... ok
test unknown_claim_travels_the_whole_chain_honestly ... ok
test unknown_absorbs_in_composites_never_coerced_to_pass ... ok
test empty_domain_quantifiers_decide_honestly ... ok
test counterexample_dominates_unknown_in_composites ... ok
test verified_claim_travels_the_whole_chain ... ok
test negation_swaps_verified_and_counterexample_keeps_unknown ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s captured before the move; after the move the same command must pass with identical test names and identical pass/fail set (diff of test-name lists empty). NEW SHARED MODULE: tests/common/kernel_corpora.rs (file-header doc, Purpose/Responsibilities/Rationale) holding CYCLIC/DANGLING/DUPLICATES/CHAIN + CLEAN/BROKEN/UNKNOWABLE/ABSORBING verbatim; registered in tests/common/mod.rs. DOC: promotion contract section in add-py-fragment-emission/design.md (deferred-of-record note per tasks.md 4.3: l8l promotes the property to a law_requires_cases-shaped Property row; cases shrink-only, never dropped or commented). NARROW: cargo test --test kernel_grounding --test kernel_status --test kernel_agreement. FULL: just test && ah check && openspec validate --all --strict && pretender check. FILES TOUCHED: tests/common/kernel_corpora.rs (new), tests/common/mod.rs, tests/kernel_grounding.rs, tests/kernel_status.rs, openspec/changes/add-py-fragment-emission/design.md.
