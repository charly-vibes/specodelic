---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-36t-ss4-3-shared-fixture-module-named-case-law-promotion-path, pipeline-step:green]
---

GREEN specodelic-36t: created tests/common/kernel_corpora.rs — verbatim move of the corpus constants from kernel_grounding.rs (CYCLIC/DANGLING/DUPLICATES/CHAIN) and kernel_status.rs (CLEAN/BROKEN/UNKNOWABLE/ABSORBING), registered in tests/common/mod.rs; both tests import via the #[path=common/mod.rs] shim (kernel_agreement.rs convention). Narrow command green: identical 18 test names (3 agreement + 8 grounding + 7 status), all pass — behavior-preserving (ids/fixture text/expected statuses untouched). kernel_agreement.rs not edited; rust interim gate at full strength. cargo fmt --check clean. Near-duplicate shapes vs kernel_fixtures.rs deliberately not unified (unification would change assertion meaning); documented in the module header.
