# Changelog

Append-only. Past entries are never edited — a correction gets a new entry,
the same discipline `specodelic.md`'s own `append_only_variants` rule
requires of spec files themselves. Displayed newest first; numbered
chronologically ascending (`#1` = oldest) so a new entry always gets the
next integer regardless of where it's inserted in the display order.

## #35 — `spk init` (SPECODELIC managed block in AGENTS.md) and `spk feedback`

`spk init` writes or refreshes a `<!-- SPECODELIC:START/END -->` managed
block in the repo's `AGENTS.md` (genesis::managed_block injector — same
convention wai and espectacular use): the lint rule catalog rendered from
the same RULE_TABLE findings name, the embedded `format_revision`, and
the core commands. Idempotent — injected when missing, updated in place
when present, surrounding content never touched; parses its own revision
so `spk doctor` can warn (never fail) when the block is missing, stale,
or declares no revision. `spk feedback` files an issue against
charly-vibes/specodelic via the genesis unified feedback handler:
`spk feedback bug --dry-run` previews (content via stdin or
`--from-last-error`); gh-unavailable fallback writes the body to a local
file. Both ship in the 0.1.0 release (beads specodelic-ze4).

## #34 — `spk lint`/`graph`/`compile` search directories recursively; lint never silently succeeds on zero files

`collect_specs` now walks directories depth-first (sorted, deterministic),
so specs in nested directories are found (beads specodelic-6pi). Hidden
and build directories (`.git`, anything dot-prefixed, `target`,
`node_modules`) are never descended into — the explicitly named root is
always searched. `spk lint` on a path set that yields zero spec files now
fails with a remediation hint instead of a silent `ok:true` (a false
green); the parse-error failure path is unchanged. Three integration
tests pin the contract: nested specs are linted, hidden/build dirs are
skipped, and the empty result fails with a hint.

## #33 — `spk doctor` dual-mode: self-hosting vs consumer + knowledge-currency warning

`spk doctor` now classifies the workspace: `self_hosting` when
`specs/specodelic.md` exists, `consumer` otherwise, and reports the mode
in its envelope data. Consumer mode never fails on the missing corpus —
it reports the embedded guide's `format_revision` and suggests
`spk new` in an empty workspace. Whenever a local corpus exists, the
doctor compares its latest `## Revision N` heading (numerically largest
trailing integer) against the binary's embedded `FORMAT_REVISION` and
warns — on the envelope's warnings channel, never failing — when the
corpus is newer than the binary; a corpus with no revision headings
skips the check with an informational note (specodelic-amg,
add-embedded-aix-guide tasks 5.1-5.3, 6.2).

## #32 — Self-describing lint findings + `explain lint-rules` catalog

