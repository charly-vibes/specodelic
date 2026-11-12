# Tasks: add-ref-kind-enforcement

## 1. Red — failing tests first

- [x] 1.1 Unit fixtures (src/lint tests): one violation fixture per
      typed field family — guard→non-invariant constraint, emits→non-effect
      constraint, traces_to→Constraint, derives_from→unit Property,
      supersedes cross-kind, satisfies→non-extension_point,
      observes→non-effect, uses→non-profile Intent — each asserting the
      labeled finding (`linter.ref_kind_compatible` + rule_semantics +
      remediation hint).
- [x] 1.2 Unit fixtures: pass-side controls — guard→invariant + guard→State,
      law→law-Property derives_from, satisfies→extension_point,
      observes→effect, uses→profile file, member-arm target typed as its
      row; plus dangling-target skip (`total_refs` owns it) and
      frontmatter/expr-cell skip (not typed fields).
- [x] 1.3 Catalog coverage: `ref_kind_compatible` joins RULE_TABLE and
      `fixture_corpus()` gains a fixture that fires it (keeps
      `catalog_covers_every_rule_the_linter_can_emit` true).
- [x] 1.4 CLI fixtures (tests/cli/lint.rs): `spk lint` on a violating
      corpus exits 1 with `linter.ref_kind_compatible` in JSON and human
      output; clean corpus exits 0.
- [x] 1.5 Narrow run fails for the expected reason (rule unknown/no
      finding), not setup breakage.

## 2. Green — make the tests pass

- [x] 2.1 `lint_ref_kinds` in src/lint (corpus-wide reference rule):
      schema-typed columns only, metasyntactic/anchor/dangling skips,
      endpoint resolution (file id → Intent-with-frontmatter-kind,
      row/member-arm → row kind), decision via
      `acset::schema::typing_violation`, `Issue::new("ref_kind_compatible", …)`
      with the schema's labeled reason + remediation hint.
- [x] 2.2 Wire into `lint_corpus` and `referential_findings` (the
      `linter-referential_integrity` checker's engine surface), never
      double-firing on either path.
- [x] 2.3 Emitted through `genesis::guide::Output::emit` (no new output
      path — findings ride the existing Issue/envelope plumbing).

## 3. Corpus stays green (dogfooding gate)

- [x] 3.1 `just lint-specs` clean after the rule lands (probe evidence:
      graph violations were 0 pre-change; any new finding is conformed
      in the corpus row and cited here — none expected). Verified:
      48 files, 0 issues, 53 advisory warnings (baseline unchanged).
- [x] 3.2 `tests/cli/lint.rs` corpus pin (`files_linted == 48`, issues
      empty) passes unchanged.
- [x] 3.3 `tests/reference_oracle.rs` agreement + corpus gates pass.

## 4. Refactor — separate commit

- [x] 4.1 Tidy pass: pretender ratchets hold (`pretender check`), no
      per-field match arms introduced, `just ci` green.

## 5. Dual-format layer (fail-closed gate)

- [x] 5.1 Wrap the delta: `spk migrate` scaffolds the `layer.md` sidecar
      for `specs/linter-referential_integrity/spec.md`.
- [x] 5.2 Deploy ONLY via `just archive-change id=add-ref-kind-enforcement`
      (never hand-edit `openspec/specs/`).
- [x] 5.3 Author `.espectacular/` contract TOMLs for the change's
      scenarios in the archive commit.
- [x] 5.4 `spk init --force` refreshes the AGENTS.md managed rule catalog;
      `git diff AGENTS.md` is catalog-only.
- [x] 5.5 Gates: `just test` green, `just ci` green, `ah check` no NEW
      findings, `openspec validate --all --strict` clean, `just lint-specs`
      clean.
