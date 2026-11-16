# Design: add-conform-expectation

## Context

conform's report already distinguishes hard verdicts, prose-only evidence,
and — since the expectation work — oracle declarations. The implementation
in `src/conform.rs` fixes three properties that must stay true together:
the verdict taxonomy stays closed (a trace still receives exactly one of
`permitted`, `forbidden`, `underspecified`, `unknown`, `unsupported`); the
report accounts for evidence TIERS on top of the vocabulary; and the
published exit predicate remains `forbidden + unsupported` over
`verdict_counts` alone. The spec delta records that split so future changes
cannot collapse the tiers back into the vocabulary.

## Goals / Non-Goals

- Goals: specify the optional closed-set `expect` field; the two additive
  verdict-record fields; agreement-tier accounting for met expectations;
  hard accounting for unmet expectations; the advisory tier for prose-only
  covering claims; and unchanged semantics for undeclared records.
- Non-Goals: new verdict values; changing the artifact gate, digest, report
  schema version, or exit-code mapping; generating or recording traces;
  expectations for verdicts other than `forbidden`; any behavior change
  beyond what `src/conform.rs` implements today.

## Decisions

- **Decision: model the expectation as a closed single-variant enum
  (`Expectation`), not a boolean or a free string.** A boolean
  (`expect_forbidden: true`) would need one schema field per future
  expectation kind, and a free string would turn a typo into "no
  expectation" at parse time. The enum keeps the field additive, and
  serde's variant error names the accepted spelling, which rides the
  `corpus_parse` message beside the field name and the
  one-JSON-object-per-line remediation hint.
  - Alternatives considered: a `bool expect_forbidden` field (rejected:
    not extensible without schema churn); an untyped string validated later
    (rejected: the refusal would lose serde's named-variant detail and the
    parse would have to build an error path by hand).
- **Decision: the expectation tier is report-side accounting; the
  mechanical verdict is never rewritten.** A met `expect: forbidden`
  record still carries `verdict: forbidden` — the walk really did classify
  it that way. Tiering lives in `agreement_counts` versus
  `verdict_counts`, so consumers that read records see the truth and
  consumers that read counts see the tier.
  - Alternatives considered: a new verdict value such as `expected`
    (rejected: breaks the closed taxonomy and every consumer of it);
    dropping met-expectation records from the report (rejected: the report
    must remain total over the corpus).
- **Decision: an unmet expectation rides the hard `forbidden` bucket.** The
  published exit predicate is the sum of `verdict_counts["forbidden"]` and
  `verdict_counts["unsupported"]`. A declared prohibition that the Model
  permits is a genuine disagreement, so `build_report_mode` adds it to the
  `forbidden` bucket while the record keeps its mechanical verdict and
  carries `expectation_met: false` plus the `expectation_unmet` reason
  prefix. This gates the run without adding a second predicate to the
  exit-code contract.
  - Alternatives considered: a dedicated `expectation_unmet` count and a
    new gate term (rejected: churns the published exit mapping for the same
    observable outcome); rewriting the verdict to `forbidden` (rejected: the
    Model did not forbid the trace).
  - Ordering: the declared-expectation partition runs before the advisory
    tier, so an unmet expectation over prose-only evidence still rides the
    hard bucket and gates — a declaration is never hidden by the advisory
    tier.
- **Decision: prose-only evidence is advisory-tier, not a hard `unknown`.**
  The verdict vocabulary stays closed — prose is still not interpretable,
  so the record says `unknown` with `prose_claim_not_interpretable` — but
  `advisory: true` plus `advisory_counts` keep a prose-heavy corpus from
  reading as a pile of hard unknowns, and advisory records stay out of the
  exit predicate.
  - Alternatives considered: a new `advisory` verdict (rejected: vocabulary
    churn); leaving advisory records in hard counts (rejected: the tier
    distinction is the point of f4sx item 2).
- **Decision: additive serde fields default to absent/null.** `advisory`,
  `expect`, and `expectation_met` carry `#[serde(default)]`, so reports
  persisted before this change deserialize, and a record with no
  declaration differs from its old serialized form only by the two `null`
  fields.

## Risks / Trade-offs

- Risk: a consumer reading only `verdict_counts` sees `forbidden: 1` for an
  unmet expectation whose record verdict is `permitted`. Mitigation: the
  requirement states the bucket is an accounting tier, and the record's
  `expect`, `expectation_met`, and `expectation_unmet` reason prefix remain
  the source of truth; the human agreement line only appears for met
  expectations.
- Risk: `expect: forbidden` could be over-read as "anything is allowed".
  Mitigation: the expectation never changes a verdict, and an expectation
  the Model fails is a gating disagreement, not a pass.
- Risk: byte-level report snapshots gain the new null fields. Mitigation:
  the fields are additive and deterministic; the digest binds structured
  content plus corpus bytes, not report bytes, so re-run determinism and
  scope identity are unaffected.
- Trade-off: advisory unknowns vanish from `verdict_counts`, so the hard
  `unknown` count no longer totals all unknowns. That is deliberate — the
  two maps sum to the record total, and both views surface the advisory
  count.

## Migration Plan

None required. The change is additive at the schema and report level: old
corpora parse unchanged (no `expect`), old reports deserialize via serde
defaults, and no artifact or lifecycle transition is touched. Rollback is
reverting the source change; no persisted state depends on the new fields.

## Open Questions

- Whether further expectation kinds (for example a declared `permitted`)
  are worth adding — deferred until an oracle needs to assert one.
- Whether agreement-tier counts should ever affect the exit code — no, by
  construction; kept explicit so the decision is revisited deliberately.