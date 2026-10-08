---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-91kt-packs-quant-finance-md-pack-artifact, pipeline-step:fix-review]
---

RO5U review of packs/quant-finance.md (5 lenses: contract-conformance, hygiene, self-hosting, precedent-fidelity, falsifiability)

CRITICAL: none.

HIGH: none.

MEDIUM:
- M1 (contract-conformance): the delta's limits_section_shape constraint requires the Limits-row kind column "closed to risk, pricing — a per-pack closed set", but the artifact states the closure only implicitly (row_shape column + dotted Kinds glosses); a reader cannot derive that a Limits row's kind label `risk` corresponds to declared kind `quant.risk`. Evidence: packs/quant-finance.md ## Sections row vs delta constraint limits_section_shape. Fix: one prose sentence in the pack gloss mapping the row-level labels to the pack kinds (the mechanism never checks row-kind cells, so prose is the right home).

LOW:
- L1 (precedent-fidelity): pack gloss mentions the vendor-history "restraint" framing — matches design.md context; no action.
- L2 (hygiene): `bound`/`unit` appear only as column names in row_shape and glosses — columns are not scanned vocabulary (D2); no action.
- L3 (self-hosting): guards cite the self-hosted [[quant.finance.vocab_declared]] — matches the bioimage precedent; the delta's pack_file_declared/limits_section_shape citations live in the change delta, not the artifact; no action.
- L4 (falsifiability): p_pack_shape generator/predicate mirror bioimage; honest-empty semantics quoted in both checker cells; no action.

Verdict: 1 actionable finding (M1), fixed in the same pass; tests re-run after fix.

