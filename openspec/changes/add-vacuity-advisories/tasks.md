# Tasks: add-vacuity-advisories

## 1. Red — failing tests first

- [ ] 1.1 Vacuous fragment warns: an invariant expr `**rust:** flag != 0`
      over a file declaring the constraint, a `held` state, an `advance`
      transition, and a unit property yields one
      `linter.fragment_binds_nothing` warning naming the owning row, with the
      file label attached and `failures() == 0`
      (`fragment_citing_no_model_element_warns_binds_nothing`).
- [ ] 1.2 Undeclared forbidden state warns: `**rust:** state != "aborted"`
      with `Model/States` = `running`, `done` yields
      `linter.vacuous_invariant` naming both the constrained row and the
      literal `aborted`, warnings channel only
      (`invariant_citing_undeclared_forbidden_state_warns`).
- [ ] 1.3 Declared phase state stays silent: `**rust:** state != "halted"`
      with `halted` declared in `Model/States` fires neither vacuity rule —
      citation versus declaration, never reachability
      (`declared_forbidden_state_in_phase_model_stays_silent`).
- [ ] 1.4 Law predicate skipped: a law-kind row whose predicate carries only
      `**identity:**`/`**associativity:**` labels (no `**rust:**` fragment)
      produces no `linter.fragment_binds_nothing` finding.
- [ ] 1.5 Whole-token citation: `**rust:** unlocked == false` does NOT cite a
      declared `lock` state and warns, while `demo.spec.held` cites the
      declared `held` and stays silent.
- [ ] 1.6 Narrow forbidden-state grammar: `state == "halted"`,
      `my_state != "halted"`, `model.state != "halted"`, `state != r"halted"`,
      and an escaped literal stay silent — no `linter.vacuous_invariant`.
- [ ] 1.7 Exit 0 pinned: both rules' fixtures assert `failures() == 0` and no
      issue entry under either rule id (warnings channel only, never issues).
- [ ] 1.8 Catalog coverage: RULE_TABLE fixtures in `src/lint/mod.rs`'s test
      module fire each rule id, so the catalog test proves each new rule is
      describable and its one-line semantics is present.

## 2. Green — make the tests pass

- [ ] 2.1 `inspect_vacuity`: judge invariant Constraint `expr` rows and
      non-law Property `predicate` rows through `compile::fragment_of`; skip
      non-invariant constraints, law-kind properties, and cells with no
      fragment.
- [ ] 2.2 `declared_model_ids`: constraints ∪ properties ∪ states from the
      shared `declared_rows` projection, plus `transition_ids`; the intent id
      is absent by construction (no exclusion filter to keep in sync).
- [ ] 2.3 `warn_binds_nothing` + `cites_declared_id`: whole-token citation
      over `[A-Za-z0-9_]+` runs with `.` as a boundary; one warning per row
      fragment; the message names the row, says the binding is vacuous, and
      offers both remedies (name a declared element / drop the fragment).
- [ ] 2.4 `warn_undeclared_forbidden_states` + `forbidden_state_literals`:
      bare `state` binder only, `!=` on either side, escapes skipped; one
      warning per undeclared literal, filtered against `Model/States` ids;
      the message says a declared-but-unreachable phase state never warns.
- [ ] 2.5 `RULE_TABLE` entries for `fragment_binds_nothing` and
      `vacuous_invariant`; module docs in `src/lint/content_advisories.rs`
      and the advisory wiring note in `src/lint/mod.rs` name both rules.
- [ ] 2.6 Docs: `spk explain lint-rules` renders from `RULE_TABLE`, so the
      rendered catalog must name both rules with their advisory severity —
      verify the rendered output, do not hand-edit a rule list.

## 3. Refactor — separate commit

- [ ] 3.1 Keep one declared-ids projection for lint and audit
      (`crate::lint::declared_rows` ∪ `transition_ids`) — no lint-local copy,
      no second citation predicate.
- [ ] 3.2 `just ci` green; the added tests stay in
      `src/lint/tests/content_advisories.rs` so no file crosses the
      file_lines ratchet.

## 4. Dual-format layer (fail-closed gate)

- [ ] 4.1 `specs/linter-content_advisories/layer.md` sidecar authored as the
      capability's FULL layer — the four existing placeholder/tautology
      constraints and their four properties restated verbatim, the nine
      vacuity constraints with one unit property each (thirteen constraints,
      thirteen properties), and the single Model extended so `clean` means no
      finding of either class — validated by
      `just archive-change id=add-vacuity-advisories`.