---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-91kt-packs-quant-finance-md-pack-artifact, pipeline-step:red]
---

RED: command=spk lint over /var/tmp/qfp-91kt scratch git workspace (consumer fixtures desk-risk_model.md with quant.risk kind cells + ## Limits + capped_by; pricing-parity.md with uses [[quant.finance]]). Expected failure=orphan_vocabulary findings (both halves) — observed: (1) 'orphan vocabulary: pack-qualified token(s) quant.risk used by desk.risk_model, but no kind: profile pack in namespace quant is discovered — candidate pack quant.*'; (2) 'uses edge targets quant.finance, but no kind: profile pack with that id is discovered — candidate pack quant.finance' naming both remediations; (3) constraint_kind_closed/property_kind_closed closed-set findings since fiber kind quant.risk is not active. Failure is for the expected missing-pack behavior, not setup breakage.
