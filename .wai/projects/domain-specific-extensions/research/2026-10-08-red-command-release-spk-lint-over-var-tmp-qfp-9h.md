---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-9h3z-quant-pack-dogfood-probes, pipeline-step:red]
---

RED: command=release spk lint over /var/tmp/qfp-9h3z/main worktree fixtures with packs/quant-finance.md removed. Expected failure=labeled findings exist without the pack: (b) property_kind_closed {unit,law} + orphan_vocabulary prefix-derived quant.* on kind=quant.risk/quant.pricing cells, exit 1; (f) uses-edge orphan_vocabulary naming candidate pack quant.finance + both remediations exit 1, vocab-only variant prefix-derived quant.*; (d) base law floor law_cases fires identically; (a) dangling capped_by fires labeled total_refs naming row id+column, exit 1; (e) corpus diff without pack is the RED baseline for byte-identity; (c) Requires dep absent = declaration-only (fires nothing, exit 0).
