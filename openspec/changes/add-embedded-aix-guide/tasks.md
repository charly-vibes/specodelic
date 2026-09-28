# Tasks: add-embedded-aix-guide

Ordered so every step keeps `just ci` green; the pure refactor (1.x)
lands before any behavior change.

## 1. Single-source the closed sets (pure refactor)

- [ ] 1.1 Create `src/guide.rs` (with the repo-standard
      Purpose/Responsibilities/Rationale header doc comment) with
      `pub const` slices for intent kinds, constraint kinds, property
      kinds, and the reference-typing pairs; add the module to
      `src/lib.rs`
- [ ] 1.2 Replace inline literal matches in `src/spec.rs` / `src/lint.rs`
      with the constants; all existing tests pass unchanged
- [ ] 1.3 Add `pub const FORMAT_REVISION: &str` to `src/guide.rs`

## 2. Embedded guide + `spk explain`

- [ ] 2.1 Write `src/guide.md` (six topics' prose; closed sets are
      *not* duplicated in prose — rendered from constants) and
      `include_str!` it
- [ ] 2.2 Implement `spk explain [TOPIC]` in `guide.rs` + thin clap
      wiring in `src/main.rs`: no arg → topic list; known topic →
      `{topic, format_revision, body}`; unknown → envelope failure with
      hint listing topics
- [ ] 2.3 Add `format_revision` to the `--version --json` payload by
      hand-rolling the version envelope in `main.rs` from
      `genesis::envelope::{Envelope, EnvelopeKind}` (stop calling
      `maybe_print_version_json` — its payload is fixed; see design
      Decision 4)
- [ ] 2.4 Integration tests in `tests/cli.rs`: consumer-dir scenario
      (tempdir, no specs), topic list exactness, unknown-topic hint,
      version payload field

## 3. Self-describing lint findings

- [ ] 3.1 Add a rule table to `src/lint.rs` mapping rule id →
      one-line semantics; thread `rule_id`/`rule_semantics` into every
      emitted finding (JSON + human)
- [ ] 3.2 Make `explain lint-rules` render from that same table
- [ ] 3.3 Unit test: catalog covers every rule id the linter can emit;
      integration test: `linter.ears_syntax` finding carries semantics

## 4. Scaffold guidance

- [ ] 4.1 Upgrade the `spk new` template with per-layer HTML-comment
      guidance (constraint kinds, guard citation rules, law-case
      requirements)
- [ ] 4.2 Test: scaffolded file carries guidance; linting a valid spec
      containing the comments yields no findings

## 5. Doctor dual-mode

- [ ] 5.1 Implement mode detection (`self_hosting` vs `consumer`) and
      mode-conditional checks in `cmd_doctor`; report
      `format_revision` in consumer mode
- [ ] 5.2 Implement the corpus-revision-vs-`FORMAT_REVISION` warning
      (extract `Revision N` headings, compare numerically; skip with an
      informational note when the corpus has no revision headings;
      warn, never fail)
- [ ] 5.3 Integration tests: self-hosting (this repo), consumer with
      specs dir, consumer empty, and the lag-warning tempdir fixture
      with a synthetic `Revision 99` corpus

## 6. Drift guard + gates

- [ ] 6.1 Unit test extracting the latest `Revision N` heading from
      `specs/specodelic.md` against `FORMAT_REVISION` (numeric
      comparison incl. `10 > 9`; fails the suite if stale)
- [ ] 6.2 Run `just ci` (fmt, clippy `-D warnings`, tests, release
      build), `spk lint specs` — corpus must stay clean — and
      `openspec validate add-embedded-aix-guide --strict`
- [ ] 6.3 Update `README.md` AIX section + `specs/STATUS.md` §3
      inventory with the new command; file/adjust beads issues
