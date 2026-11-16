# Design: add-vacuity-advisories

## Context

`src/lint/content_advisories.rs` already owns the placeholder/tautology
advisory class (specodelic-xci): whole-cell placeholder words, the `spk new`
stub statement, `scaffold_*` row ids gated on a scaffold signature, and the
narrow tautology set (`true`/`⊤`/`x == x`). It rides `Report::warnings` only —
the `Issue` model has no severity, and `Report::failures()` counts issues, so
warnings never reach the exit code (src/lint/mod.rs:247-271; the CLI folds
them onto the success envelope at src/commands/corpus.rs:206-216).

The catitas/bowling rev1 study recorded the failure that family cannot see:
green lint over a corpus whose executable fragments never touched the model
they exist to exercise, plus invariants that forbade states the Model never
declared. `spk audit` (add-evidence-audit) already turns that study's first
shape into structured `binds_nothing` evidence rows on demand, over the same
`crate::lint::declared_rows` ∪ `transition_ids` index (slice 3.3). This change
adds the lint-time, per-file view of both shapes.

## Goals / Non-Goals

- Goals: warn — never gate — on (a) an invariant/unit-property fragment that
  cites no declared model element and (b) an invariant that forbids a state
  `Model/States` never declares; keep both checks citation-versus-declaration,
  share the one declared-ids index with audit, and leave the
  placeholder/tautology advisories semantically untouched.
- Non-Goals: behavioral analysis of fragment expressions (no evaluation, no
  symbolic execution); reachability or coverage judgments over declared ids;
  gating on any of these findings; a second citation semantics for audit;
  widening the surface to `**python:**`/`**ts:**` fragments.

## Decisions

- **D1 — The citation target is the declared-id index, never the transition
  relation.** Declared ids are every constraint/property/state row id (one
  shared projection, `crate::lint::declared_rows`) plus every transition id
  (`crate::lint::transition_ids`) — exactly audit's evidence-unit target,
  reused rather than re-derived.
  - Alternatives considered: reachability via the model's transition relation
    (requires model semantics inside a lint pass and would flag a legitimate
    phase model's defensive invariant over a phase its correct runs never
    enter — rejected); a substring check (rejected: `unlocked` would cite
    `lock` and `beheld` would cite `held`, so almost every fragment would pass).
- **D2 — The intent id is not a model element.** A fragment citing only its
  own file id binds nothing: the intent is the file's identity, not a declared
  row, and a fragment that cites it checks nothing the file did not already
  state. Same rule audit documents for its `binds_nothing` rows.
- **D3 — Whole-token matching over `[A-Za-z0-9_]+` runs, with `.` as a token
  boundary.** The tokenization makes dotted references (`demo.spec.held`) cite
  `held` through their segments without a resolution pass, and makes
  `unlocked`/`beheld` miss `lock`/`held`. This is deliberately the cheap v1
  rule audit already uses; a graph-backed fan-in citation check remains the
  recorded refinement, not a second implementation.
- **D4 — Law predicates are outside the surface.** A law-kind predicate's
  `**identity:**`/`**associativity:**` occurrences are the law-case grammar,
  not fragment markers; the row carries per-case bodies (Revision 21) rather
  than one fragment cell. Skipping law rows mirrors `validate_fragments`'
  kind-based disambiguation and audit's evidence-unit collection.
- **D5 — The forbidden-state grammar is narrow on purpose.** Only the bare
  `state` binder's `!=` string comparison (either operand order) is judged;
  `==`, other binders (`my_state`, `model.state`, `state_extra`), helper calls
  (`is_forbidden(state)`), arithmetic, and escaped/raw literals stay silent.
  A lint pass is not a state-space analysis: false negatives are preferable to
  nagging an author about a comparison the checker only half understands.
  - Alternatives considered: parsing the fragment as Rust and matching
    `state` operands (a parser dependency in the lint path for a cheap
    citation check — rejected); judging `state == "<declared>"` as a
    reachability claim (that is behavior, not a forbidden-state citation —
    rejected).
- **D6 — Advisory severity comes from the channel, not a severity field.**
  Both findings are `Issue` values pushed to `Report::warnings`; nothing is
  pushed to `report.issues`, so `failures()` (and therefore the CLI exit code)
  is unaffected. Alternatives considered: adding a severity enum and demoting
  existing issues (touches every consumer for a decision the envelope's
  warnings channel already encodes — rejected).
- **D7 — One family, per-subject emission, no restatement.** Placeholder/
  tautology rules judge whole cells (prefix/equality on the stripped cell
  body, never a substring scan inside a fragment); the vacuity rules judge the
  fragment's citation set. A cell that is both tautological and non-binding
  draws each rule's own finding — two claims with two remedies — but a rule
  never emits twice for one subject: one warning per fragment for
  `fragment_binds_nothing`, one per undeclared forbidden literal for
  `vacuous_invariant`. Neither rule's message restates the other's finding.
- **D8 — The sidecar is the capability's full layer, not a patch.** The
  archive companion does not merge tables: it strips the deployed body's
  Constraints/Model/Properties sections and re-inserts the sidecar's three
  tables, re-keying the sidecar id to the deployed id
  (src/archive_companion.rs:273-291,367-404,508-548). A sidecar carrying only
  the new rows would therefore DELETE the four placeholder/tautology rows and
  their properties, dangling every `VERIFIES` link the deployed requirement
  scenarios carry. The sidecar restates those four constraints and four
  properties verbatim, adds nine vacuity constraints with one unit property
  each, and extends the single Model rather than adding a second one.
  - Alternative considered: a merge-aware companion that appends tables
    (changes the fail-closed contract and the rerun-convergence guarantee —
    rejected; the full-layer convention is what every archived sidecar
    already follows).

## Risks / Trade-offs

- **False negative — incidental citation.** A fragment that happens to mention
  a declared id (`let held = 0;`) passes. Accepted: the check asks whether the
  fragment binds anything, not whether the binding is meaningful. Graph-backed
  fan-in is the deferred refinement.
- **False positive — a fragment on a model whose ids are all unused words.**
  It warns, but the remedy in the message ("name a declared
  state/transition/constraint/property to bind it, or drop the fragment") is
  actionable and the finding never gates.
- **Nagging risk — reachability creep.** The temptation is to flag a declared
  bad state no transition enters. The spec's `SHALL NEVER` wording, the
  phase-model scenario, and
  `declared_forbidden_state_in_phase_model_stays_silent` pin the opposite:
  citation versus declaration, never reach.
- **Cross-file citations bind nothing.** A fragment citing
  `errors.envelope_error_kind` tokenizes into `errors`/`envelope`/`error`/
  `kind`, none of which is a declared id of the file being linted, so it
  warns. Documented as an open question rather than silently widened: a
  per-file lint has no corpus resolution pass.
- **Fragment-language drift.** `add-py-fragment-emission` and
  `add-ts-fragment-emission` are live changes; `compile::fragment_of` still
  filters to Rust. If the fragment surface generalizes, this rule's surface
  must be revisited deliberately, never widened by accident.

## Open Questions

- Should a cross-file citation count as binding for this rule? Today it does
  not (see Risks). Widening needs corpus-wide resolution in a per-file lint;
  defer until a real fragment needs it.
- Should the declared-id set include nested members (`row.member` paths)?
  Today only the row id is a citation target; members tokenize into their own
  segments and generally hit the row name anyway.
- Could the two lint rules and audit's `binds_nothing` rows share one
  emission surface (a structured advisory record)? Deferred: lint's findings
  are per-file, audit's are evidence-corpus rows with distinct exit semantics.