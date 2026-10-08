---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-5-guide-schema-json-and-schema-diagrams-tasks-2-1-2-3, pipeline-step:green]
---

GREEN (2.1/2.2): src/guide.rs gained pub fn value_set_payload() (format_revision, intent/constraint/property kinds, row_shapes; NO reference typing — D4) and pub fn schema_export(&Schema) -> serde_json::Value (schema_version 1, format_revision, sorted objects, morphisms sorted (source,name), refinements sorted (side,kind), source_rule strings, endo_acyclic bool|null; shared exporter accepts any &Schema so constructed fixtures use production serialization). src/main.rs: new Guide { --schema } subcommand, cmd_guide emits both payloads via genesis Output::emit. REFERENCE_TYPING untouched, outside both payloads. Narrow: cargo test --lib guide:: 20 passed; just test-smart all green (after clearing stale /tmp/specodelic-verify scratch cache that had filled the tmpfs quota — environmental, unrelated).
