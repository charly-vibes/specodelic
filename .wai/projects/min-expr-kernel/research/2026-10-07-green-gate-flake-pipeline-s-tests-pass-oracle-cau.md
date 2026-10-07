---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-bf5-ss5-1-5-2-opaque-kernel-binding-extraction-and-claim-carrier, pipeline-step:green]
---

GREEN gate flake: pipeline's tests-pass oracle caught citation_resolution::qualified_citation_without_its_file_in_the_invocation_is_unknown failing once (exit 1 vs 0). Isolated run: ok ×2; full 'just test' clean ×2 consecutively after. The test exercises qualified-citation model-check exit codes with no kernel.binding column — unrelated to the diff; diagnosed as a parallel/order flake in the shared cli test binary, not a regression.
