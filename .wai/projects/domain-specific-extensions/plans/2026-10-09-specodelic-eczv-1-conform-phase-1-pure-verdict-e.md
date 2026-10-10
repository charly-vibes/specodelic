---
tags: [pipeline-run:tdd-ro5-2026-10-09-specodelic-eczv-1-conform-phase-1-verdict-engine-core, pipeline-step:plan]
---

specodelic-eczv.1 conform phase 1 — pure verdict engine, no CLI.

DESIRED BEHAVIOR (design D1-D4, delta spec):
- Closed five-value taxonomy Verdict {permitted, forbidden, underspecified, unknown, unsupported}.
- classify_claims(spec) partitions invariant-kind Constraints into required (rust fragment / kernel / citation = executable), unchecked (prose-only), unsupported (py/ts fragment: kind exists, no emitter this run) — reusing compile::ModelIr machinery + fragment_of_strict, no new status vocabulary.
- classify_traces(spec, corpus, closed_world): mechanical name identity after trimming (D3); walk setup.state -> declared transitions; undeclared names -> uncovered (underspecified open-world / forbidden closed-world, reason closed_world_forbidden_recorded); not-enabled step or observed-state mismatch -> positive contradiction forbidden in ANY mode (reason contradiction_forbidden_any_mode, names executable covering claim ids); prose covering claim -> unknown naming it; py/ts covering claim -> unsupported naming the kind; else permitted. Every trace exactly one verdict (totality).

OUT OF SCOPE: CLI wiring (phase 4), report schema/envelope/digest (phase 2), input gate + corpus strict validation (phase 3), laws (OQ2), scratch-crate fragment evaluation.

TEST CASES (tests/conform_classification.rs, fixture spec demo.conform + JSONL corpus):
1.1 executable contradiction -> forbidden naming claim id; prose covering -> unknown naming claim; undeclared transition -> underspecified; py covering -> unsupported naming 'py'.
1.3 same contradiction with closed_world=false -> forbidden + closed_world:false + contradiction_forbidden_any_mode; uncovered w/ closed_world=true -> forbidden + closed_world:true + closed_world_forbidden_recorded; uncovered w/o -> underspecified (uncovered_trace_never_forbidden).
1.4 taxonomy_is_total_and_distinct: five-value fixture corpus, each scenario exactly one verdict, all five values present.

NARROW TEST CMD: TMPDIR=/var/tmp/specodelic-eczv just test-smart
FULL VERIFY: TMPDIR=/var/tmp/specodelic-eczv cargo test --test conform_classification && just fmt && cargo clippy --all-targets -- -D warnings && just pretender-check

FILES: src/conform.rs (new), src/lib.rs (+1 pub mod conform line), tests/conform_classification.rs (new), openspec/changes/add-conform/tasks.md (1.x checkoffs only).
