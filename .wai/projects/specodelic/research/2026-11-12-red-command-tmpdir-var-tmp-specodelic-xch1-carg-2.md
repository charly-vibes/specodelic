---
tags: [pipeline-run:tdd-ro5-2026-11-12-specodelic-q7uo-compile-pipe-unescape, pipeline-step:red]
---

RED: command='TMPDIR=/var/tmp/specodelic-xch1 cargo test --test cli fragment_unescape' — escaped_pipes_reach_emitted_rust_unescaped_and_verify FAILED as expected: emitted frag_pipe_props.rs contains literal backslash bytes (v0.is_empty() \|\| v0 == "skip"), so verify would fail to compile the artifact and model-check's scratch crate would hit fragment_compile. Byte-stability pin (fragments_without_escape_emit_byte_identical_artifacts) PASSED on pre-change code against pinned pre-change artifact bytes — proving the pin is the identity floor.
