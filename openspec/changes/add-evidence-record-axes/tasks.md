# Tasks — add-evidence-record-axes

Each numbered step is one red→green cycle. Steps are separate commits; the
refactor step (5) is its own commit, never mixed with behavior.

## 1. Guard — red

- [ ] 1.1 `scripts/test_check_evidence_axes.py` (stdlib unittest, discovered by
  `sync-sections-test`): fixture tree where the documented sets equal the code
  sets → exit 0.
- [ ] 1.2 Fixture: a `Verdict` variant present in the code but absent from the
  table → exit 1 naming the variant.
- [ ] 1.3 Fixture: a table row marked `implemented` with no code counterpart →
  exit 1 naming the row.
- [ ] 1.4 Fixture: a documented `AuditClass::of` mapping that disagrees with the
  code arm → exit 1 naming both sides.
- [ ] 1.5 Fixture: a `reserved` row without a `trigger:` → exit 1.
- [ ] 1.6 Fixture: malformed enum (missing `pub enum Verdict`) → exit 1 with a
  remediation hint (fail-closed, never a silent pass).

## 2. Guard — green

- [ ] 2.1 Implement `scripts/check_evidence_axes.py`: extract `Verdict` variants
  from `src/conform.rs`, `AuditClass` variants and every `Verdict::X =>` arm of
  `AuditClass::of` from `src/audit.rs`, parse the table from
  `docs/src/graded-evidence.md`, compare in both directions.
- [ ] 2.2 Run it against the real tree: expect exit 1 because the axis table
  does not exist yet — this is the change's own red state, recorded in the task
  log, not worked around.

## 3. Documentation — green

- [ ] 3.1 Author the five-axis table in `docs/src/graded-evidence.md`: run
  status, spec defect, currency, knowledge absence, scope exclusion; every
  `Verdict` variant and `AuditClass` value appears exactly once; `implemented`
  vs `reserved (trigger: …)`.
- [ ] 3.2 Document the `AuditClass::of` mapping verdict-by-verdict, including
  why `agree` is `unchecked` (design D4).
- [ ] 3.3 Document `stale`/`new` as `--before` overlays guarded by
  `scope_sha256`, plus the prior-art note (Kelly & McDermid 2001; IEEE 828-2012
  §10) and the T1 citation for `inconclusive` (X.290 3.3.124, 6.5.1.3).
- [ ] 3.4 Re-run the guard against the real tree → exit 0.

## 4. Wiring

- [ ] 4.1 Add `evidence-axes` recipe to `justfile` and place it in the `ci`
  chain; no separate test recipe (the `test_*.py` discovery covers 1.x).
- [ ] 4.2 `openspec validate add-evidence-record-axes --strict` → clean.
- [ ] 4.3 `just ci` → green (gate included, docs table in place).

## 5. Refactor — separate commit

- [ ] 5.1 If row parsing and enum extraction duplicate normalization logic,
  extract it into two named functions with no behavior change; re-run 1.x + 4.3.

## 6. Close-out

- [ ] 6.1 Update the D-1 row in `.wai/resources/research/2026-11-17-meth-research-decision-memo.md`
  §7.1 with the shipped scope and the reserved-value triggers.
- [ ] 6.2 File the follow-up trigger tickets named in the `reserved` rows
  (`inconclusive` producer, `origin` annotation syntax, `binding_kind` consumer)
  so no reserved row references a nonexistent change.