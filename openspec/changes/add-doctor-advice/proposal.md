# Change: Doctor sweet-spot advice profiler

> **PARKED (2026-10-25, demand-gated — simplification review):** the advice
> content is decision-guide knowledge parameterized by numbers, and those
> numbers are exactly the evidence-audit aggregate (add-evidence-audit).
> Near-term home: the Wave-1 decision-guide detection-class table
> (specodelic-f046). Re-activate as doctor output only if the docs prove
> insufficient and the audit aggregate exists to profile against.

## Why

The mistral kata study located specodelic's value in a narrow sweet spot —
small, data-free, control-dominated state machines — and measured that
outside it simpler methods (oracle + differential/property testing) deliver
equal or better evidence at lower cost. Nothing in the CLI tells an author
which side of that line their file is on before they invest.

## What Changes

- `spk doctor --advice`: per-file profiler over concrete, computable signals —
  model states per data dimension (product growth), guards citing claims
  with no executable fragment (`**rust:**`/`**kernel:**`), fan-in
  concentration — emitting routing guidance derived from the study's
  sweet-spot verdict.
- Advisory-only: advice is on the warnings/information channel, exit 0,
  never gates, never modifies the corpus.
- No new checking machinery: signals are computed from already-parsed spec
  data.
- The doctor.layer sidecar extension is authored at implementation
  (fail-closed at archive).
- Provenance: catitas item B1; conversation `mistral-work/923c311c…`
  ("Example-Driven Design Evaluation", final verdict).

## Impact

- Affected specs: `doctor` (ADDED requirement; existing doctor requirements
  unmodified).
- Dependencies: none hard; the advice text cites the same sweet-spot
  evidence the Wave 1 decision-guide table (specodelic-f046) documents —
  landing the docs first avoids citing an empty page.
- Affected code: `src/doctor.rs` (advice subcommand/flag), docs
  (decision-guide cross-link).