Every lint finding now carries a stable `linter.<name>` `rule_id` and a
one-line `rule_semantics` stating what the rule requires, in both the
JSON envelope and human output (`src/lint.rs`). The semantics come from a
single `RULE_TABLE` — the same table `spk explain lint-rules` renders its
catalog from, so the documentation can never disagree with what the
linter emits. The EARS rule was renamed to the stable id
`linter.ears_syntax` (matching `specs/linter-ears_syntax.md`'s id), and
the `{{lint_rules}}` placeholder in the embedded primer is no longer a
stub. Unit tests pin the catalog to exactly the rule ids the linter can
emit; `--version --json` already reports `format_revision` (specodelic-2kc,
add-embedded-aix-guide tasks 3.1–3.3, 6.2, 6.3).

## #31 — `model_check` made backend-pluggable: stateright default, TLC opt-in; Alloy dropped from the corpus

Neither TLC nor Alloy has native Rust bindings — both are JVM
subprocesses — but `stateright` is an embedded Rust model-checking crate
whose API already satisfies `model_check.md`'s whole contract:
breadth-first exploration gives `counterexample_is_minimal` by
construction, `target_max_depth`/`timeout` give
`exhaustive_within_bound`'s stated bound and `timed_out`, and named
properties give `counterexample_names_violated_invariant`.
`model_check.md` now specifies a backend contract instead of naming
engines in its invariants: stateright is the default (no JDK, unit-
testable inside `cargo test`), TLC stays as the opt-in reference engine
run against the `.tla` module, and a new `backend_identified` invariant
requires every run report to name its engine and version (with its
deriving property, keeping the corpus at zero coverage gaps).
`compile.md` keeps emitting the `.tla` module unconditionally — it is
the engine-portable, human-reviewable artifact, independent of backend
choice. Alloy is gone corpus-wide (`specodelic.md` Revision 8,
`compile.md`, `STATUS.md`, `USAGE.md`, `linter-model_shape.md`):
a SAT-based engine returns *an* instance, not a minimal trace, which
fights `counterexample_is_minimal`. `specodelic.md`'s
`no_counterexample` now says "the selected model-check backend" instead
of "(TLC/Alloy)". Implementation of the two backends tracked as beads
`specodelic-ug3`.

## #30 — corpus reaches covered: 23 deriving Properties rows added, no schema change

`spk lint specs` reported 23 coverage-rule findings (beads
`specodelic-qc8`): constraints across nine corpus files had no deriving
property, including fourteen in `specodelic.md` itself — the format's own
description failing its own coverage invariant. Closed entirely on the
corpus side, per `qc8`'s anti-goals: no lint rule was touched and no
Notes-cited waiver was used. Each gap got a genuine deriving `unit`
Property row whose generator/predicate test the constraint's own claim
(e.g. `specodelic.prose_untouched` ← `prose_does_not_affect_lint`, a
two-specs-differing-only-in-prose generator; `merge.graph_reused_not_rederived`
← `reachability_from_graph_artifact_only`, a walker-patched-to-panic
generator). `tests/cli.rs`'s corpus assertion was tightened from
"coverage gaps are known and tracked" to "zero findings, exit 0".
Also fixed `just lint-specs` failing on cargo's two-bin ambiguity
(`default-run = "specodelic"` in `Cargo.toml`). This unblocks
`specodelic-lnq` (compile), whose `compile` guard requires coverage to
hold.

## #29 — `USAGE.md`: empirical runtime bounds (§2.8), no schema change

Checked whether runtime/performance constraints belong in a spec at all.
Split into three cases, only one of which needed a new write-up:

- **Configured resource ceilings** (a max message size, a max eval
  timeout, a concurrency cap) were already representable and had already
  been used that way in practice (the REPLy.jl evaluation) without being
  named — an ordinary `invariant` Constraint, violation as a named
  terminal state. No change needed; called out explicitly in the new §2.8
  so it isn't confused with the pattern below it.
- **Measured performance SLAs** (p99 latency, throughput, memory under
  load) reuse Revision 5's threshold-Property mechanism
  (`precision(check(corpus)) ≥ 0.95`) unchanged — a `unit` Property with a
  benchmark-scenario generator. New in this entry: an explicit note that a
  benchmark result is re-runnable and environment-relative, not a
  permanent fact the way `acyclic_traces` is, borrowing `model_check.md`'s
  own framing for its checker output rather than inventing new language
  for the same idea.
- **Provable hard real-time guarantees** ("this machine always responds
  within Xms," verified by the model checker itself) — confirmed out of
  scope, for the reason `model_check.md` already states for unbounded
  checking generally: TLA+/Alloy verify discrete state reachability, not
  wall-clock behavior against real hardware. Recorded as a boundary, not
  built around.

Quick-start list, migration table, `AGENTS.md`'s catalog mention, and
`STATUS.md`'s inventory row updated to match. No `specodelic.md`/
`kinds.md`/linter change: `expr`/`generator`/`predicate` were already
unparsed strings before this entry, so a threshold needed no new field —
same shape Revision 5 already established, applied to a new domain.

## #28 — `USAGE.md`: event-sourcing pattern (§2.7), monad/idempotency law examples, no schema change

Surveyed FP and data-oriented practices (functional core/imperative
shell, exhaustive matching, errors-as-data, algebraic laws, event
sourcing) against the existing four layers the same way OOP practices
were surveyed for Revision 7. All of it turned out already representable
— several already enforced today but not narrated as such, none needing
a schema change:

- **New §2.7, event sourcing.** Current state as a Property whose
  `predicate` folds over a §2.1 sealed set of event-shaped Constraints,
  rather than a State the Model overwrites — the same
  `append_only_variants`/`supersedes`/"status is computed, never stored"
  mechanism this repo already uses on itself (`no_stored_superseded_flag`),
  pointed at a domain instead of at the format's own Revision history.
  Model/States stays reserved for a genuinely different concept: one
  event's own processing lifecycle, not the ledger's running total.
- **§2.5 extended**, not replaced: added a worked idempotency case
  (`apply(apply(x)) == apply(x)`, a named case beside identity/
  associativity) and a monad-laws worked example (`bind`'s left/right
  identity plus associativity), noting explicitly that the floor's own
  identity/associativity cases already *are* two of a monad's required
  laws under different traditional names.
- **Quick-start pattern list, migration table, `AGENTS.md`'s catalog
  mention, and `STATUS.md`'s inventory row** all updated to list the new
  pattern alongside the existing six.
- **Fixed a staleness bug found while doing this**: §4's "one of §2's
  five patterns" was already wrong after Revision 7 added §2.6 last
  session and nobody updated the count. Reworded to stop stating a
  number in prose at all — point at §2's own headings instead — so this
  can't go stale silently a third time.

No `specodelic.md`/`kinds.md`/linter change accompanies this entry: every
addition here is either an existing mechanism applied to a new worked
example (§2.5's extra cases) or an existing mechanism narrated as a named
pattern for the first time (§2.7) — nothing needed a new Constraint kind,
reference field, or invariant the way Revision 7's `extension_point` did.

## #27 — `specodelic.md` Revision 7: consumer-extended contracts (`extension_point` / `satisfies`), OCP disambiguated

Checked this format against Julia multiple dispatch and, more sharply,
Clojure protocols/multimethods — the expression-problem case where a
consumer conforms to a generic function or interface from a file the
origin author never edits. An earlier draft added a two-way `implements`
edge and a `single_root_reachable` carve-out so the origin file could
enumerate its conformers; discarded deliberately, since verifying
unknown, not-yet-written code isn't something this format's own
boundedness rules can honestly support (the same limit already admitted
for unbounded recursive structures, here across files instead of depth).

- **`specodelic.md`** — `constraint_kind_closed` widens to `{invariant,
  advisory, effect, extension_point}`; new Reference Typing row,
  `satisfies` (Constraint, any file → Constraint, kind == `extension_point`
  only), one-directional and outbound only. Two properties added
  (`satisfies_wrong_kind_rejected`, `extension_point_needs_no_reachability_carveout`).
  Written up as Revision 7, including why `guard`'s existing typing and
  `single_root_reachable` both needed zero changes, and why
  `linter-referential_integrity.md` needed zero code changes (same reason
  as `supersedes`/`emits` before it — it reads the Reference Typing table
  generically).
- **`kinds.md`** — `constraint_row_shape`'s kind set widens to match
  (Revision 4); new acceptance property
  `constraint_row_extension_point_accepted`.
- **`linter-schema_shape.md`** — `constraint_kind_closed` row widened to
  match; new acceptance property `constraint_kind_extension_point_passes`.
- **`USAGE.md`** — new pattern, §2.6: publish the contract as an
  `extension_point` Constraint in your own file; a consumer's own file
  points `satisfies` back at it. Migration table gets a new row for
  "third parties can extend this" (protocols, multimethods, plugin
  interfaces). Quick-start summary and `AGENTS.md`'s pattern-catalog
  mention both updated to list it alongside the existing five patterns.
- **`STATUS.md` §2** — the "Switch/case rigidity (OCP violation)" row is
  now two rows: OCP *within* one file's own governed set (already solved
  by append-only variant tables) and OCP *across* files (this Revision) —
  flagged explicitly as two mechanisms, not one restated twice, per the
  same-token-different-guarantee guideline `AGENTS.md` #6 already tracks.

## #26 — Simplification pass: removed a redundant invariant family, unified the CLAR items, added a frontmatter convention

A general review looking for abstractions/generalizations across all
files (not just within one), rather than another new feature.

- **Removed**, `refactor.md`: `finding_kind_closed_advisory`,
  `finding_never_gates_orchestrate`, and their Properties rows.
  **Removed**, `orchestrate.md`: `refactor_advisory_never_gates` and its
  Properties row. All three restated a fact `specodelic.md` already
  guarantees generically since Revision 5 (`advisory_cannot_gate`: no
  `advisory`- or `effect`-kind Constraint can ever be a `guard` target, by
  typing) — the exact "one rule, stated three ways" pattern Revision 6
  already caught once for `append_only_variants`. Each file now cites
  `[[specodelic.advisory_cannot_gate]]` in Notes instead.
  **Not removed**: `external_completeness_never_gates` in `orchestrate.md`
  — its own constraints are `invariant`-kind, so its non-gating status is
  a wiring fact (no transition happens to cite it), not a typing fact the
  type system already forbids; it still needs asserting and testing.
  `orchestrate.md`'s Notes now state this distinction explicitly so it
  isn't re-collapsed later.
- **`AGENTS.md`** — new item 3a addendum: check whether a proposed "X can
  never gate Y" invariant is already implied by `specodelic.md`'s
  Reference Typing before adding one. New item 3b: record a clean
  check-against-core-constraints result (item #3) as a
  `checked_against_core: clear` frontmatter field, not as restated Notes
  prose; reserve prose for when a gap actually surfaced or a judgment call
  is worth showing. New item 6: file naming-confusion items as instances
  of one guideline, not as unrelated one-offs.
- **`checked_against_core: clear` added** to the frontmatter of `compile.md`,
  `verify.md`, `model_check.md`, `rename.md`,
  `linter-external_completeness.md`, `orchestrate.md`, `graph.md`,
  `refactor.md`, `merge.md`; each file's near-identical "no new gap
  surfaced" sentence trimmed to a one-line pointer, substantive remainder
  of each paragraph kept as-is.
- **`STATUS.md`** — `CLAR-001`/`CLAR-002`/`CLAR-003` regrouped under one
  heading citing `AGENTS.md` #6, instead of three separately-titled
  subsections; `EXCL-001` kept adjacent but marked as a related-but-distinct
  shape (missing `kind`, not a confusable name). New "Done" subsection
  backfilled for `graph.md`/`refactor.md`/`merge.md` (missed in Changelog
  #25), including their three still-open `Needs Human Review` items.
- **`orchestrate.md`** — new Notes paragraph stating explicitly that
  `graph.md`'s reference graph and this file's Checker Ownership table are
  two different graphs (content-reference edges vs. tool-execution-order
  metadata) that shouldn't be folded into one, flagged because the surface
  resemblance makes that an easy mistake later.
- **No schema change.** Every removal above deleted a restatement, not a
  fact; every file's actual checked behavior is unchanged.

## #25 — `graph.md`, `refactor.md`, `merge.md` created; `theory.md` and `orchestrate.md` extended

Made the cross-file reference graph an explicit, queryable artifact
instead of something only reconstructible by hand, then built two
consumers on top of it: a non-gating tidy-first advisor and a
merge-time semantic-conflict check.

- **`graph.md`** — new file. Derives a single adjacency structure from
  every typed reference field in the repo (never hand-edited), and
  answers transitive-closure ("blast radius") queries against it. Scope
  boundary flagged `Needs Human Review`: whether `linter-referential_integrity.md`
  and `linter-graph_shape.md` should be refactored to query this artifact
  internally, left open rather than forced.
- **`refactor.md`** — new file. Mechanizes the "God object / cyclic
  dependency" pathology `STATUS.md` §2 already named and Revision 2 of
  `specodelic.md` already fixed once by hand: a node with high,
  unrelated fan-in (per `graph.md`), or a changeset touching only part of
  what a node owns, gets a non-gating finding. Reuses two existing
  mechanisms rather than adding new ones — the `advisory` Constraint kind
  (Revision 5) for non-gating, and `emits` (Revision 6) for the finding's
  shape — no sixth `𝒦` object, no new kind.
- **`merge.md`** — new file. Closes the gap textual (git) merges can't
  see: independent id collisions, and a rename on one branch left dangling
  by a new reference minted on the other (a critical pair, resolved as a
  pushout — see `theory.md`'s new **Confluence** entry). Depends on
  `graph.md` for blast-radius queries and `rename.md` for the actual
  rewrite mechanism, rather than re-deriving either; checked against
  `AGENTS.md` #3a before adding `rename_replayed_onto_foreign_edits` as a
  new row, since it's easy to mistake for a restatement of `rename.md`'s
  own `old_id_fully_replaced` (it isn't — that one guarantees completeness
  within a single linear rename, this one is about a reference the rename
  never saw, minted on a different branch).
- **`theory.md`** — two new entries: **Affected graph** (transitive
  closure / blast radius) and **Confluence** (pushout of two divergent
  rewrites), plus matching glossary rows.
- **`orchestrate.md`** — one new invariant, `refactor_advisory_never_gates`,
  mirroring the existing `external_completeness_never_gates` shape; a
  Notes paragraph clarifying that `graph.md`/`refactor.md`/`merge.md` sit
  outside this file's four-stage pipeline rather than adding a fifth or
  sixth stage.
- **No schema change** beyond what's listed above. `specodelic.md`'s own
  Constraints, Model, and Properties are untouched.

## #24 — `theory.md` created; category-theory framing centralized

Every category-theoretic claim scattered across `STATUS.md` §1,
`specodelic.md`'s Reference Typing intro, its Checker Ownership summary,
and its Notes section is now stated in full exactly once, in `theory.md`.
Each source location keeps a plain-language restatement of the same
guarantee and links out (`[term](theory.md#anchor)`) rather than
paraphrasing the math locally — the same "one stated rule, not several
hand-maintained copies that drift" move Revision 6 already made for
`append_only_variants`, applied to prose framing instead of a constraint.

- **New file**: `theory.md` — ten entries (schema/`𝒦`, typed foreign keys,
  document instance, well-formedness, naturality, namespacing/Grothendieck
  construction, interface-contract laws, Moore output, limit-over-a-
  diagram, additive-only evolution), each as plain-term / rigorous-term
  pair, plus a glossary table and a note on the `kind`/`kind`-column
  overload (`CLAR-002`).
- **Edited, current prose only**: `STATUS.md` §1's "categorical
  formalization" section and the three CT-framed passages in
  `specodelic.md` (Reference Typing intro, Checker Ownership's
  "Categorically:" paragraph, the Notes section's opening sentence).
  **Not touched**: any Revision N section in `specodelic.md`, or any
  other file's historical narrative — those are a record of what was
  true and reasoned about at the time, not current framing, and this
  repo's own discipline (`CHANGELOG.md`'s header, above) is that past
  entries are never edited. `𝒦`-notation left as-is inside structured
  `expr`/`predicate`/`guard` field text throughout, since `theory.md`'s
  glossary now defines `𝒦` rather than removing it from technical fields.
- **No schema change.** No constraint, property, state, or transition was
  added, removed, or reworded — this is a documentation-layer change only.

## #23 — `specodelic.md` Revision 6; `kinds.md` Revision 3; `USAGE.md` created

Checked the format against a second, unrelated domain (a lazy,
category-theoretic Python data library) end to end, then folded in the
same session per `AGENTS.md`:

- **`emits`** — a new optional field on `State` (`{id, emits?}` in
  `kinds.md`), typed via a new Reference Typing row (`State → Constraint,
  kind == effect`) and a new `effect` value on `Constraint.kind`. Gives a
  Model the output half of a Moore machine, which had nowhere to live
  before (`State` was fixed to `{id}` only). `compile.md`'s `model_to_tla`
  extended to compile it into a small `Output` function alongside `Next`.
- **Simplification**: `append_only_variants` (`specodelic.md`),
  `kind_field_extensible` (`kinds.md`), and `reference_field_extensible`
  (`specodelic.md` Revision 5) were the same "grows only, only via a new
  Revision heading" rule, discovered three times with wordings that had
  already drifted out of sync with each other. Collapsed into one
  statement of `append_only_variants` covering all three id-sets (variant
  tables, kind value-sets, the Reference Typing table's field set);
  `reference_field_extensible` retired as a separate row, `kind_field_extensible`
  reworded to cite the merged rule instead of restating it.
  `linter-schema_shape.md`'s matching constraints and Properties merged
  the same way (`id_set_grows_only`/`id_set_order_stable` replacing three
  separate constraints), with no loss of enforcement.
- **Wording bug fixed**: `linter-referential_integrity.md`'s
  `ref_kind_compatible` named three reference fields by hand
  (`traces_to`/`derives_from`/`guard`) instead of reading the Reference
  Typing table generically — already stale (missing `supersedes`, added
  Revision 5) despite `specodelic.md`'s own Revision 5 notes claiming
  this check needed zero changes for that addition. The implementation
  claim was true; the file's *wording* wasn't. Reworded to match.
- **`USAGE.md` created** — a domain-spec quick-start plus a pattern
  catalog (closed enumerations, Moore output, multi-implementation
  conformance via the Checker Ownership shape, staged/lazy evaluation, law
  cases beyond the required floor) and a migration guide from
  artifact-per-purpose formats. Linked from `AGENTS.md` and `STATUS.md` §5
  so it's found before someone concludes the format needs a new mechanism
  it already has under a different name.

## #22 — `specodelic.md` Revision 5: `advisory` Constraint kind, `supersedes`

Checked the format against ten features a real polyglot tool ecosystem
needed. Seven were already representable with no schema change (non-gating
quality signals via existing mechanisms once `advisory` below exists,
confidence-scored properties via the already-unparsed `predicate` field,
feedback/regression loops and priority-ordered fallback via the open
States/Transitions table, cross-language adapter contracts via a new file
rather than a sixth `𝒦` object). Two needed real additions:

- **`advisory`** added to `Constraint.kind` (`{invariant, advisory}`),
  with `guard`'s Reference Typing row narrowed to `Constraint, kind ==
  invariant` only — an advisory constraint can never gate a transition, by
  typing rather than by the checker remembering to skip it.
- **`supersedes`** added as a new, self-typed Reference Typing field
  (Constraint→Constraint, Property→Property) so a newer row can declare
  what it replaces; checked for acyclicity as its own independent
  `supersedes_acyclic` graph, kept separate from `traces_to`/`derives_from`
  on purpose (lineage and intent-tracing answer different questions).
  `reference_field_extensible` added alongside it, since adding
  `supersedes` exposed that the Reference Typing table's own field set had
  no stated growth rule (superseded, along with `kind_field_extensible`,
  by the merged rule in #23 above).

Enforced the same session: `linter-schema_shape.md` (kind-closure,
reference-table growth) and `linter-graph_shape.md` (`supersedes_acyclic`
as its own DAG check). `linter-referential_integrity.md`'s
`ref_kind_compatible` needed no code change, since it already read
`allowed_targets(field)` from the Reference Typing table rather than
hardcoding it — though see #23 for the wording debt this created and
didn't pay off until the next revision.

## #21 — `orchestrate.md` created; orchestration (P0) done — every prioritized §4 item now specced

Addressed `STATUS.md` §4's last prioritized item, the top-level
orchestrator (analogous to how `ddl` orchestrates the rest of that tool
ecosystem). Every pipeline stage already had its own spec — the six
Checker Ownership checkers plus `linter.coverage` for `lint`, `compile.md`,
`model_check.md`, `verify.md` — but nothing specified the thing that calls
them in order and gates each stage on the last. `orchestrate.md` closes
that with an `idle → lint_stage → compile_stage → model_check_stage →
verify_stage → succeeded/failed` lifecycle: checkers with a Checker
Ownership dependency are skipped (not failed) if their dependency failed,
independent branches (referential/graph/model vs. ears_syntax vs.
schema_shape) always run and report regardless of each other,
`linter.external_completeness` runs but never gates anything, and later
stages never start before the prior stage's exact specodelic.md guard is
met.

Explicit scope boundary drawn in Notes: this file drives the
lint/compile/model_check/verify pipeline only, never `rename` — a rename
is always a separate, on-demand operation, not something a pipeline run
can trigger implicitly.

No new gap surfaced in `specodelic.md`'s own constraint list, same
reasoning as `rename.md` and `linter-external_completeness.md`. One
`Needs Human Review` item opened: whether the orchestrator should also own
driving each file's own `draft → parsed` transition, or stay scoped to
`parsed → verified` as specified here. With this file, every item
`STATUS.md` §4 had prioritized (the old P0 through P2) is now specced;
what remains is the lower-severity `CLAR`/`EXCL` loose ends already on
record.

## #20 — `linter-external_completeness.md` created; external completeness (P0) done

Addressed `STATUS.md` §4's long-standing P1-then-P0 unsolved problem:
every gap folded into Revision 2 was found by a human eyeballing a new
checker file against `specodelic.md`'s existing list — `linter.coverage`
mechanizes internal consistency (a present constraint has a test) but says
in its own Notes it can never prove a constraint is *missing* outright.
`linter-external_completeness.md` mechanizes the structurally different
thing instead: given a declared external checklist, every item must carry
an explicit `covered` (mapped to a real constraint/property id) or
`waived` (with stated rationale) claim — so the checklist can never be
silently unconsulted, even though whether a mapping is *semantically*
correct stays outside what any checker here can verify (spelled out
plainly in the file's own Notes, the same honest half-measure
`linter.coverage` makes one level in).

Unlike the six Checker Ownership table checkers, this one is optional per
repo and doesn't gate `linted`/`compiled`/`verified` — a repo with no
declared checklist has nothing to be incomplete relative to. Whether to
require it for release is left to CI or the still-unbuilt orchestrator, a
policy layered on top of `specodelic` rather than a fact it asserts about
every repo.

One `Needs Human Review` item opened: `mapped_ids` is a reference the
existing Reference Typing table doesn't name, because a checklist file
isn't itself a specodelic file (no frontmatter, no Constraints/Model
layers) — whether it's a sixth kind outside `𝒦` or a degenerate spec file
reusing the four-layer shape is left undecided, and `mapping_naturality`
is asserted aspirationally until that's settled. No new gap folded into
`specodelic.md` itself. `STATUS.md`'s P0 is now fully done; orchestration
(previously P2) is renumbered P0.

## #19 — `rename.md` created; rename/refactor tool (P0) done

Addressed `STATUS.md` §4's P0 item. `rename_naturality` had been asserted
as a property in three files (`specodelic.md`,
`linter-referential_integrity.md`, `linter-graph_shape.md`'s
`topo_sort_naturality`) without the tool it's a law *of* ever having its
own Intent, Constraints, Model, or Properties. `rename.md` closes that: a
`requested → checked → applying → applied → verifying → passed/failed`
lifecycle for a single `(old_id, new_id)` request, with local invariants
for id-availability, filename-matching (per `id_matches_file`), atomicity
(all-or-nothing, never a partial edit), kind-preservation, and
non-interference with prose — then a `verify` step that re-runs
`linter.referential_integrity` and `linter.graph_shape` (the two checkers
whose owned constraints depend on cross-file state a single rename's own
bookkeeping can't self-certify) before reporting `passed`.

No new gap surfaced in `specodelic.md`'s own constraint list — the four
local invariants this file needed are the same shape as `compile.md`,
`model_check.md`, and `verify.md` each carrying constraints specific to
their own lifecycle step. One `Needs Human Review` item opened: whether a
batch rename (a whole namespace prefix at once) is one atomic transaction
or `n` independent ones — not specified here, left open rather than
assumed. `STATUS.md`'s P0 is now fully done; P1 (external completeness
checking) is renumbered P0.

## #18 — `verify.md` created; compile/verify pipeline (P0) fully done

Addressed the last of `STATUS.md` §4 P0's three items. `specodelic.md`
names `verify` as the `model_checked → verified` transition guarded by
`no_counterexample ∧ properties_pass`, but neither what "running the
properties" means nor how that conjunction is enforced was specified.
`verify.md` closes it: it executes every proptest! block `compile.md`
produced, and combines that with `model_check.md`'s clean/counterexample
outcome (`both_gates_required`) into the single `verified` gate.

`law_cases_all_run` deliberately echoes `specodelic.md`'s
`law_requires_cases`: that constraint ensures a law-kind property has
≥ 2 cases before it's allowed to compile; this one ensures all of the
compiled cases actually pass before the file can verify — the same
discipline on either side of `compile.md`.

No new gap surfaced in `specodelic.md`'s own constraint list.
`STATUS.md`'s compile/verify pipeline P0 is now fully done and folded
into a "Done" note; the rename/refactor tool (previously P1) is
renumbered P0.

## #17 — `model_check.md` created

Addressed the second of `STATUS.md` §4 P0's three items. `specodelic.md`
names `model_check` as the `compiled → model_checked` transition, guarded
only by `model_present` (a Model section exists) — the actual invocation
of TLC/Alloy against `compile.md`'s `model_to_tla` output, and what a
result must contain, had never been specified. `model_check.md` specifies
a bounded, re-runnable check with a minimal-counterexample guarantee and
a clean/counterexample/timed_out outcome space.

**New backlog item — CLAR-003:** writing this file made explicit something
implicit in `specodelic.md`'s naming: `model_checked` means "a run
happened," not "the run found no counterexample" — that fact
(`no_counterexample`) is a separate invariant consumed only by `verify`'s
guard. Same shape as CLAR-001 and CLAR-002; not resolved by renaming the
state, for the same reason (should go through `rename_naturality`).

No new gap surfaced in `specodelic.md`'s own constraint list — every
constraint in `model_check.md` traces to its own intent, referencing
`specodelic.md`'s existing `no_counterexample` and `model_present`
rather than restating them.

`verify` is the last item in the compile/verify pipeline (`STATUS.md` §4
P0) — it consumes both `compile.md`'s proptest! blocks and this file's
clean/counterexample outcome.

## #16 — `compile.md` created

Addressed the first of `STATUS.md` §4 P0's three items: `Compile`
(`specodelic.md`'s `linted → compiled` transition) had never been given
its own spec — only named as "the functor `Set^𝒦 → TOML`" in prose.
`compile.md` specifies all three of its target translations
(`Constraints → TOML`, `Model → TLA+/Alloy`, `Properties → proptest!`),
plus totality, id-preservation, and round-trip-stability guarantees.

No new gap surfaced in `specodelic.md` this time — every constraint in
`compile.md` traces to its own intent rather than to a top-level
invariant. One thing it *does* retroactively firm up: `specodelic.md`'s
`rename_naturality` law has always included a **naturality** case
(`compile(rename(I)) == rename(compile(I))`) referencing a `compile`
function that had no specification — that case is now checkable in
practice, not just written down.

`model_check` and `verify` — the other two P0 items — both consume
`compile.md`'s output (the TLA+/Alloy module and the proptest! blocks,
respectively) but still need their own spec files for running the
generated artifact and interpreting the result.

## #15 — `linter-schema_shape.md` enforces `constraint_kind_closed`/`property_kind_closed`

Closed the follow-up `kinds.md` (#14) recorded but left open: no checker
enforced the two new closed-kind invariants `specodelic.md` Revision 4
added. Added a `kind_checking` phase to `linter-schema_shape.md`'s model,
ahead of the existing revision-diffing (`diffing` needs a prior revision
to compare against; the kind-closed check doesn't, so it runs first and
independently — a `diff_skip` edge was added for files with no revision
history, which the model previously had no path for). Two new properties
(`constraint_kind_invalid_rejected`, `property_kind_invalid_rejected`)
plus a passing case. Updated `specodelic.md`'s Checker Ownership table
row for `linter-schema_shape.md` to list both newly-owned constraints.

`STATUS.md`'s P0 backlog item is now closed; P1 (compile/verify pipeline)
renumbered to P0.

## #14 — `kinds.md` created; `specodelic.md` Revision 4

Addressed `STATUS.md` §4 P0: `Intent`, `Constraint`, `State`, `Transition`,
`Property` — the five objects of `𝒦` — had never been specified as
subjects in their own right, only referenced from scattered prose and the
Reference Typing table. `kinds.md` now gives each a canonical field set
and, for the two that carry their own `kind` column (Constraint:
`invariant`; Property: `unit`/`law`), a closed value set.

**Gap surfaced, folded into `specodelic.md` as Revision 4 (same
session):** nothing previously required a Constraint or Property row's
own `kind` column to come from a closed set at all. Added
`constraint_kind_closed` and `property_kind_closed`, tracing to
`kinds.md`'s row-shape constraints. **Follow-up recorded, not yet done:**
neither new invariant is enforced by any checker file's Constraints table
yet; `linter-schema_shape.md` is the natural owner and needs an edit to
add the enforcing rows.

**New backlog item — CLAR-002:** `kinds.md`'s Notes flag that "kind" is
overloaded in this repo — `𝒦`'s five objects vs. the `kind` column that
Constraint and Property rows separately carry. Same shape as the open
`CLAR-001` naming collision; not resolved here, since the fix is a column
rename that should itself go through `rename_naturality` rather than be
done by hand.

**EXCL-001 note:** `property_row_shape`'s `kind ∈ {unit, law}` in
`kinds.md` is now the authoritative place that enum lives — when EXCL-001
(adding an `audit` kind) is actioned, this is the file that gets the new
Revision, alongside `linter-schema_shape.md`.

## #13 — Second Rule-of-5 review, fixes applied

Reviewed the full 11-file corpus, focused on the 3 files added in #12 and
whether Revision 3's fixes held. Found: `STATUS.md` had already drifted —
still labeled `specodelic.md` as "Revision 2" one round after Revision 3
was made, plus two broken `§5` cross-references that should have been `§2`
and `§4`. Also found `AGENTS.md` restated two constraints from
`specodelic.md` as free prose instead of referencing them (a second
source of truth that could silently drift), and `CHANGELOG.md`'s own
historical section headers used pre-rename filenames with no pointer to
the current name. All four fixed: `STATUS.md`'s revision label and both
cross-references corrected; `AGENTS.md` items 2 and 4 now point to
`specodelic.md` instead of repeating its content; stale changelog headers
annotated with current filenames; sequence numbers (this note included)
added to every entry.

## #12 — `AGENTS.md` created

Standing operating instructions for any agent working in this repo,
distinct from `STATUS.md` (current state + plan) and `CHANGELOG.md`
(history): the file-naming rule, the required workflow for adding a spec
file (check against the existing constraint list before considering it
done — four of the first seven checker files needed this), the
after-every-change checklist (update changelog + status, grep for stale
references), and what parts of the pipeline don't exist yet so an agent
doesn't assume otherwise.

## #11 — Revision 3 — Rule-of-5 review fixes

**Files renamed (5):**
- `linter-referential-integrity.md` → `linter-referential_integrity.md`
- `linter-graph-shape.md` → `linter-graph_shape.md`
- `linter-model-shape.md` → `linter-model_shape.md`
- `linter-ears-syntax.md` → `linter-ears_syntax.md`
- `linter-schema-shape.md` → `linter-schema_shape.md`

**Files edited:**
- `specodelic.md` — fixed `id_matches_file` (was not an invertible
  function; hyphens collided between namespace dots and underscores).
  Removed stale "not yet written" notes. Added Revision 3 section.
- `linter-coverage.md` — corrected "seven checker files" claim to the
  accurate count and scope (six in the Checker Ownership table; coverage
  itself gates `compile`, not `lint`).
- `STATUS.md` — logged Revision 3, updated inventory table to renamed
  filenames.

**Trigger:** Rule-of-5 review (`rule-of-5-universal`) of the full corpus,
requested by the user, found 1 CRITICAL, 2 HIGH, 2 MEDIUM, 3 LOW findings.
The CRITICAL and both HIGH findings were fixed above. Two findings remain
open (see Backlog in `STATUS.md`): the `coverage`/`linter.coverage` naming
collision (CLAR-001), and the missing `audit` property kind for
`no_prose_field_parsed`-style claims (EXCL-001).

## #10 — `linter-coverage.md` created

Seventh file. Checks every constraint has a deriving property and every
`law`-kind property has its associativity/identity cases. Gates the
`compile` transition in `specodelic.md`. No new gaps in `specodelic.md`
surfaced — third checker in a row to close clean, after `linter.ears_syntax`
and `linter.schema_shape`.

## #9 — `linter-schema-shape.md` created (now `linter-schema_shape.md`)

Checks variant tables only grow across revisions and that the parser never
branches on prose field content. Scope narrowed from the original plan:
`no_boolean_columns` ended up owned by `linter-model-shape.md` instead,
scoped to the model section specifically. Surfaced one unresolved item:
`no_prose_field_parsed` doesn't fit the generator/predicate property shape
(it's a claim about the parser's implementation, not spec-file content).

## #8 — `linter-ears-syntax.md` created (now `linter-ears_syntax.md`)

Checks the intent `statement` matches an EARS pattern and that row ids
don't encode two capabilities or universal quantifiers. First checker to
surface zero new gaps in `specodelic.md`.

## #7 — `specodelic.md` Revision 2

Decomposing the linter into separate checker files surfaced four gaps in
Revision 1's constraint list: `id_matches_file`, `ref_kind_compatible` (a
Reference Typing table was added to resolve it), `single_root_reachable`,
and the `every_state_used`/`every_transition_valid` pair. The flat
seven-clause `lint` guard was also replaced with the Checker Ownership
table, since the flat conjunction was itself a God-transition.

## #6 — `linter-model-shape.md` created (now `linter-model_shape.md`)

Checks every transition has a guard and the model's states/transitions are
internally consistent (declared states are used, transition endpoints
exist). Surfaced two gaps folded into Revision 2:
`every_state_used`/`every_transition_valid`.

## #5 — `linter-graph-shape.md` created (now `linter-graph_shape.md`)

Checks the `traces_to`/`derives_from` reference graph is acyclic. Surfaced
one gap folded into Revision 2: `single_root_reachable` (acyclicity alone
doesn't rule out an orphaned cluster with no path back to an intent).

## #4 — `linter-referential-integrity.md` created (now `linter-referential_integrity.md`)

Checks id uniqueness (within file and across the repo) and that every
`[[wiki-link]]` resolves. Surfaced one gap folded into Revision 2:
`ref_kind_compatible` had no defined typing table at the time.

## #3 — `linter-frontmatter.md` created

First checker file. Checks frontmatter has `id`/`kind`/`statement` and
`kind == "intent"`. Surfaced a new invariant not yet in `specodelic.md`
at the time: `id_matches_file`.

## #2 — `specodelic.md` Revision 1

Initial version. The meta-spec: `specodelic` described as an instance of
its own format. Defined the core constraint list, the six-state lifecycle
(`draft → parsed → linted → compiled → model_checked → verified`), and one
`law`-kind property (`rename_naturality`) establishing the format's
refactor-safety guarantee.

## #1 — Format design established (pre-file)

Before any file existed: the four-layer structure (Intent / Constraints /
Model / Properties), the choice of markdown + YAML frontmatter + tables +
`[[wiki-links]]` as the concrete syntax, the categorical formalization
(`𝒦`, copresheaves, natural transformations for refactoring, the
Grothendieck construction for cross-file ids), the diagnostician-to-schema
mapping (making specific code smells ungrammatical rather than merely
lint-flagged), and the data-oriented/Clojure-flavored bias (open maps,
namespaced keys, predicates over inheritance).
