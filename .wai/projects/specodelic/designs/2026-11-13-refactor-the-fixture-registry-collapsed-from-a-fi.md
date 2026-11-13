---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-lf3-migrate-specodelic-kinds-properties, pipeline-step:refactor]
---

REFACTOR: the fixture registry collapsed from a five-line struct literal per entry to a flat (name, builder) table (type Entry), the per-builder doc comments were re-wrapped, and the fixtures' property-row generator cell was shortened to a single shared name — behaviour-preserving (the artifacts are byte-identical after the refactor). The reason is the structural-quality ratchet: the new module pushed src/lib.rs to 2626 lines, over the pretender file_lines_max of 2300, so the registry shape had to shrink rather than the threshold move. src/lib.rs is now 2286 lines and pretender check reports it green.
