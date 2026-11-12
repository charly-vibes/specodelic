---
tags: [pipeline-run:tdd-ro5-2026-11-12-specodelic-q7uo-compile-pipe-unescape, pipeline-step:green]
---

GREEN: one helper unescape_cell_pipes (literal two-char \\| -> |, the exact inverse of the cell rule, identity otherwise) in src/compile.rs, applied at the ONLY two production sites: Fragment.text in fragment_of_strict (so _props.rs via blocks_for_row, IR invariants -> .tla manifest + model_check scratch crate, audit/conform evidence all see the same unescaped string) and the expr field in constraints_doc (toml carries cell content, not markdown transport). src/model_check.rs needed NO change — the scratch crate consumes IR. src/packs.rs untouched. Narrow: 2/2 fragment_unescape tests pass; full --test cli 312 passed; --lib 439 passed. Test-only deviation of record: the //predicate metadata comments keep quoting the raw cell verbatim (their documenting job) — the no-backslash assertion is scoped to compiled (non-comment) lines.
