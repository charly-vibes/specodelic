---
tags: [pipeline-run:tdd-ro5-2026-11-12-specodelic-q7uo-compile-pipe-unescape, pipeline-step:orient]
---

ORIENT: q7uo — parser (spec.rs::parse_row) splits on | outside backticks only; inside spans text is literal so a fragment cell keeps \| bytes. Fragment text is produced in ONE place: compile.rs::fragment_of_strict (Fragment.text). Consumers: properties_to_proptest via fragment_of (blocks_for_row), extract_model_ir invariants -> model_to_tla manifest + model_check scratch_run_source, audit evidence units, conform classification (lang only). constraints_doc/toml emits raw expr cell. Fix: one unescape_cell_pipes helper (literal \\| -> |, mirrors packs.rs:155 ESC precedent) applied at fragment text production + constraints_doc expr; model_check.rs needs no change. Test home: new tests/cli/fragment_unescape.rs (verify runs cargo test on _props.rs = honest compiled-Rust check; model-check scratch crate = second). Pretender: global maxima only, no per-file pins.
