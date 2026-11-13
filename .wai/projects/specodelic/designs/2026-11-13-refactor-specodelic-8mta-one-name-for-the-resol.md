---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-8mta-resolve-check-kernel-id-literals, pipeline-step:refactor]
---

REFACTOR (specodelic-8mta): one name for the resolution test — KernelEnv::declares(id) wraps the BTreeSet lookup and is used by both call sites (the Term::Id arm of term_value and the reachable target check), so the resolution rule reads the same in both places. The declared set is built from the SAME kind_index walk that fills the per-object membership maps (one walk, no second corpus scan; the earlier draft called kind_index twice). The unresolved-literal reason lives in ONE free function (unresolved_id_reason) shared by the comparison path and the reachable target path — no duplicated reason text. No behavior change: cargo test (all 52 targets) green, cargo clippy -- -D warnings green.
