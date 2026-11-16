# Change: Oracle expectations and tiered verdict accounting for conform

## Why

`spk conform` classifies external traces with its own closed verdict
taxonomy, but until the expectation and advisory work landed an oracle had
no way to declare intent: a negative probe that the compiled Model correctly
refuses was indistinguishable from a model disagreement and failed the run,
and prose-only covering evidence was counted as if it were a hard `unknown`
verdict. `src/conform.rs` now implements both behaviors — the optional
`expect` declaration (gh-18 item 7) and the prose-only advisory tier (f4sx
item 2) — and neither has a requirement in
`openspec/specs/conform/spec.md`. This change specs the implemented
behavior; it adds no new semantics.

## What Changes

- Scenario records accept an optional `expect` field whose value comes from
  the closed set {`forbidden`} (`Expectation`). The record schema keeps
  `deny_unknown_fields`; an invalid value is a `corpus_parse` schema error
  naming the field and the closed set with the one-JSON-object-per-line
  remediation hint, never a silent "no expectation".
- `VerdictRecord` gains two additive fields: `expect` (the declaration
  echoed) and `expectation_met: Option<bool>` (`null` when the scenario
  declared none).
- A forbidden verdict that matches `expect: forbidden` is agreement-tier
  evidence: counted in `ConformReport.agreement_counts`, excluded from the
  hard `verdict_counts`, non-gating (exit 0), and rendered as the oracle
  agreeing in the human view.
- An unmet expectation keeps the mechanical verdict, prefixes its reason
  with the `expectation_unmet` marker, counts in the hard `forbidden`
  bucket, and exits 1.
- Records without `expect` keep today's semantics; their existing fields
  are unchanged and only the two new fields serialize as `null`.
- Undeclared prose-only covering claims are marked
  `VerdictRecord.advisory: true` with the verdict vocabulary unchanged
  (`unknown`), counted in `ConformReport.advisory_counts`, excluded from
  `verdict_counts` and from the `forbidden + unsupported` exit predicate,
  and named as non-gating in the human view.

## Impact

- Affected specs: `conform` — four ADDED requirements (scenario expectation
  declaration; expectation echo and agreement tier; unmet expectations;
  advisory tier). No existing conform requirement is modified: the closed
  verdict taxonomy, artifact gate, report schema, digest, and exit-code
  mapping keep their current text.
- Affected code: `src/conform.rs` (`Expectation`, `ScenarioTrace.expect`,
  `VerdictRecord`, `build_report_mode`, `human_view`),
  `src/commands/corpus.rs` (the unchanged `forbidden + unsupported` exit
  predicate now reads tiered `verdict_counts`), and the pinned tests in
  `tests/cli/conform.rs`, `tests/conform_classification.rs`, and
  `tests/conform_gate.rs`.
- Compatibility: additive serde fields with `#[serde(default)]`; reports
  without declarations or advisory records remain deserializable, and
  re-running conform over unchanged inputs stays byte-identical.
- Provenance: `specodelic-18mt` (gh-18 item 7 — optional `expect` on
  scenario records) and `specodelic-f4sx` class 2 (prose-only evidence
  degrades to advisory), both recorded in `src/conform.rs` doc comments
  and pinned by the named tests.