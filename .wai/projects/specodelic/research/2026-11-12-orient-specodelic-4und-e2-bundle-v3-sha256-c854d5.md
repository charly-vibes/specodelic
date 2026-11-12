---
tags: [pipeline-run:tdd-ro5-2026-11-12-specodelic-4und-kernel-coverage-of-fragments, pipeline-step:orient]
---

ORIENT specodelic-4und E2: bundle v3 sha256 c854d590…8cbab6 extracted /var/tmp/4und-bundle (harness 15 py files; NO mutB*.py, k08.py exists — brief list deviates). Kernel IS landed (src/kernel.rs: closed atomics {resolves,unique,acyclic,reachable,==,comparisons,∀,∃,∧,¬}; reachable() = single-morphism forward_closure over the REFERENCE graph (traces_to/derives_from/from/to/…) — no State→State morphism, so no transition-graph transitive reachability in v0). evaluate_corpus_claims runs in orchestrate; aggregate_required_claims: refuted kernel claim → CounterexampleFound. Helper baselines from bundle logs: k04 a1 killed/a2 survived/a3 survived; k08 w1-w3 killed. Probe plan: re-express one-hop forbidden-edge + forbidden-name-set as **kernel:** claims, run spk orchestrate on scratch specs under /var/tmp/4und-probes, compare exit codes with helper outcomes.
