---
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-3a8-decouple-dual-format-metadata-into-an-openspec-adapter, pipeline-step:green]
---

GREEN: added src/openspec_adapter.rs (DualFormat + from_source + insert_into), pub mod in lib.rs; removed the six dual-format fields and SectionCtx.dual_section/capture_dual_body/on_h2 markers from src/spec.rs; src/parse.rs gains parse_file_with_source (parse_file delegates); cmd_parse merges the adapter keys into the Spec JSON so the external envelope keeps has_added_requirements/has_requirements_section/added_requirements_body/has_modified_requirements/modified_requirements_body/requirements_body. Verified: cargo test --test spec_parse 10 passed; cargo test --test cli parse_json_keeps_dual_format_fields passed; cargo test --lib 437 passed; spk lint openspec 68 files 0 issues (unchanged).
