# Change: Doctor sweet-spot advice profiler

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
- Affected code: `src/doctor.rs` (advice subcommand/flag), docs
  (decision-guide cross-link).
