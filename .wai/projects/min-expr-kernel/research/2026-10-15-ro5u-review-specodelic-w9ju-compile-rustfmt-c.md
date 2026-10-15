---
tags: [pipeline-run:tdd-ro5-2026-10-15-specodelic-w9ju-compile-rustfmt-clean-props-rs-emission-gh-15, pipeline-step:fix-review]
---

# RO5U review — specodelic-w9ju (compile rustfmt-clean emission)

- **Critical**: none.
- **High**: none blocking.
- **Medium**:
  1. `src/compile.rs` `rustfmt_clean` — write-stdin-then-wait pattern can deadlock for generated sources larger than the pipe buffer if rustfmt ever emits before stdin EOF; fix per the `std::process::Child` docs pattern (write stdin from a spawned thread, then `wait_with_output`).
  2. Envelope payload `props` field (`corpus.rs`) carries the pre-fmt string while disk bytes are formatted — cosmetic envelope/disk drift; document rather than change (payload is the semantic artifact, disk is the fmt-clean artifact).
- **Low**:
  3. Fallback warning names path + reason — good as-is.
  4. `rustfmt` resolved from PATH only — by-design graceful fallback.

