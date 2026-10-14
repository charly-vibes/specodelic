---
tags: [pipeline-run:tdd-ro5-2026-10-14-specodelic-kjzh-svc-contracts-pack, pipeline-step:red]
---

RED: command='target/release/specodelic lint /var/tmp/kjzh/red-corpus' (scratch corpus /var/tmp/kjzh/red-corpus/consumer.md: valid intent file whose Properties row uses kind svc.compat with no pack present). Expected failure=pack_shape-era baseline: property_kind_closed (svc.compat outside {unit,law}) + orphan_vocabulary naming candidate namespace 'svc' and both remediations, exit 1 — exactly bundle evidence E10c. Observed: both findings present, exit 1. This is the honest baseline the pack must turn into advisory-only acceptance (E10b) with zero byte drift for corpora using none of the vocabulary (E11).
