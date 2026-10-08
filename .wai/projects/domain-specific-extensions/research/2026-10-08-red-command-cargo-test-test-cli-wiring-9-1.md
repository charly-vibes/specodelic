---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-4-wiring-view-task-1-6, pipeline-step:red]
---

RED: command='cargo test --test cli wiring_' → 9/11 failed with exit code 2, observed 'error: unexpected argument --view found' (flag absent, the intended reason). 2 vacuous passes: wiring_tsv_is_byte_identical_on_rerun (both runs fail identically) and wiring_view_requires_a_format (clap already refuses). Fixtures: write_wiring_producer/write_wiring_consumer (cross-file satisfies via extension_point contract rows + self-loop cell), write_self_loop_only_spec.
