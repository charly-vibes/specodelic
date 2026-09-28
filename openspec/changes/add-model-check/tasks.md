# Tasks: add-model-check

Beads ticket: `specodelic-nx7` (claimed). Regression bar: `just ci` green
at every step. TDD: each phase is a red→green cycle — write the failing
test first.

## 1. Run report foundation

- [x] 1.1 RED→GREEN: `RunReport` struct (backend `{engine, version}`,
      bound, outcome ∈ {`no_counterexample`,`counterexample_found`,
      `timed_out`}, `violated_invariant_id?`, `trace?`,
      `artifact_sha256`, `invariants_checked`) serializes stably;
      unit test pins field set + JSON shape
- [x] 1.2 RED→GREEN: `Bound` (max-depth default 100, optional
      max-states, optional timeout) restates itself in the report
      (`exhaustive_within_bound`)

## 2. Native stateright backend

- [x] 2.1 Add `stateright` dependency; `backend.version` reports the
      pinned crate version (`backend_identified`)
- [x] 2.2 RED→GREEN: interpret `ModelIR` as a program-counter stateright
      model — init = first listed state, actions = transitions, always
      enabled (matches the committed `.tla` semantics); test pins the
      reachable set for a fixture automaton
- [x] 2.3 RED→GREEN: exhaustive-within-bound exploration yields
      `no_counterexample` with `invariants_checked: []` for a prose-guard
      spec — never a fabricated counterexample (Decision 3, Option A)
- [x] 2.4 RED→GREEN: budget exhaustion yields `timed_out` (tiny
      `--max-depth`/`--max-states` fixture), never collapsed into
      `counterexample_found` or `clean`

## 3. Step machinery + CLI

- [x] 3.1 RED→GREEN: precondition — the compiled artifacts must exist
      (`<stem>.tla` from a prior `spk compile`); missing artifact is a
      labeled ERROR with a `spk compile` hint; model-check never
      re-compiles
- [x] 3.2 RED→GREEN: run state machine transitions
      (`not_run → running → clean|counterexample_found|timed_out`) and
      the envelope outcome mapping; failure envelopes follow the repo
      convention (`ok:true` + `envelope_kind:"error"`, stderr in JSON
      mode)
- [x] 3.3 RED→GREEN: `<stem>.check.json` written next to the artifacts,
      carrying `artifact_sha256` of the consumed `.tla`
      (`rerun_on_model_change` input for 1pv); staleness self-check:
      re-running with an edited model re-hashes and reports fresh
- [x] 3.4 Wire `spk model-check <files> [--max-depth] [--max-states]
      [--timeout-secs]` in `src/main.rs` (file-header rule on new/changed
      files); `--json` envelope per meter: `.data.outcome`

## 4. Artifacts + docs

- [x] 4.1 Regenerate the committed corpus under `specodelic/` (now with
      `.check.json`); dogfood `spk model-check specs/model_check.md`
- [x] 4.2 CHANGELOG entry; STATUS/README CLI rows updated if needed

## 5. Close out

- [x] 5.1 `just ci` + `just lint-specs` + `openspec validate
      add-model-check --strict`
- [x] 5.2 Update this checklist; amend `specodelic-nx7` description
      (Decision 3 outcome, ug3 re-scope); commit + push
