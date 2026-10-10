# Tasks: add-conform

Each phase is one red→green→refactor cycle; the failing test is written
and observed to fail before the implementation that makes it pass.
Tidying commits are separate from feature commits.

## 1. Verdict engine core (pure, no CLI)

(Assumes design.md OQ2 resolution — invariant claims + Model only, laws
excluded; revisit before 1.2 if the maintainer rules otherwise.)

- [x] 1.1 **RED**: unit tests in `tests/conform_classification.rs` for the
      closed taxonomy over a fixture spec + fixture JSONL corpus:
      executable-claim contradiction → verdict with claim id in reason
      (per `required_claims_classified` semantics); prose-only covering
      claim → `unknown` naming the claim; no covering claim →
      `underspecified`; unsupported evaluator kind → `unsupported` naming
      the kind. Run `just test-smart` — all new tests must fail (module
      does not exist).
- [x] 1.2 **GREEN**: implement claim classification consumption in
      `src/conform.rs` reusing model_check's claim types (D4 — no new
      status vocabulary); classify traces against the compiled Model's
      transition relation (mechanical name identity after trimming, D3).
- [x] 1.3 **RED→GREEN**: evidence-class separation (design D2) — a
      contradicting trace classified *without* the declaration is still
      `forbidden`, record `closed_world: false`, reason names the claim
      (`contradiction_forbidden_any_mode`); an uncovered trace *with*
      `--closed-world` is `forbidden` with `closed_world: true` and the
      exhaustiveness declaration named in the reason
      (`closed_world_forbidden_recorded`); the same uncovered trace
      without the declaration is `underspecified`
      (`uncovered_trace_never_forbidden`).
- [x] 1.4 **RED→GREEN**: taxonomy totality — every corpus trace receives
      exactly one verdict; fixture corpus reaching all five values
      (`taxonomy_is_total_and_distinct`).
- [x] 1.5 **TIDY**: extract verdict-reason formatting into a testable
      unit; dead-flag and clippy sweep.

## 2. Report schema and scope digest

- [x] 2.1 **RED**: `tests/conform_report.rs` — persisted report carries
      `report_schema_version` (conform-local, not model_check's
      `claim_schema_version`), per-trace records (scenario id, verdict,
      reason, evaluated claim ids, closed_world flag), `evidence_scope`
      (fixed statement: agreement-on-corpus, not behavioral equality),
      `scope_sha256`; JSON and `--human` views both carry
      `evidence_scope` (`report_schema_roundtrip`,
      `evidence_scope_present_in_both_views`); two runs over identical
      inputs are byte-identical — records sorted by scenario id, no
      timestamps (`rerun_byte_identical`).
- [x] 2.2 **RED**: digest binding — identical spec content with
      one-byte-different scenario corpora produce different
      `scope_sha256`; identical inputs produce identical digests across
      runs and path reordering (`digest_binds_scenarios`).
- [x] 2.3 **GREEN**: implement the report in `src/conform.rs` emitting
      through `genesis::guide::Output::emit` (envelope default for pipes,
      `--human` for TTYs); extend model_check's scope-digest computation
      to bind scenario corpus bytes for this run only — model_check's own
      digest contract is unchanged.
- [x] 2.4 **TIDY**: shared digest helper extracted if (and only if) the
      model_check digest contract stays byte-identical; otherwise keep
      conform-local.

## 3. Input gate

- [ ] 3.1 **RED**: `tests/conform_gate.rs` — stale artifacts (spec
      structured content changed after compile) refused with hint naming
      compile; lint-dirty file refused with hint naming lint; both emit
      zero verdict records (`stale_artifacts_refused`,
      `lint_dirty_refused`).
- [ ] 3.2 **GREEN**: gate in the command path reusing the artifact
      currency check orchestrate applies between stages (D5); refusal
      carries the remediation hint per the error contract.
- [ ] 3.3 **RED→GREEN**: scenario corpus validation — malformed JSONL
      lines, missing `id`, duplicate ids, unknown fields refused with
      remediation hints, never silently ignored (D3).
- [ ] 3.4 **RED→GREEN**: zero-line corpus yields a valid report with
      zero records and `evidence_scope` intact, exit 0 — never an error
      (`empty corpus` scenario in the report-schema requirement).

## 4. CLI wiring

- [ ] 4.1 **RED**: CLI test in `tests/cli/` — `spk conform <file>
      --oracle scenarios.jsonl` produces the report envelope;
      `--closed-world` is accepted and recorded; no-declaration runs
      never contain a `forbidden` verdict
      (`open_world_never_forbidden` at CLI level).
- [ ] 4.2 **GREEN**: add the `Conform` command to `src/main.rs`
      (`Commands` enum, help text stating read-only evaluation); wire the
      exit-code contract: 0 = no `forbidden`/`unsupported` verdict, 1 =
      any `forbidden`/`unsupported`, `unknown`/`underspecified` surfaced
      as counts but never failing, 2 = invocation error / gate refusal
      with no verdict records emitted.
- [ ] 4.3 **TIDY**: completions regenerate; `spk explain` untouched (no
      new topic this change — proposal contract).

## 5. Lifecycle independence and dogfood

- [ ] 5.1 **RED**: `tests/conform_lifecycle.rs` — `spk orchestrate` over
      a spec with conform available never invokes conform and the
      pipeline stages are unchanged (`outside_lifecycle`); a conform run
      over a `model_checked` artifact leaves the stage unchanged.
- [ ] 5.2 **GREEN**: assert orchestrate's stage list; no orchestrate
      code change expected (constraint is conformance, not behavior).
- [ ] 5.3 **RED→GREEN**: read-only dogfood — run conform against a
      fixture copy of a `specs/` corpus file and a fixture oracle corpus;
      assert spec files and git worktree are byte-identical after the
      run (`consumes_never_writes`).
- [ ] 5.4 **GREEN**: dual-format gates — `spk lint` the delta, `just
      sync-sections` passes (ADDED/Requirements mirror identical), `spk
      lint openspec/specs` unaffected.
- [ ] 5.5 **TIDY**: `just ci` full gate; docs book pipeline section gains
      the conform page stating the evidence scope (D6 wording verbatim).

## Dependencies

- Phase 1 → 2 → 3 → 4 (each phase consumes the prior one's types).
- Phase 5 is partially parallelizable (5.1–5.2 independent of 4).
- Sequencing: read-only consumer of compile/model_check types — do not
  modify `src/compile.rs`'s claim model concurrently with
  `add-py-fragment-emission` / `add-ts-fragment-emission` (proposal note).