# Tasks: add-language-neutral-property-binding

Each phase is one red→green→refactor cycle; the failing test is written
and observed to fail before the implementation that makes it pass.
Tidying commits are separate from feature commits. Phase 1 is the
espectacular seam (no specodelic code); phase 2 is the grammar; phase 3
is docs and closure.

## 1. Espectacular pytest/property wiring (no specodelic code)

- [ ] 1.1 **RED**: enable the capabilities — `ah doctor --enable pytest`
      and `ah doctor --enable property`; verify `.espectacular/config.toml`
      gained the pytest runner entry. Confirm `hypothesis` is actually
      importable (`uv run python -c "import hypothesis"`); if absent,
      add it to the exemplar's pinned runner invocation (`[[tests.shell]]`
      with `uv run`) per design D-risk 3.
- [ ] 1.2 **RED**: choose the exemplar scenario from a *deployed*
      capability (espectacular contracts bind deployed scenarios, not
      change overlays). Prefer `parse`'s envelope well-formedness if its
      scenarios are contract-free. Write
      `tests/python/test_exemplar_props.py` — a hypothesis property test
      asserting the scenario's contract (e.g. envelope `ok: true` and
      `envelope_version` present over generated corpus inputs).
- [ ] 1.3 **GREEN**: author the contract TOML
      (`.espectacular/<capability>/<id>.toml`, archetype PF) binding the
      scenario to the pytest test — `[[tests.pytest]]` with a **node id**
      (not `-k`) per `ah explain scenario-scoped-tests`. Run
      `ah check --run-tests` — green.
- [ ] 1.4 **TIDY**: extract any helper wiring into the test file's
      conftest; re-run `ah check` and `just ci` (the pytest test must
      also pass under `just ci` or be explicitly scoped out with a
      recorded reason — decide against real output).

## 2. Closed language-tag fragment grammar

- [x] 2.1 **RED**: grammar tests in `src/spec.rs` / extraction tests —
      `**py:**` fragment extracts with the fragment-position rule;
      `**go:**` marker produces a labeled failure naming the tag and the
      closed set; mid-span `**py:**` is a mention (no extraction); an
      invariant expr cell accepts the same grammar. Run `just test` —
      new tests fail (tags unknown today).
- [x] 2.2 **GREEN**: widen the marker grammar to the closed tag set
      `{rust, py, ts}` in the extraction path (`src/spec.rs`,
      `src/compile.rs` fragment handling) — tag recorded alongside the
      fragment; unknown tag → labeled failure (design D7). `**rust:**`
      path byte-identical (2.3 proves it).
- [x] 2.3 **RED→GREEN (back-compat proof)**: pin the existing rust
      extraction fixtures (`tests/spec_parse.rs` and compile tests) and
      assert post-change extraction is byte-identical
      (`rust_back_compat`). These must pass unchanged — if any fails,
      the widening is not pure and 2.2 is wrong.
- [x] 2.4 **RED→GREEN (emitter gate)**: compile a fixture corpus carrying
      a `**py:**` fragment — expect the labeled extraction failure with
      remediation naming the py-emitter follow-up (`no_emitter_labeled_failure`),
      never silent rust emission and never a vacuous artifact.
- [x] 2.5 **TIDY**: dead-code and clippy sweep (`just ci`); confirm
      `just lint-specs` stays clean (this change's delta spec is the
      corpus candidate — the dual-format archive lands it).

## 3. Format revision + docs (after or concurrent with specodelic-lf3)

- [x] 3.1 **Corpus revision**: add the Revision heading to
      `specs/specodelic.md` and `specs/compile.md` —
      `fragment_language_closed`, `unknown_tag_rejected`,
      `no_emitter_labeled_failure` rows; the widening note (pure, per
      `rust_back_compat`); the `**py:**`/`**ts:**` no-emitter gate. Fold
      the review's three-layer ownership table into
      `specs/compile.md`'s trailing notes (where proptest blocks' scope
      is already recorded). Dogfood: `just lint-specs` clean.
- [ ] 3.2 **Docs**: add the boundary restatement (D6) and the two-binding
      layers note (artifact metadata comments vs contract-TOML flags,
      CORR-001's correction) to `docs/src/commands.md`; note the deferred
      py emitter and generator-vocabulary follow-ups in
      `specs/STATUS.md`'s tracking section.
- [ ] 3.3 **Validate**: `openspec validate add-language-neutral-property-binding --strict`;
      `ah check` green over deployed specs with the change overlay;
      `just ci` green.
- [ ] 3.4 **File follow-ups** (bd): `add-py-fragment-emission` (py emitter
      + pytest adapter behind `PropertiesRunner`, carrying the generator
      vocabulary decision D5 in the same change) and `add-ts-fragment-emission`.
- [ ] 3.5 **TIDY**: fix the stale notes prose in `specs/compile.md` —
      "Neither has its own spec yet" contradicts the parenthetical
      "(both now done — see `model_check.md` and `verify.md`)" two lines
      above it; reword to reference both spec files.
      Dogfood: `just lint-specs` clean.