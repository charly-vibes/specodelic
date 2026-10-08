---
reviews: 2026-10-08-refactor-tidy-pass-folded-into-the-ratchet-driven.md
verdict: pass
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-6-per-file-state-diagrams-tasks-1-7-render-2-3-corpus-2-4, pipeline-step:ro5u-review]
---

## RO5U review — specodelic-gre.6 diff (per-file state diagrams)

**Verdict: pass** (with one toolchain note, not caused by this diff).

### High
None.

### Medium
1. **Refusal messages read the wrong envelope field** — `scripts/state_view.py`
   read `payload["error"]["message"]`, but genesis refusal envelopes ride the
   reason on `warnings[].message` (verified against live `spk graph --json`
   output over `tests/fixtures/graph/zero_file`). The refusal still fired
   (ok:false is the trigger) but the carried reason was generic.
   **Fix applied:** `_failure_message()` extracts from `warnings`; the
   `GRAPH_ZERO_FILES` / `LINT_FAILED_ENVELOPE` test fixtures now match the
   real envelope shape; a test asserts the refusal carries "no spec files
   found".
2. **Owning-file grouping needed the declared intent set** — an intent
   without typed-reference endpoints never appears in the edges TSV, and
   dotted intent ids make prefix-splitting ambiguous without the declared
   set (live failure: `c1.t1` grouped under itself in the typing_violations
   corpus). **Fix applied (additive Rust):** `GraphReport.intents` (sorted,
   deduped) in `src/graph.rs`; consumed by the scope gate; pinned by the
   single-intent fixture test in `tests/cli/graph_views.rs`.

### Low
1. D2's "fan-in counts distinct targets" is read as distinct
   `(source, field)` pairs per target — matches the graph-views spec's
   traceability fan-in scenario (k distinct sources); documented in
   `_fan_in`'s docstring.
2. Violation labels strip trailing `": "` — cosmetic.
3. `states_read_args` rejects artifact paths starting with `--` — acceptable.
4. The `no_transitions` marker uses the `:::violation` classDef, mirroring
   wiring's `no_wiring` precedent (D3: labeled, never silently clean).

### Toolchain note (pre-existing, not this diff)
clippy 1.98 emits `while_let_on_iterator` on `tests/cli/model_check.rs:1637`
— present at HEAD, file untouched by gre.6. `just ci`'s clippy leg fails on
it regardless of this change.

