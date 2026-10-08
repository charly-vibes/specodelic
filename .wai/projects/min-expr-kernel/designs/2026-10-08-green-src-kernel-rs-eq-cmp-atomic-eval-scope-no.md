---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-7gh-claim-records-carry-reason-when-not-verified-f4-f6, pipeline-step:green]
---

GREEN: src/kernel.rs — eq/cmp/atomic/eval_scope now return (ThreeValued, Option<String>) (Kleene-with-reasons, first unknown cause wins, mirrors citation_corpus::evaluate); evaluate() keeps its signature, new evaluate_reasoned() returns the tuple; evaluate_corpus_claims native arm attaches reason: unknown → traversal/eq cause naming the bad id (QueryError display), counterexample → self-naming reason, verified → None. Verdict semantics/exit codes untouched. Narrow tests green; just test-smart (492 selected) exit 0.
