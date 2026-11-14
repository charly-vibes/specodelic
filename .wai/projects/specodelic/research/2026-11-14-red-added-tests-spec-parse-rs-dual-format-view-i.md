---
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-3a8-decouple-dual-format-metadata-into-an-openspec-adapter, pipeline-step:red]
---

RED: added tests/spec_parse.rs::dual_format_view_is_derived_from_source importing specodelic::openspec_adapter::DualFormat (module missing -> E0432 compile red) and a CLI parity pin tests/cli/parse_misc.rs::parse_json_keeps_dual_format_fields asserting the six keys survive remove-and-merge. Command cargo test --test spec_parse dual_format_view_is_derived_from_source fails with unresolved import, as expected for the absent adapter.
