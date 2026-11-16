# Change: Vacuity advisories — fragments that bind nothing, invariants over undeclared states

## Why

The content-advisory family (specodelic-xci) catches cells with **no**
content: whole-cell placeholders and tautological predicates. It cannot see
the opposite failure — content that exists, lints green, and binds nothing to
the model it claims to constrain. Two shapes carry it:

- **A fragment that cites no declared model element.** The catitas/bowling
  rev1 study recorded it exactly: executable `**rust:**` fragments that never
  called the scorer they exist to exercise, so the corpus was green while
  nothing was checked (`src/lint/content_advisories.rs`'s module doc;
  `src/audit.rs`'s closure note on the bowling-rev1 failure). `spk audit`
  already reports this shape on demand over an evidence corpus
  (`AuditClass::BindsNothing`, src/audit.rs:89 and its row emission at
  src/audit.rs:601) — but lint, the per-file authoring gate, is silent about
  it.
- **An invariant over an undeclared forbidden state.** `state != "aborted"`
  cannot fail when `Model/States` never declares `aborted`; the claim is
  structurally vacuous. No checker compares a forbidden literal against
  `Model/States` today.

Both are advisory by nature: a fragment may legitimately bind nothing yet
still be a scaffolding step, and a sparse phase model may deliberately keep a
defensive invariant over a phase its correct runs never enter.

## What Changes

- New rule `linter.fragment_binds_nothing` (advisory — warnings channel,
  exit 0). An invariant Constraint `expr` or a non-law Property `predicate`
  whose cell carries a `**rust:**` fragment MUST cite at least one declared
  model element: a Model state id, a transition id, or a Constraint/Property
  row id. The intent id is excluded — it is the file's own identity, not a
  model element. Matching is whole-token over `[A-Za-z0-9_]+` runs, never
  substring (`unlocked` does not cite `lock`), and `.` is a token boundary so
  a dotted `demo.spec.held` cites `held`. Law-kind predicates are outside the
  surface: their `**label:**` occurrences are the law-case grammar, never
  fragment markers.
- New rule `linter.vacuous_invariant` (advisory — warnings channel, exit 0).
  An invariant Constraint `expr` carrying a `**rust:**` fragment MUST, for
  every `state != "<literal>"` comparison (or its mirror
  `"<literal>" != state`), name a state declared under `Model/States`. Each
  undeclared literal is reported. Only the bare `state` binder's string
  inequality is judged — other binders (`my_state`, `model.state`,
  `state_extra`), `==`, arithmetic, calls, and escaped/raw literals stay
  silent.
- Both rules are **citation-versus-declaration** checks, never reachability or
  coverage: a declared element that is simply never exercised is never
  flagged. A declared-but-unreachable phase state is legitimate design
  pressure and stays silent, so the rule cannot nag an author off it.
- **One family, distinct rule ids.** The existing placeholder/tautology
  advisories keep judging WHOLE cells; the new rules judge the fragment's
  citation set. Each finding carries its own rule id and remedy, with one
  finding per subject — one per fragment (`fragment_binds_nothing`), one per
  undeclared forbidden literal (`vacuous_invariant`). Neither rule restates
  the other family's finding.
- **No new severity machinery.** The `Issue` model has no severity field;
  advisories ride `Report::warnings`, which `Report::failures()` (issue count)
  never counts (src/lint/mod.rs:247-271), and `spk lint` folds every warning
  onto the success envelope's warnings channel before it decides the exit code
  from `failures()` (src/commands/corpus.rs:206-216).

## Impact

- Affected specs: `linter-content_advisories` — two ADDED requirements for the
  new rules plus one ADDED requirement pinning their warnings-channel severity
  and their partition from the placeholder/tautology rules. No existing
  requirement is MODIFIED: `placeholder_content`, `tautological_predicate`,
  their scenarios, and the family's `advisory_severity` constraint stand
  unchanged — the new findings simply join the channel that constraint already
  governs.
- The delta's `layer.md` sidecar carries the capability's FULL layer, not just
  the new rows: the archive companion strips the deployed body's Constraints,
  Model, and Properties sections and re-inserts the sidecar's three tables
  (src/archive_companion.rs:273-291,508-548), refusing a sidecar that lacks
  any of them (src/archive_companion.rs:561-597). So the sidecar restates the
  four placeholder/tautology constraints and their four properties verbatim,
  adds the nine vacuity constraints with one unit property each (thirteen
  constraints and thirteen properties in total), and extends the one Model so
  `clean` means no finding of either class and `warned` means one of the four
  advisory constraints fired.
- Affected code: `src/lint/content_advisories.rs` (`inspect_vacuity` plus the
  declared-id, citation, and forbidden-literal helpers), `src/lint/mod.rs`
  (`RULE_TABLE` entries — the catalog, `spk explain lint-rules`, and the
  rule-semantics lookup all render from that one table),
  `src/lint/tests/content_advisories.rs` (rule tests and the catalog-coverage
  fixtures in src/lint/mod.rs's test module).
- Relationship to `spk audit`: audit's `binds_nothing` class is the on-demand,
  evidence-corpus/structured-row view; these rules are the per-file, lint-time
  view. Both read the SAME declared-ids index
  (`crate::lint::declared_rows` composed with `crate::lint::transition_ids`,
  shared by add-evidence-audit slice 3.3) and the same whole-token citation
  rule — one citation semantics, two consumers, never a parallel walk.
- Out of scope: fragment languages other than Rust. `compile::fragment_of`
  filters to `**rust:**` today (py/ts fragments are gated at validation), so a
  `**python:**`/`**ts:**` cell is outside this surface; widening it is a
  separate decision, not a silent drift.
- Provenance: specodelic-h5ks (vacuity advisories); the bowling rev1 failure
  recorded in the catitas studies; add-evidence-audit slice 3, which shares
  this check's declared-ids index with lint.