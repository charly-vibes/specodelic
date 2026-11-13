---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-9o1h-compose-kernel-outcomes-into-the-citation-algebra, pipeline-step:refactor]
---

REFACTOR (specodelic-9o1h): the kernel-evidence branch of Resolver::resolve_target built its optional reason with a nested if/else; bool::then states the same rule in one line (only an unknown rides a reason, exactly like the exec leg). Behavior unchanged — every published reason stays byte-identical, verified by the full cli + citation_algebra suites (332 + 12 passed) and pretender check (exit 0). No other tidy was needed: the evidence type (citation_corpus::ClaimEvidence) is neutral so the algebra stays evaluator-agnostic, the single conversion lives in kernel.rs (kernel owns its own shape), and the exec/kernel disjointness is enforced by construction in extract_model_ir plus the exec map's guard_kernel filter.
