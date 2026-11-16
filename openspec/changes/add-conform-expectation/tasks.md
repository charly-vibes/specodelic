# Tasks: add-conform-expectation

## 1. Red — failing tests first

- [ ] 1.1 Schema acceptance: `expect: forbidden` parses and the scenario
      fields are unchanged
      (`expect_forbidden_accepted_by_the_schema`).
- [ ] 1.2 Schema refusal: an invalid `expect` value is a `corpus_parse`
      error naming the field, the closed set, and the one-JSON-object-per-line
      hint (`invalid_expect_value_refused_with_remediation`); an unknown field
      is still refused (`unknown_field_refused`).
- [ ] 1.3 Agreement tier: an expected forbidden exits 0, carries
      `expect: forbidden` and `expectation_met: true`, is absent from
      `verdict_counts`, and the human view says the oracle agreed
      (`conform_expected_forbidden_is_an_agreement_exit_0`).
- [ ] 1.4 Unmet expectation: a permitted trace declared `expect: forbidden`
      exits 1, keeps `verdict: permitted`, and its reason leads with
      `expectation_unmet`
      (`conform_unmet_forbidden_expectation_exits_1_with_labeled_reason`).
- [ ] 1.5 Undeclared pin: a contradicting trace with no `expect` stays a
      hard forbidden, exits 1, and the additive fields are `null`
      (`conform_record_without_expect_keeps_todays_semantics`).
- [ ] 1.6 Advisory tier: prose-only covering evidence is `unknown` with
      `advisory: true` and `prose_claim_not_interpretable`, counted in
      `advisory_counts`, absent from `verdict_counts`, and non-gating
      (`prose_only_covering_claim_is_advisory_not_a_hard_unknown`,
      `prose_advisory_never_counts_as_a_failing_verdict`).

## 2. Green — make the tests pass

- [ ] 2.1 `Expectation` closed enum plus `ScenarioTrace.expect` with the
      serde `default`; corpus parse errors keep the `corpus_parse` stage and
      remediation hint.
- [ ] 2.2 `VerdictRecord.expect` and `VerdictRecord.expectation_met` echo
      the declaration; `classify_trace` applies the expectation without
      rewriting the verdict, prefixing an unmet reason with
      `expectation_unmet`.
- [ ] 2.3 `build_report_mode` partitions met expectations into
      `agreement_counts`, unmet expectations into the hard `forbidden`
      bucket, and advisory records into `advisory_counts`.
- [ ] 2.4 `human_view` renders `agreed` for met expectations and the
      non-gating `advisory` line for prose evidence.
- [ ] 2.5 CLI exit predicate stays `forbidden + unsupported` over the tiered
      `verdict_counts`; no new gate term.

## 3. Refactor — separate commit

- [ ] 3.1 Keep a single partition point in the report builder so the tier
      order (expectation met → unmet → advisory → hard) is readable and
      tested.
- [ ] 3.2 `just ci` green; report determinism and serde-default
      deserialization pinned by tests.

## 4. Dual-format layer (fail-closed gate)

- [ ] 4.1 `specs/conform/layer.md` sidecar authored and validated via
      `just archive-change id=add-conform-expectation`.