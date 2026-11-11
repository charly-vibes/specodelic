---
tags: [pipeline-run:tdd-ro5-2026-11-11-specodelic-x0wy-docs-layer-archive, pipeline-step:red]
---

RED: command='just docs-build' + test -f book/graded-evidence.html. expected failure=page absent. Observed: SUMMARY.md entry added first; mdbook build exit 0 but book/graded-evidence.html does not exist (mdbook 0.5.2 silently tolerates dangling SUMMARY targets — noted honestly; the missing deliverable IS the red). Green will be: page written, rendered html present, links resolve both ways.
