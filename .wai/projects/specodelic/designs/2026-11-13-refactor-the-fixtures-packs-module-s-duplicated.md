---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-lf3-migrate-packs-properties, pipeline-step:refactor]
---

REFACTOR: the fixtures::packs module's duplicated file templates collapsed — pack_file_with_kinds and pack_file_in_state now share one private pack_file_text(id, revision, extra_kinds, model) with single_state_model/lifecycle_model supplying the Model section, and consumer_spec/fiber_kind_consumer now share one private consumer_text(id, uses, kind, expr); five byte-identical registry builders (build_pack_file_alone/enabled/four_layer/lifecycle) collapsed onto build_pack_file, and the consumer builders build their uses link through the single uses_link() seam instead of literal [[...]] text. Behaviour-preserving: the regenerated artifacts (specodelic/packs_props.rs, specodelic/packs.check.json) are byte-identical after the refactor, cargo test is green, and just verify-specs still exits 0.
