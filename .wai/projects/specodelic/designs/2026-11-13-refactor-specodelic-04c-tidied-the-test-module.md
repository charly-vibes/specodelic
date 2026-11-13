---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-04c-kernel-vocabulary-pack, pipeline-step:refactor]
---

REFACTOR (specodelic-04c): tidied the test module without behavior change — extracted the duplicated kernel-pack lookup into a kernel_pack(json) helper, added a repo_path(rel) helper so the corpus and pack paths resolve from CARGO_MANIFEST_DIR instead of a relative CWD, and corrected the header comment that still named the pre-pivot landing site (openspec/specs/kernel-pack/spec.md -> packs/kernel-pack.md). No behavior expansion, no unrelated cleanup. cargo fmt clean; cargo test --test kernel_pack_vocabulary 4 passed after the tidy.
