# Changelog

Append-only. Past entries are never edited — a correction gets a new entry,
the same discipline `specodelic.md`'s own `append_only_variants` rule
requires of spec files themselves. Displayed newest first; numbered
chronologically ascending (`#1` = oldest) so a new entry always gets the
next integer regardless of where it's inserted in the display order.

## #113 — v0.5.2: executable release tarballs land post-tag; binary re-syncs to corpus Revision 16 (specodelic-6sb, mvl, PR #11)

The v0.5.1 tag shipped before three fixes landed the same day, so its
published artifacts lagged main. This patch releases them together:

- **Executable release tarballs + curl install docs + LLM summaries on
  Pages** (PR #11, `aee950f`): the v0.5.1 tarball assets lost the
  exec bit (upload-artifact v4) and the README curl snippet referenced a
  wrong asset name. Users who downloaded 0.5.1 assets got
  non-executable archives — this is the primary reason for the patch.
- **FORMAT_REVISION 15→16** (specodelic-6sb, `c5d2b39`): the
  property-binding corpus bump (`1ec6b90`) outran the released binary —
  the v0.5.1 binary embeds Revision 15 while docs deployed from main
  describe Revision 16. The binary now embeds 16, with the two new
  deriving Properties (`one_tag_extracts_per_cell`,
  `missing_emitter_labeled`) keeping `linter.coverage` unblocked.
- **`llm.txt` at repo root** (specodelic-mvl, `0158614`): required by
  the ddl sibling-repo conformance rule `s2_llms_txt_at_root`; the cke
  guard is reversed to REQUIRE it (`llms.txt` stays the deploy target,
  not deployed).
- **Gate-drill advisory lock** (specodelic-do8, `5f4335b`): drills hold
  `.beads/gate-drill.lock`; the pre-commit hook refuses commit/push
  while a drill is live — closes the dz4 concurrent-session race's
  hook-side interlock. Also shipped: the commit-hygiene standing
  instruction (specodelic-2g1) and the g17 file-splits/ratchet
  (internal, no behavior change).

No format Revision change (16 already on main); no spec corpus change.

## #112 — graph-views schema view re-pointed at the acset Schema value; docs/governance sweep around the acset landing (specodelic-hya, F7, F9)

The pending `add-graph-views` change's schema view was pinned (design D4,
2026-09-29) to `guide`'s closed value sets — written before the acset
Schema value existed. With add-acset-core landed, following the original
D4 would have created two sources for one table: the lint-gated Schema
and `guide`'s ungated `REFERENCE_TYPING` const.

- **D4 revised** (specodelic-hya, closed 2026-10-04): the schema view
  derives from `acset::schema::canonical()` — the Reference Typing table
  as data, already lint-gated against `specs/specodelic.md` row-for-row
  by `schema_matches_typing_table` — plus the format revision marker.
  The alternative (keep both sources + a build-time cross-check) was
  rejected as redundant: the Schema is already doc-gated at lint time.
- `guide::REFERENCE_TYPING` is demoted to a render-only surface (the
  embedded guide's prose); `spk guide --json` stays in scope for
  value-set-only consumers, and the schema view no longer consumes it.
- **Stale evidence refreshed** throughout the change docs: the corpus's
  38 typing violations (the drafting-era number) are gone — the acset
  landing typed them away, `spk graph` reports 0 as of 2026-10-04 — so
  violation rendering is now proven via fixture corpora, not the live
  corpus.
- README pipeline/status claims re-verified against code (finding F7:
  "not implemented yet" claims for verbs that all ship, and the missing
  Rev 15 executable-fragment mention) — fixed 2026-10-04.
- Finding F9 filed as specodelic-814: the corpus's guard-typing row
  doesn't mention the `**rust:**` executable-fragment escape hatch
  (Rev 15 transition residue); recorded in the external-architecture
  reviews register.

## #111 — acset writer: span-preserving writer over recorded parse spans; rename realized through it (specodelic-p5b, acset slice S3)

`spk rename`'s rewriting was a second, hand-wired edit path beside the
parser — the exact duplication the acset adoption targets. The writer
makes every file rewrite happen through spans the parser recorded.

- The parser now records **byte spans** for id cells, links, and state
  bullets; the writer emits by those recorded spans — identity emission
  is verified corpus-wide (emitting without edits reproduces the input
  byte-for-byte), and edit laws pin identity, composition, and a
  roundtrip gate.
- `spk rename` runs through the writer path (`run_via_writer`); the dead
  hand-wired rewriting was deleted, so there is one edit path.
- The rename parity property (writer-driven rename must equal the
  hand-wired run byte-for-byte, 64-case proptest plus fixtures) caught
  **two real bugs** before the flip: the hand-wired frontmatter rewrite
  corrupted single-char intent ids (`id: i` → `irenamed: i`, an
  unanchored replace — now anchored on the `id: ` prefix), and the
  roundtrip link check misclassified rewritten links when the new id
  has the old id's `old_id.<child>` shape (now a target multiset over
  before-links).

## #110 — acset core: the Reference Typing schema and typed instances as data (specodelic-84i.1, acset slices S1+S2)

The format's Reference Typing table existed twice in code: as a
documented table (`specs/specodelic.md`) and as hand-written enforcement
(a match in graph.rs whose agreement with the table nothing checked).
The acset-core landing makes the schema a **value** and enforcement read
from it.

- `acset::schema::canonical()` — the schema as data: objects, typed
  morphisms (with refinements, source rules, violation prose,
  endo-acyclicity flags) in canonical order. The doc and the code
  cannot drift silently: the lint-time gate `schema_matches_typing_table`
  compares the Schema row-for-row against the corpus doc's Reference
  Typing section (parsed via `Spec.reference_typing_body`), firing in
  either direction; the gate no-ops for corpora without the section.
- Typing enforcement delegates to the Schema (`schema::typing_violation`)
  — the hand-written match is retired, and graph/merge/refactor outputs
  stay byte-identical (parity gates held throughout the landing).
- A **typed instance builder** (`acset::instance`): dangling references
  are values (labeled), no link is dropped, forbidden edges are stored
  as violations and never recorded as edges, duplicate ids resolve
  first-wins parity, rebuild is byte-stable.
- Closure/query primitives (one traversal primitive) replace
  per-command edge walks where adopted.
- The corpus's 38 typing violations (the drafting-era figure cited in
  several openspec change docs) are resolved — `spk graph` reports 0 as
  of 2026-10-04.

## #109 — `**rust:**` mentions are not opt-ins (specodelic-sd1)

The extractor treated ANY occurrence of the marker as an opt-in — including
mentions. The corpus rows that DEFINE the mechanism (compile.md's
`predicate_fragment_opt_in`, `invariant_fragment_opt_in`,
`fragment_law_rejected`, `fragment_guard_rejected`, `properties_to_proptest`;
model_check.md's `executable_invariants_execute`) carry a `**rust:**`
occurrence inside their own expr code spans, and verify.md's
`fragments_reach_verified` carries one in prose between spans — so the prose
after each mention was extracted as a "fragment": committed `compile.tla`
carried 5 bogus `INVARIANT` entries of raw prose, and the scratch runs
hard-failed (`spk model-check specs` exit 101 on 2 files; the verify.md
predicate left a `properties_uncompilable` residual after specodelic-8aq).
The exact bootstrapping-bug shape specodelic.md's Notes warn about: the
tool's own description of the mechanism broke the mechanism.

- Fragment position is now structural: an occurrence opts in only when it
  STARTS the cell (after leading whitespace) or immediately follows an
  opening code-span backtick (`` `**rust:** <expr>` `` — the Revision 15
  authoring form). Any other occurrence — mid-span or in prose between
  spans — is a mention and never extracts. A marker after a CLOSING
  backtick (`` `x` **rust:** y ``) is prose-position: a mention.
- "Exactly one marker" sharpens to "exactly one marker in fragment
  position"; the two-opt-in-span cell still fails labeled.
- Transition-guard rejection now fires only on real opt-ins (a mentioning
  guard cell is prose, never a labeled rejection).
- Spec-side sharpening: compile.md's `predicate_fragment_opt_in` /
  `invariant_fragment_opt_in` rows define fragment position; verify.md's
  `fragments_reach_verified` predicate reworded to not carry the literal
  marker in prose.
- Deliberately NO new advisory: mention-position markers are legitimate
  prose in a spec ABOUT the marker — this corpus proves the pattern; an
  advisory would fire on the defining rows forever. A mis-authored opt-in
  falls back to the honest placeholder (pre-Revision-15 behavior).
- Corpus artifacts regenerated: `compile.tla` / `model_check.tla` now
  declare "No executable invariant fragments"; `spk model-check specs` runs
  20/20 scratch-clean; `spk verify specs` reports 0 uncompilable.

## #108 — parameterless property blocks emit outside `proptest!` (specodelic-8aq)

A Property row whose generator cell yields no strategy params (no
`name(` occurrence — e.g. `` `(a, b)` where one was removed `` prose) was
emitted as a bare `fn name()` INSIDE the `proptest!` macro, which requires
`(pat in strategy)` on every fn — a parse error that left the scratch crate
uncompilable, so `verify` reported `properties_uncompilable` (worse than the
honest todo-panic failure it replaced) for 7/22 corpus files.

- Blocks with no generators now emit OUTSIDE the macro as plain `#[test]`
  fns — same metadata comments (`// id:` / `// case:` / `// generator:` /
  `// predicate:`) so `verify`'s staleness fingerprint and block discovery
  are unchanged, same honest `todo_predicate!` / verbatim-fragment `assert!`
  bodies. Blocks with strategies keep the `proptest!` form; the macro is
  opened/closed around contiguous runs so row order is preserved.
- Law-case splitting is intact: a parameterless law row still yields one
  plain `#[test]` fn per required case.
- The `use proptest::prelude::*;` import is emitted only when at least one
  block carries strategies — a fully parameterless artifact stays
  warning-clean.
- Fragment rows are unchanged (the rjb emission shape); a parameterless row
  opting in with `**rust:**` emits its verbatim fragment as the `assert!`
  body of the plain fn.

## #107 — vocabulary-triggered orphan labeling (the second half of `orphan_vocabulary_labeled`, specodelic-erd)

The pack mechanism's orphan rule fired only on declared `uses` edges —
pack vocabulary used with **no pack discovered and no `uses` edge** fell
through silently (found by the bioimage D6 pilot's probes: `## Data` +
`data.dataset` used with the data.lineage pack absent produced no orphan
finding). `packs.md`'s `orphan_vocabulary_labeled` constraint requires
both halves; the vocabulary half now ships.

- The signal is **structured-position-only**: a pack-qualified (dotted)
  token in a kind cell (frontmatter kind, Constraint/Property `kind`
  cells) or a table column header whose namespace matches no discovered
  pack's namespace. Prose stays unscanned — a dotted token in prose
  (`compile.extraction_failure`, a row-id citation) is not vocabulary —
  so the signal stays false-positive-free and files using no pack
  vocabulary lint byte-identically (`no_pack_no_change` holds).
- A discovered pack in the token's namespace (namespace = the pack id's
  first segment; dotless ids name their namespace directly) means the
  token is in an active-namespace workspace — vocabulary matching and
  the fiber-kind walkers handle it, no orphan fires.
- The finding names the candidate pack **prefix-derived when only the
  namespace is known** (`data.dataset` → candidate `data.*`) and both
  remediations (add/enable a `kind: profile` pack declaring the
  vocabulary, or fix the vocabulary) — never a generic dangling message,
  never silent. A concrete `uses`-edge orphan for the same namespace
  wins (it names the exact pack); the prefix-derived finding does not
  duplicate it.

## #106 — executable predicate fragments (specodelic.md Revision 15, specodelic-rjb)

The verified gate becomes reachable: `specodelic-mp1` row 7's option C
ships as a format Revision. A Property's `predicate` cell or an
invariant-kind Constraint's `expr` cell may opt into executable
translation with exactly one `**rust:**` marker — the cell text after it
is a Rust boolean expression emitted **verbatim** (no mini-language;
Property fragments bind the generator values `v0…` as `String`s,
invariant fragments bind the current state's id). Cells without the
marker behave byte-identically to Revision 14 — pure widening.

- **compile.md** grows the fragment contract: `predicate_fragment_opt_in`,
  `invariant_fragment_opt_in`, and the three labeled rejection rows
  (`fragment_law_rejected` — one fragment cannot honestly serve a law
  row's several named cases; `fragment_guard_rejected` — executable
  guards have no binding over the program-counter model, deferred of
  record; `non_invariant_fragment_labeled`) plus `fragment_hygiene` — the
  banned-token security/totality guard (`unsafe`, `extern`, `include!`,
  `std::fs`, `std::process`, `std::net`, `std::env`, `asm!`, `Command`),
  defense-in-depth, not a sandbox. The proptest scaffolding's element
  type simplifies from the `GenVal` wrapper to plain `String`.
- **model_check.md** grows the execution contract:
  `executable_invariants_execute` (the native backend executes every
  executable invariant as a dependency-free scratch-crate BFS run —
  engine attribution `native-bfs`, BFS keeps `counterexample_is_minimal`
  by construction) and `invariant_totality` (a panicking fragment is a
  violation, never a pass). `rerun_on_model_change` extends to fragment
  edits: the module now carries the fragment manifest as comments, so the
  artifact hash covers them. TLC still reports `exploration_only` against
  fragment artifacts — it cannot execute Rust and never claims clean.
- **verify.md** pins the reachable verdict: `fragments_reach_verified` —
  both gates clean on a fragment fixture is `verified` at last;
  `both_gates_required` and every honesty rule untouched.

`specodelic.md` carries the Revision 15 decision-of-record heading; no
row of its own tables changes.

## #105 — the D6 pilot: bioimage-data domain pack (add-bioimage-pack)

`add-bioimage-pack` implemented — no code, no base-set change: the
mechanism's thin domain instance (the decision of record's D6 pilot)
ships as an in-repo `kind: profile` artifact at
`packs/bioimage-data.md` (id `bioimage.data`, base pin `specodelic.md`
Revision 14) and the mechanism's **first cross-pack `## Requires`
consumer** (deps: `data.lineage`, `numeric.predicates`,
`empirical.registry`). It declares a namespaced `## Axes` section (row
shape `| name | axis | scale | unit |` — OME-NGFF-style axis semantics
with opaque `scale`/`unit` columns, bridge-never-absorb), the
`bioimage.transform` property kind (the first fiber kind whose natural
home is a base-table kind column — riding `specodelic-ung`'s
base ∪ active-pack-fiber walkers), the `same_shape_as` reference field
resolving to a `## Data` row of the declaring file (cross-pack
dependency, intra-file resolution — no reachability join, no acyclic
edge set), the R2 data-shaped predicate grammar declared as named
pack-qualified checkers (`bioimage.dtype_is`, `bioimage.shape_eq`,
`bioimage.units_convertible` — the mechanism names them, never
executes them; `specodelic-rjb` owns the executable-fragment track)
plus the `bioimage.same_shape_closed` closure checker (the
`data.lineage.closure` precedent), and the kind-dependent transform
floor (`**preserves:**` + `**dtype:**` — the base `law` floor's
non-universality critique made concrete; purely additive,
byte-identical with the pack discovered).
Vocabulary hygiene pre-paid a third time: every declared token is
dotted or verified absent (`Axes`, `same_shape_as`, `preserves` 0-file;
bare `bioimage` documented asymmetry; bare predicate tokens never
declared — they ride the checker rule names; `pipeline` prose-heavy →
rejected, `within` prose-toxic → excluded; `dtype` survives only as an
opaque column header and floor label). Pilot probes verified: orphan
`uses: [[bioimage.data]]` with the pack absent → labeled
`linter.orphan_vocabulary` exit 1; `kind = bioimage.transform` accepted
with the pack active, labeled finding without it; base `law` floor
byte-identical with all four packs discovered; corpus lint
byte-identical except the delta's own known activation. Probe gap
filed: vocabulary-triggered orphan labeling (no pack, no `uses` edge)
is mechanism work, not this pack's.

## #104 — pack fiber kinds are typeable in base-table kind columns (specodelic-ung)

The domain-pack mechanism's closed-set walkers (`property_kind_closed` /
`constraint_kind_closed`) walked the static base sets, so a
pack-declared fiber kind (e.g. `empirical.statistic`) fired the finding
even with its pack discovered. The walkers' effective set is now
base ∪ active-pack-fiber — the `## Kinds` tokens of every pack active
for the file (declared `uses` edge or vocabulary match, the standard
advisory-first activation rule) — never narrower than the base set: base
set members are accepted unconditionally, the base sets themselves do
not grow (no format Revision; the packs' Revision 14 base pins stay
valid), and a fiber kind with no active declaring pack stays outside the
closed set, firing the labeled finding — never a silent pass. A pack
activates per-file, so a file using none of a pack's vocabulary keeps
byte-identical lint findings. Recorded in `specs/packs.md` (Notes +
`fiber_kinds_typeable` property), `specs/linter-schema_shape.md` (both
closed-set rows + Notes), and `specs/kinds.md` (row shapes + Notes).
Implementation: `packs::active_fiber_kinds` (workspace scan shared with
discovery) wired into the schema-shape family; the empirical-registry
pack's `fiber_kinds_typeable` contract is now enforced end-to-end.

## #103 — the third standard pack: empirical registry (add-empirical-registry-pack)

`add-empirical-registry-pack` implemented — no code, no base-set change:
the statistical half of the R4 vendor convergence (mistral ×2, grok, zai)
ships as an in-repo `kind: profile` artifact at
`packs/empirical-registry.md` (id `empirical.registry`, base pin
`specodelic.md` Revision 14). It declares a namespaced `## StatTests`
section (row shape `| name | metric | alpha | window |`), the
`empirical.statistic` property kind, the `tested_by` outbound-leaf
reference field, the `empirical.statistic_labels` and
`empirical.stat_test_closed` checker declarations (honest-empty), and
the per-kind case-label floor (`**alpha:**` + `**window:**` — the
Floors facet's first real consumer, reusing the `**name:**`
enumeration machinery with a different required label set than the base
`law` floor; floors are per-kind in the pack fiber, purely additive —
the base `law` floor is byte-identical with the pack discovered, and a
law-kind row may carry the labels additively, floor-not-ceiling).
Vocabulary hygiene pre-paid twice (the D6 lesson): no bare-English
token is declared — `window` appears in corpus prose and is excluded
from every vocabulary-carrying facet (floor label + column only,
neither in `vocabulary()`); `StatTests`, `tested_by`, `alpha`, bare
`statistic` verified absent; bare `empirical` cannot falsely activate
(dotted tokens cannot match prose words). Orphan probe verified: a
declared `uses: [[empirical.registry]]` edge with the pack absent
produces the labeled `linter.orphan_vocabulary` finding naming the
candidate pack and both remediations, exit 1. A fiber kind in a base
Properties `kind` column is not typeable under mechanism v1
(`property_kind_closed` walks the static base set) — the mechanism
follow-up is `specodelic-ung`, never silently passing. Completes what
the bioimage D6 pilot (`specodelic-0dn`) consumes; its proposal sits at
the approval gate as `openspec/changes/add-empirical-registry-pack/`.

## #102 — the second standard pack: numeric predicates (add-numeric-predicates-pack)

`add-numeric-predicates-pack` implemented — no code, no base-set change:
the numeric half of the R1/R2/R4 vendor convergence ships as an in-repo
`kind: profile` artifact at `packs/numeric-predicates.md` (id
`numeric.predicates`, base pin `specodelic.md` Revision 14). It declares
a typed `## Quantities` section (row shape `| name | kind | unit |
domain |`), the namespaced `numeric.quantity` / `numeric.bound` /
`numeric.tolerance` kinds, the `measured_by` outbound-leaf reference
field, the `numeric.quantity_closed` and `numeric.tolerance_labels`
checker declarations (honest-empty), and the tolerance case-label floor
(`**bound:**` + `**against:**` — R4's reuse phrasing; floors are not in
`vocabulary()`, so the labels never trigger activation). Unit systems
(UCUM, ISO4217, microscopy) stay opaque in `unit`/`domain` —
bridge-never-absorb, lint byte-identical across systems.
Vocabulary hygiene pre-paid (the data pack's D6 lesson): no bare-English
token is declared — `within` (13 corpus files + 7 dual-format files),
`bound`, `against`, `unit`, `domain` are excluded from every
vocabulary-carrying facet; corpus lint issues+warnings verified
byte-identical with both packs discovered. A tolerance *kind* here is
pack-fiber-relative — the general per-kind floor registry is
`specodelic-aal`'s separate proposal. Completes a third of what the
bioimage pilot (`specodelic-0dn`) consumes.

## #101 — the first standard pack: data/lineage (add-data-lineage-pack)

`add-data-lineage-pack` implemented — no code, no base-set change: the
first standard pack on the Revision 14 mechanism ships as an in-repo
`kind: profile` artifact at `packs/data-lineage.md` (id `data.lineage`,
four-layer spec in its own right, base pin `specodelic.md` Revision 14).
It declares a typed `## Data` section (row shape `| name | kind | dtype
| units | binding |`), the namespaced `data.dataset` / `data.artifact` /
`data.environment` kinds, the `produced_by` / `consumed_by` lineage
reference fields (each resolving to a `## Data` row of the declaring
declarative file — outbound leaves, intra-file), the
`data.lineage.closure` checker declaration (honest-empty), and the
`data.artifact` provenance floor. The `binding` column stays opaque:
external standards (OME/NGFF, Parquet, BioImage.IO) are bridged, never
absorbed — lint is byte-identical regardless of binding target.
Deviation from the R3 trio, recorded in the change's design D6: bare
`produces` is NOT declared — the word appears in corpus prose and the
mechanism's whole-word vocabulary match would falsely activate the pack
on every lint; `append_only_packs` lets a later release add it. The
corpus lints byte-identically with the pack discovered (issues +
warnings diffed equal); `data.lineage` is now the enablement target the
bioimage pilot (`specodelic-0dn`) consumes via `uses`.

## #100 — the extension mechanism becomes first-class: domain packs (specodelic-dcx)

`add-domain-pack-mechanism` implemented (Revision 14). A domain pack is a
four-layer spec file with `kind: profile` frontmatter whose six manifest
tables (Sections/Kinds/References/Checkers/Floors/Requires) declare
exactly what the pack introduces. Discovery is a corpus scan anchored at
the git toplevel; opt-in is advisory-first, triggered by vocabulary use,
upgradeable to a declared `uses` edge (set-valued, targets the pack's
frontmatter id); a `uses` edge to a pack that is not discovered is a
labeled orphan failure naming the candidate pack and both remediations.
The pack-authored `## Requires` table pins the base format revision —
skew against the workspace corpus revision is a warnings-channel
advisory, never silent, never failing. Lifecycle states draft/published/
deprecated are read from the pack's Model (full machine = published;
single state = that state); draft findings name the draft status and
deprecated findings name the deprecation. Pack files are self-exempt
(their manifest rows never activate checkers or produce orphan
findings). Files without packs lint byte-identically: the `packs` data
key surfaces only when a pack is discovered. Three lint rules added
(append-only): `linter.pack_shape`, `linter.orphan_vocabulary`,
`linter.skew_advisory`. V1 scoping: orphan detection is typed (the
declared `uses` edge), not a prose-token heuristic — honest-empty beats
over-reporting; pack checkers ship as declarations reporting an empty
checked-set. `spk doctor` surfaces discovered packs.

## #99 — corpus prose sweep: checker counts reconciled to eight, non-gating checkers named, copresheaf anchor fixed (specodelic-5f2)

Rule-of-5 corpus review (2026-10-01) CLAR-001 (HIGH) + EDGE-001 (MEDIUM)
+ CLAR-002 (LOW) — a prose-only sweep, no schema change, no Revision
heading (corrections of record per the review's own scope note):

- **CLAR-001 — checker counts reconciled.** `specodelic.md`'s prose
  said "the join point of seven" while its own Checker Ownership table
  lists 8 rows (failure_shape joined 2026-09-30, specodelic-hhp); the
  prose now says eight. `STATUS.md` §1's reading-order step 6 said
  "the six checker files" and omitted `linter-failure_shape.md`; it now
  names all eight gating rows including failure_shape (coverage
  noted as the join leaf read with the tooling in step 7), and the
  Revision 3 narrative's historical count is stamped as historical
  ("had six then; eight gating rows today"). Two further stale counts
  the review's line numbers missed, same class, fixed in the same
  sweep: `theory.md`'s limit prose said "all six checkers" and
  `USAGE.md` §2.3 "this repo's six checkers" — both now "eight gating
  checkers"; `linter-external_completeness.md`'s "Unlike the six
  checkers" → eight; `linter-failure_shape.md`'s "Ownership is
  deliberately absent ... ships no table row" note was stale since the
  row landed (specodelic-hhp) — rewritten as "was deliberately absent,
  then granted" with the decision of record; and
  `linter-coverage.md`'s "All six checker files ... now exist" (a list
  missing failure_shape and coverage itself) → all eight, coverage
  included.
- **EDGE-001 — non-gating checker files named at the table.** A
  one-line note under the Checker Ownership table now states that
  `linter-observability.md` (advisory, warnings channel, exit 0 by its
  own `advisory_severity`) and `linter-external_completeness.md`
  (runs only when a repo declares a checklist; never gating per
  `orchestrate.md`'s `external_completeness_never_gating`) exist as
  instances of the same format but sit outside the join on purpose —
  their absence is a membership statement, not an oversight.
- **CLAR-002 — broken copresheaf anchor.** `specodelic.md`'s document
  instance link targeted
  `theory.md#document-instance-functor-copresheaf`, but the heading
  "Document instance (functor / copresheaf)" slugs with a double hyphen
  in every renderer — broken on the deployed docs site too. Heading
  reworded to "(functor and copresheaf)" and the link updated to match.
  A same-class dead example anchor in `theory.md`'s own link-usage
  note (`#naturality` — no such slug; the heading is "Naturality (safe
  rename)") fixed to the real one.

Verification: table-row count == prose count (grep `^| \`linter-` = 8);
no "six checker" in `STATUS.md`; an ad-hoc anchor check over all
intra-corpus markdown links (heading-slug match, CHANGELOG's historical
entries excluded) reports zero broken anchors; `just lint-specs` 0
issues (prose_untouched holds); `just ci` green.

## #98 — kinds.md Revision 7: reject guard de-tautologized; optional frontmatter keys declared (specodelic-x4w)

Two spec bugs from the Rule-of-5 corpus review (2026-10-01, CORR-002 +
CORR-003, both MEDIUM), fixed together in `kinds.md` Revision 7 —
rewordings that narrow nothing:

- **CORR-002 — `reject`'s guard was a tautology.**
  `¬(intent_row_shape ∧ … ∧ property_row_shape)`: no single row can
  satisfy all five mutually-exclusive shape conditions, so the
  conjunction was always false and `reject` fired unconditionally from
  `shape_checked` — well-formed rows had both `accept` and `reject`
  enabled (non-deterministic model). The guard now negates the
  applicable disjunct (`¬((row.kind==Intent ∧ intent_row_shape) ∨ …)`) —
  exactly one disjunct applies per row. The same-shaped `reject` in
  `linter-graph_shape.md` is correct as-is: its four conjuncts co-apply
  to one artifact.
- **CORR-003 — `intent_row_shape` now acknowledges
  optionally-declared frontmatter keys.** It read "exactly
  {id, kind, statement}", but ten corpus files carry the fourth key
  `checked_against_core` (the `AGENTS.md` convention), documented but
  undeclared. Base fields stay exact; optional keys are acknowledged,
  mirroring the optional-typed-columns carve-out `constraint_row_shape`
  got in Revision 5. No linter change: `frontmatter_valid` is a subset
  check and stays one — the spec text is the only guard, and the
  pinning fixture `extra_frontmatter_key_lints_clean` (tests/cli.rs)
  asserts the tool-level reading (lints clean; the parser captures the
  key as an extra field rather than dropping it).

## #97 — `single_root_reachable` enforces the Revision 10 own-file reading, tiered (specodelic-erb)

The shipped checker had drifted from its spec (Rule-of-5 CORR-001,
2026-10-01 review): it implemented undirected connectivity to SOME intent
row corpus-wide, while `specodelic.md` Revision 10 (HITL mp1 row 8) had
reworded the invariant to own-file primary linkage — a row reaches its
file's OWN intent through `traces_to`/`derives_from` chains resolved
within the file, and cross-file typed edges (`guard` citations of foreign
constraints, `satisfies`, `observes`) are outbound leaves, never
reachability paths. The loose reading was the exact bootstrapping bug the
format's Notes warn about: a cross-feature reference filed under the wrong
id false-resolved through another file's intent and passed silently —
the self-lint was structurally blind to it.

Now the checker builds the own-file edge set per file (plus the model's
from/to edges — connectivity, not outbound-only, so non-emitting states
are not flagged) and requires every row to reach its own intent within
it. Enforcement is tiered exactly as Revision 10 specifies: a row with no
own-file path whose full-graph component still contains some intent row
produces an advisory warning on the warnings channel (exit 0 — "anchor
them to this file's intent, or they may be filed under the wrong id"); a
row with no path to ANY intent at all remains an orphaned island and
hard-fails. `RULE_TABLE`'s semantics string (and therefore `spk explain
lint-rules`) now states the own-file reading + tier note, replacing the
superseded loose wording.

Corpus effect: zero hard failures (the cxq reconciliation holds) — three
law rows (`linter.coverage.coverage_naturality`,
`linter.external_completeness.mapping_naturality`,
`linter.referential_integrity.rename_naturality`) now ride the advisory
tier by design: they derive from `[[specodelic.rename_naturality]]`
cross-file, the "same claim, two altitudes" restatement pattern, so their
only tie IS cross-file — the advisory tier is their sanctioned home (the
converse anti-goal — hard-failing legitimate cross-file structure — is
what Revision 10's tiering exists to prevent). The conformance case
`graph_typing_traces_to_constraint` gains the advisory expectation: the
rows stranded by the wrong-file `traces_to` are no longer silently
false-resolved through the foreign intent.

## #96 — deployed llms.txt carries a version stamp; undeployed llm.txt variant resolved (specodelic-2m7, specodelic-cke)

The deployed site rebuilds from main, but `llms.txt` carried no version
identifier — an agent writing specs from the deployed grammar could target
a format newer than its installed `spk` crate, and lint failures were then
undiagnosable (Rule-of-5 EDGE-001, 2026-10-01 session). Separately, the
repo root had an `llm.txt` richer variant (install + Links sections) that
was deployed nowhere (404) and referenced by no build recipe — pure drift
surface.

Now: `scripts/stamp_llms.py` runs at docs build time in BOTH paths (the
docs.yml workflow and `just docs-build` share the script, so they cannot
drift). The `page` phase generates `docs/src/release.md` from Cargo.toml
+ git ref before `mdbook build` (version claims never hand-typed — §6,
DDL-j0u); the `llms` phase copies `llms.txt` into `book/llms.txt` after
the build and appends a Release Status stamp — version, built-from ref,
and a link to the generated Release Status page. The repo-source
`llms.txt` is never touched (it stays the static, versionless input).
The `llm_txt_served` invariant is unaffected — the deployed copy is still
the tool summary at the site root. The undeployed `llm.txt` variant is
deleted; its unique content (Installation, Links) is merged into
`llms.txt`. Guards in `scripts/test_stamp_llms.py` (stdlib unittest,
wired into `just ci` via the discover recipe): stamp content, source-file
immutability, and the drift guards (no `llm.txt` at repo root; llms.txt
carries the merged sections).

## #95 — graph dangling messages for consumption edges are interface-shaped (specodelic-2q8)

A dangling typed consumption reference (`satisfies`, and by symmetry
`observes`) reported the generic dangling shape — `consumer →
[[producer.nope]]` — which left a reader unable to tell "typo'd row id"
from "consuming a contract nobody published"; the remediation differs
(fix the id vs publish the row). Found by the contract-wiring spike
(specodelic-x30, case p2a); the spike's review pass endorsed exactly
this follow-up and no other.

Now: when the failing link's column is a consumption column, the graph
dangling entry is interface-shaped — `consumer.hooks_installed
(satisfies) → [[producer.nope]]: no published contract row
producer.nope exists — publish it in the producer's file or fix the
id`. It names the consumer row, the consumption column, the missing
row, and BOTH remediations without claiming which applies (honesty
rule: structurally the two causes are the same fact). Non-consumption
columns (`traces_to`, `derives_from`, `guard`, `transitions.*`,
`supersedes`, `emits`) keep the generic shape. The wrong-kind path
(Reference Typing violations) was already interface-shaped and is
untouched. No spec row pins the dangling message format; `graph.md`'s
`total_extraction` semantics are unchanged — this is message polish
riding the existing dangling machinery, not a new checker (the spike
rejected all four candidate checkers).

## #94 — `spk lint` fails on parse errors, even alongside clean files (specodelic-in9)

A malformed-frontmatter file in a batch that also held parseable specs
rode the warnings channel of a success envelope — exit 0 — so the
pre-commit gate let a corrupted spec commit (empirically verified with
a gate-probe commit). The exit-2 failure only fired when NOTHING in
the batch was linted.

Now: parse errors fail the lint stage (exit 1) whenever any are
present, with a next-step hint naming that the file was NOT linted;
the error still names the offending file on the notes channel.
Single-malformed-file invocations keep their labeled invocation
failure (exit 2). Commands that consume `parse_batch` directly
(orchestrate) already threaded parse errors into their stage results
— the gap was lint-only.

## #93 — `linter.requirement_drift` compares per requirement, not per section (specodelic-eh0, GH#8)

`requirement_drift` (gh#4's rule) compared the `## Requirements`
mirror against each delta section with whole-section normalized
equality — which made a dual-format file carrying BOTH `## ADDED
Requirements` and `## MODIFIED Requirements` unsatisfiable: the
mirror can never equal both sections at once. Reported from the
consumer side (espectacular's CI went red when the rule was widened
to MODIFIED; their mixed-delta
`2026-09-30-adopt-genesis/specs/cli/spec.md` has no satisfying
mirror under 0.3.0).

The rule now compares per requirement: each `### Requirement:` in
every carried delta section must appear in the mirror with identical
normalized text (same trailing-space/blank-line normalization as
before). Single-section semantics are unchanged — a drifted
requirement still fires, now naming the requirement and its delta
section in the finding. RULE_TABLE semantics updated to match.

## #92 — `spk parse`: the typed Spec IR exits the crate (specodelic-9rv)

The four-layer grammar was fully parsed in-process — `spec::Spec` is a
typed, `Serialize`-derived IR — but only from inside the crate:
external consumers had to reimplement the markdown-table parser (a
silent drift vector against format revisions) or scrape `spk compile
--json`, which embeds generated artifact text and exposes no
Properties rows as structure. `spk parse <file>` is the minimal
exposure: the parsed `Spec` emitted directly as a json envelope —
intent, constraints, states, transitions, properties (cells as keyed
maps), structured links — nothing filtered, no new IR. Parse is
syntax-only: it succeeds on lint-dirty files and embeds no lint status
(single-purpose envelopes; consumers chain `spk lint` themselves, and
the envelope's hint names the command). Exactly one file per
invocation — no globbing; consumers loop. Unparseable, missing,
non-regular (FIFO/device), and >2 MiB input are labeled error
envelopes with remediation hints and non-zero exit (the suz
hostile-input discipline; placeholder parse-error labels are remapped
to the real path). Requested by espectacular (`ah sync` consumes the
IR through the versioned-binary boundary); docs/src/commands.md
carries the reference, and the parse capability spec now lives at
openspec/specs/parse/spec.md.

## #91 — law-row named cases are machine-checkable: `linter.law_cases` (specodelic-9qw)

Law rows are monoid witnesses, but where their cases appeared in the
predicate was free prose — the linter could not tell a declaration from
a mention. `specodelic.md` **Revision 13** ratifies the machine form
compile's `required_law_cases` always parsed: required cases are
`**name:**` case labels in the row's own predicate, the identity +
associativity floor mandatory, extras first-class. The new append-only
rule `law_cases` enforces the floor at lint time (shared
`spec::law_case_labels` helper — compiler and linter cannot disagree),
ahead of compile's precondition gate, whose unlabeled fallback is now
defense-in-depth only. The repo's first `## MODIFIED Requirements`
delta shipped alongside: the dual-format mirror rules
(`requirement_drift`, `dual_format_valid`, `check_section_sync.py`)
widen additively to MODIFIED-carrying files. Dogfood: the rule
immediately caught three corpus law rows living on prose
(`coverage_naturality`, `mapping_naturality` — missing associativity;
`topo_sort_naturality` — missing both); each now carries
rename-instantiation floor cases and compiles one proptest block per
case. Companions: USAGE §2.10 (derived parallelism — the accumulator's
algebra licenses the architecture) and the batch-resume worked example
promoted out of /tmp to `docs/src/examples/`, its law rows relabeled to
the machine form.

## #90 — spk archive-companion: dual-format survival of the openspec archive round-trip (specodelic-fzo, GH#7)

`openspec archive` strips the specodelic half when merging deltas;
the hardened recipe is now a tool: `spk archive-companion <id>` runs
`openspec archive <id> --skip-specs --yes`, verifies each archived
delta carries the layer (frontmatter + `## Constraints`), and deploys
it verbatim to `openspec/specs/<cap>/spec.md`. Fail-closed on any
stripped delta (the tool never deploys one); idempotent re-run skips
the invocation; newest archive dir wins and is named; `--dry-run`
resolves without invoking or writing; empty delta set is a stated
success. `just archive-change` delegates via `cargo run` (never a
stale PATH spk). Dogfood proof: the add-archive-companion change
archived itself through its own command. 16 module tests (injectable
runner seam, zero spawned processes) + 3 integration tests.

## #89 — SUMMARY.md forbids absolute-URL entries (specodelic-j0m)

mdbook 0.5 materializes absolute-URL SUMMARY entries as literal
`src/https:/...` directories — the ecosystem-map entry added with the
landing page (DDL-frh) produced `docs/src/https:/charly-vibes.github.io/...`
at build time. Fix: the ecosystem link moved from SUMMARY.md into
`docs/src/index.md` prose (new **Ecosystem** section), and the
summary-completeness gate (specodelic-b3p's checker) now FAILS on any
`http(s)://` link target in SUMMARY.md with a move-it-into-prose hint —
the regression cannot recur silently. Relative links unaffected.

## #88 — doctor carries a crates.io update-availability notice (specodelic-4le)

`spk doctor` now consults genesis `update_check` (binaries, not libs,
notify — genesis-2ex) with our own crate name + version: a newer stable
specodelic on crates.io rides the warnings channel with the actionable
geness notice (`specodelic X.Y.Z available — you have … (cargo install
specodelic)`), same advisory discipline as the knowledge-currency
warning. 7-day cache TTL (`$XDG_CACHE_HOME`/`$HOME/.cache` →
`genesis/update-check/specodelic.json`); transport failures are silent
by the genesis contract — the doctor never fails on the network. CI
runs and `GENESIS_NO_UPDATE_CHECK` skip the check entirely.

Doctor-only by design: a startup-time check would tax every invocation
(revisit only if doctor-only proves invisible).

Gotcha fixed en route: XDG_CACHE_HOME is itself the cache home — the
wiring initially appended `.cache` to it unconditionally, sending the
check to a nonexistent directory (silent) — caught by the hermetic
CLI test, pinned with `doctor_warns_when_a_newer_version_is_on_crates_io`.

## #87 — `spk init` registers specodelic in `.genesis/tools.toml` (specodelic-las)

`spk init` now declares specodelic's presence via
genesis `discovery::register` — `.genesis/tools.toml` gains a
`[tools.specodelic]` entry with a `file` detector on `AGENTS.md`, so
orchestrators (wai) discover the tool without hardcoding. `register`
creates the file when absent and merges without clobbering sibling tool
entries; re-running init is idempotent (no duplicates). A failure
(e.g. an unwritable `.genesis`) becomes a warnings-channel note — the
AGENTS.md block stays the primary payload and init still succeeds.

## #86 — `spk doctor` runs on the genesis doctor framework, gains `--fix` (specodelic-sok)

The three issue checks (specs/ directory, beads, SPECODELIC block) are
genesis `DoctorCheck` impls run through `DoctorRunner`; workspace facts
(mode, core format spec, corpus discovery) render from detail helpers —
a consumer workspace is a legitimate state, not a finding. Genesis maps
Warning/Advisory to warn and only Error to fail, so the doctor
capability's warn-never-fail invariant holds by construction.

New `spk doctor --fix`: the SPECODELIC block check auto-fixes via
`blocks::inject_into` (the same idempotent code path as `spk init`) and
the runner verifies after fixing — an uncured fix is reported as a
failure, honestly. Envelope shape, currency warnings, next-step
footers, and exit 0 unchanged; all 10 existing doctor integration tests
pass unchanged.

## #85 — hooks wiring migrated onto genesis `ensure_command_wired` (specodelic-x56)

The consolidation ticket closed the loop: genesis-vibes 0.10.0 ships
`lefthook::ensure_command_wired` — the two-case anchor (in-mapping
insert at the mapping's entry indent, wrapper for missing/empty/
commands-less stages) upstreamed from this repo's donor implementation
(genesis-au8). `src/hooks.rs` install() now delegates to it; the local
`wire_wrapper` / `insert_inside_commands` surgery and the private-helper
mirrors (`find_anchor`, `stage_section`, `children_indent`,
`find_commands_key`, `is_column_zero_key`, `wrapper_at`,
`wired_entry_at`) are deleted. The framework gate (husky/prek refusals)
and uninstall stay local; genesis errors map onto the module's labeled
`HooksError` variants, preserving remediation hints.

Behavior is byte-identical, pinned by golden install/uninstall
round-trip tests over 2-space and 4-space configs committed before the
migration (so the pre-migration bytes were the baseline the migration
had to reproduce). 21 hooks unit + 6 CLI integration tests green.

## #84 — failure_shape joins the Checker Ownership table and the lint stage (specodelic-hhp)

The ct5 deferral resolved, **decision (a)**: the failure-shape checker
now has its Checker Ownership row and rides the orchestrator's lint
stage — it was already enforced on every `spk lint` pass (ct5) but had
no ownership row and no orchestrate entry, so `spk orchestrate`'s lint
stage under-reported the checkers that actually gate `linted`.

Wiring: `lint::failure_shape_findings` (the checker's engine-side
invocation, over the same `lint_failure_shape_family` slice the flat
pass runs) joins `run_lint_stage` as branch A's tail, dependent on
`linter.model_shape` — the failure-shape walk reads the Model's states,
transitions and emits edges, so it runs only once the model is proven
well-formed (`dependency_respecting_skip`); a model_shape failure skips
it with the dependency's status, and its verdict joins the stage's
conjunction. The stage doc comment and the schema_shape entry's stale
"tracked coverage gap" basis note (superseded by 7h8) were corrected in
the same stroke.

Spec changes: `specodelic.md`'s Checker Ownership table gains the
`linter-failure_shape.md` row (owns the three rules, depends on
`linter-model_shape.md`) and the join-point prose now says seven checker
files; `linter-failure_shape.md`'s "deliberately deferred" note is
replaced by the decision of record.

TDD: 1 new orchestrate test (clean corpus → checker present and passed;
mute failure terminal → checker failed and the stage fails) + the
dependency-skip and checker-list tests extended. Gates: just ci +
lint-specs 0 + openspec strict.

## #83 — constraint_kind_closed + property_kind_closed as real table-walking lint rules (specodelic-7h8)

The 8kk dogfood's honest-stub finding, closed:
`linter-schema_shape.md`'s `constraint_kind_closed` and
`property_kind_closed` invariants were declared but never enforced as
lint rules — `spec.rs` parses row `kind` as `Option<String>` with no
validation, `schema_shape_findings` returned an honest empty vec, and a
Constraint row with kind `made_up` (or a Property row with kind
`audit`) passed `spk lint` silently.

Two table-walkers now enforce the closed sets from the same constants
the scaffold and the primer render from (`guide.rs`'s
`CONSTRAINT_KINDS` / `PROPERTY_KINDS` — the enforced values and the
documented values cannot disagree): an unreadable (absent) kind cell is
outside the closed set by the invariant's own reading, and the finding
names the offending kind plus every closed-set member. Wired into
`lint_one`'s composition last (frontmatter → referential → model_shape
→ ears → failure_shape → schema_shape), so `spk lint` and the
orchestrator's `linter.schema_shape` checker (already wired) both run
them; `spk explain lint-rules` and the `spk init` managed block pick
the rows up automatically from `RULE_TABLE` (append-only growth).

TDD: 3 unit tests (made-up constraint kind, made-up property kind,
missing kind cell) + a fixture-corpus entry keeping the
catalog-covers-every-emittable-rule test true.

## #82 — constraint-level `derives_from` declared out-of-format; `acyclic`'s edge set qualified (specodelic-huf)

The vv8 conformance matrix's `constraint_derives_cycle` gap, closed by
**spec decision** (the ticket's option c, with the typing hole closed
properly): `linter-graph_shape.md`'s `acyclic` invariant read
"traces_to ∪ derives_from ∪ guard-as-edge has no cycle" — unqualified —
while the checker built the edge set from (constraints, traces_to),
(properties, derives_from) and (transitions, guard) only. A
constraint↔constraint derives_from cycle escaped both the acyclic check
and edge typing (which read only the Must-resolve-to column).

**Decision of record: constraint-level derives_from is out-of-format.**
A constraint is derived FROM by properties; it does not derive. The
Reference Typing table's **Appears on: Property** column is normative,
not descriptive. `ref_kind_compatible` now reads it source-side for
`derives_from` (`typing_violation` in `graph.rs`) — the same way it
already read `supersedes`' same-kind rule — so a Constraint-row
derives_from is a labeled typing violation recorded as no edge. The
cycle is unrepresentable, not merely uncycled; `acyclic`'s edge set
stays closed over well-formed edges, and its invariant text now names
the exact implemented set.

Spec changes: `linter-graph_shape.md` `acyclic` expr qualified + a
decision note (the edge set grows only under a new Revision, same
discipline as the `observes` exclusion) + a pinning property
(`derives_from_edges_property_sourced`); `specodelic.md` gains
`constraint_derives_from_rejected` (unit, `ref_kind_compatible`) pinning
the Appears-on reading beside `supersedes_cross_kind_rejected`.
Corpus artifacts regenerated; no Revision bump — no normative row
changed, the table was already normative on this point.

TDD: integration test `constraint_row_derives_from_is_a_typing_violation`
(red: the edge was recorded, no violation fired) + the matrix case
flipped from `conformance_matrix_known_gaps` into `cases()` with its
amended spec-true expectation (`find:linter.single_root_reachable` +
`graph.typing` — the island condition persists, the cycle does not).
The known-gaps harness is now empty; its flip pattern stays documented
in the file.

## #81 — `no_orphan_property` resolves the `derives_from` target, not just its presence (specodelic-rk3)

The vv8 conformance matrix's `orphan_property` gap, closed:
`linter-coverage.md`'s invariant reads "`p.derives_from` **resolves to a
real constraint**", but the checker only verified that a `derives_from`
link *exists* — a unit property deriving from a non-law **Property** row
passed silently (only the graph layer's `edge_kind_matches_typing`
flagged it).

`lint_coverage` now resolves each `derives_from` target the way
`total_refs` does (`file.row` by file, bare row own-file-first then
corpus-wide) against a `(file_id, row_id) → kind` map, and fires
`no_orphan_property` when the target resolves to a Property row and the
source property is not `law` — the cxq law-restates-law edge stays the
one sanctioned same-kind derivation. Unresolved targets remain
`total_refs`' beat (the invariant restates it, scoped to this edge) — no
double-firing.

TDD: 3 unit tests (target-kind rejection naming the property + required
kind, law→Property passes, dangling target stays `total_refs`' beat);
the matrix case flipped from `conformance_matrix_known_gaps` into
`cases()` with its spec-true expectation (`find:linter.no_orphan_property`
+ `graph.typing`).

## #80 — `spk verify --timeout-secs`: wall-clock bound on the runner's cargo run (specodelic-xx1)

The bc39f9d..main Ro5's advisory DoS finding, closed: `CargoRunner`
executed the scratch `cargo test` unbounded, so a pathological
(user-authored) predicate could hang `spk verify` forever — the same
flavor specodelic-suz fixed on the ingest path, cargo-test domain.

`run_bounded` (the `run_tlc` try_wait/kill poll pattern) wraps the
whole cargo run (build + test) under a wall-clock bound:
`--timeout-secs <N>` (default 600, `0` = unbounded). A run exceeding
the bound is killed, reaped, and its partial output captured — libtest's
"has been running for over N seconds" lines name the hanging block — and
reported as a labeled failure, never a silent hang.

The label is deliberately its own verdict stage: the new
`PropsGateState::TimedOut { secs, detail }` maps to status
`properties_timed_out` with a raise-the-bound-or-pass-0 hint, distinct
from `properties_failed` (spec: new `bounded_wall_clock` invariant +
`hang_reported_as_labeled_timeout` unit property in `verify.md`) —
a timeout is no verdict on the property, so a legitimately long suite
is re-run with a raised bound instead of misread as a failing predicate.

TDD: 5 new unit tests (bounded completion, unbounded escape hatch,
hang kill → labeled `TimedOut` with partial output, kill reaps the
child, gate/verdict mapping distinct from `Failed`).

## #79 — Reference Typing `guard` row reconciled with the shipped State-citation typing; specodelic.md Revision 12 (specodelic-tik)

The independent reference oracle's KNOWN GAP, closed: the cxq
reconciliation shipped guard→State typing in `graph.rs` and the oracle,
but the Reference Typing table still said `Constraint, kind ==
'invariant' only`, and `graph.md`'s `wrongly_typed_edge_rejected` note
contradicted graph.md's own Model extract (whose `extract` transition
guards on `[[specodelic.parsed]]`, a State).

`specodelic.md` gains **Revision 12**: the `guard` row now admits an
invariant Constraint **or a State** (the "has reached state X" pattern —
graph.md's `extract`, refactor.md's `analyze`, orchestrate.md's
`start_lint`); a State citation records progress, it gates nothing;
`advisory` still can never gate, by typing. A deriving Property
(`state_guard_citation_accepted`, unit, under `ref_kind_compatible`)
pins the acceptance beside the existing rejection rows. The stale
surface is reconciled in the same stroke: `graph.md`'s fixture now
picks a still-rejected target (guard → Property), `USAGE.md` §2.6's
typing remark, the embedded guide (`REFERENCE_TYPING` const + three
prose spots + `FORMAT_REVISION` → Revision 12, pinned by the drift
guard), the `spk new` scaffold's guard comment, and
`guard_required`'s RULE_TABLE semantics (presence-only; target typing
is `ref_kind_compatible`'s beat). The oracle's KNOWN GAP comment now
cites Revision 12. Corpus artifacts regenerated (21 compiled, 0
failed).

## #78 — verify scratch dir retention policy + `SPECODELIC_VERIFY_SCRATCH` override (specodelic-5m2)

Found during specodelic-oet's `just ci` run: the properties gate's
shared scratch base (`<tmp>/specodelic-verify`) accumulated 21GB of
cargo artifacts and exhausted the /tmp quota (`Os error 122`). The
per-invocation crate dirs were already removed post-run; the growth is
the shared `target/` compile cache, which never shrank.

`CargoRunner::run_blocks` now prunes the scratch base before staging:
orphaned crate dirs (from killed runs — normal lifetime is seconds)
are removed after 24 hours, identified by the `<pid>-<nanos>-<blocks>`
name shape (other entries are never touched — the base may point at a
user-chosen location); the shared `target/` dir is never age-pruned
(it IS the compile cache) but is dropped whole when it exceeds 4 GiB,
trading one cold proptest rebuild for quota headroom. The base path is
overridable via `SPECODELIC_VERIFY_SCRATCH`. Pruning is best-effort —
it never fails a verify. Documented in docs/src/commands.md.

## #77 — graph extraction honors the closed union of typed reference columns (specodelic-mlg)

Found by the conformance matrix's `total_refs_dangling` gap case, now
flipped ungated. `graph::build` walked ALL links and recorded an edge
whenever `typing_violation` fell through — which it does for untyped
columns — so a `[[ref]]` inside an `expr`, `predicate`, or frontmatter
`statement` cell became an edge and could dangle in the graph report,
violating `total_extraction`'s closed union. The links loop now skips
every column outside the typed set {traces_to, derives_from, guard,
supersedes, emits, satisfies, observes}: no edge, no graph dangling, no
typing — resolution of non-reference links is `total_refs`' beat in the
linter, which already scopes over every link in the file. `from`/`to`
stay out of the links loop by design: the transitions walk owns those
fields (one edge per well-formed cell), so nothing double-records.

Authority note: the filter uses the Reference Typing table's field set,
not graph.md's stale parenthetical — `satisfies` joined the typed fields
in Revision 7 and `errors.md`'s `contract_satisfied_from_consumer` pins
satisfies edges as extracted. graph.md's `total_extraction` enumeration
is reconciled to restate the table (and now says so). Corpus effect:
33 edges removed (expr/predicate/frontmatter kinds), dangling stays 0,
violations 0, supersedes cycles 0. Gates: just ci, lint-specs 21/0,
openspec strict, sync-sections; conformance matrix + oracle green.

## #76 — corpus reconciled with its own Reference Typing table (specodelic-cxq)

The mp1 decisions of record (#75) applied to the corpus itself. All 38
typing-forbidden edges `spk graph specs` reported are gone — the corpus
now passes the complete typed check with zero violations, and the dogfood
graph gate is meaningful (exits 0):

- **31 `constraints.traces_to` → Constraint retargets to own intent.**
  Every linter checker file's invariant rows previously traced to the
  `specodelic.md` constraint rows they re-own
  (`[[specodelic.guard_required]]` et al.) — a Constraint→Constraint
  target the table forbids. Each now traces to its own file's intent;
  the cross-file "same claim, two altitudes" relationship moves into
  prose (each file's Notes record the reconciliation).
- **Guard citations bridged to legal targets.** The three file-level
  guard citations (`graph.extract`, `orchestrate.start_lint` →
  `[[specodelic]]`; `refactor.analyze` → `[[graph]]`) now cite States:
  `[[specodelic.parsed]]` / `[[graph.queryable]]` — Revision 10 admits
  State targets for guards, so "has reached state X" is typed where a
  state exists, prose elsewhere. `orchestrate.compile_gate_matches_coverage`'s
  expr dropped its stray `[[specodelic.coverage]]` citations (an expr
  citing its own target row — nothing to add).
- **Typing table: `derives_from` gains the same-kind law edge.** The
  checker-file `*_naturality` laws derive from
  `specodelic.rename_naturality` (a Property) — the same-kind
  law-restates-law edge is now typed (`derives_from`: Property →
  Constraint, or the same Property when the deriving row is itself a
  law), kept well-formed by `acyclic_traces`, which already includes
  `derives_from`. A Revision-11-in-`specodelic.md` fact under
  `append_only_variants`.
- **Three laws completed to `law_requires_cases`.**
  `linter.coverage.coverage_naturality` gained its identity case;
  `linter.external_completeness.mapping_naturality` gained identity;
  `linter.referential_integrity.rename_naturality` gained its naturality
  case (it had identity+associativity only).
- **Six row ids renamed off forbidden tokens** (`no_universal_in_id`
  applied to every row kind): `scan_all` → `scan_effects`,
  `warning_never_fails` → `warning_never_gates`,
  `parser_ast_never_reads_prose` → `parser_ast_prose_free`,
  `external_completeness_never_gates` →
  `external_completeness_never_gating`, `fan_in_never_rewalked` →
  `fan_in_graph_sourced`, `law_cases_all_run` → `law_cases_unexecuted`.
  `linter-ears_syntax.md`'s `no_universal_in_id` expr now scopes the
  rule to every row kind explicitly (decision of record in that file).
- **`no_prose_field_parsed` rewritten as a runnable unit property**
  (kinds.md Revision 6's landing step): `parser_ast_prose_free` is now
  the planted-prose differential parse — no `audit` framing, no
  generator-preclusion claim remains.
- **`linter-coverage.md` added to specodelic.md's Checker Ownership
  table** — the corpus's own dogfood had left the last checker unlisted.

Also reconciled: the openspec `model-check` capability spec's two
Constraint→Constraint `traces_to` rows (`backend_identified`,
`provenance`) retargeted to `[[spec]]`. Graph tool tests pin the widened
typing (law→law allowed, guard→State allowed, invariant-only and
unit-Property halves still enforced); `graph_corpus_is_fully_resolved`
now asserts zero violations and exit 0.

## #75 — mp1's final four rows decided: guard typing, reachability, EXCL-001, merge approval (of record)

All ten HITL rows on `specodelic-mp1` are now decided — the last four via
a grill session (one question per turn, each user-approved and advised by
a typed Jev evaluation, `jev-1.13.0`):

- **Row 7 — guard typing = hybrid** (jev 0.95, conf 0.93): a transition
  guard may be prose, but when its gating condition corresponds to a
  declared constraint it must cite that typed invariant Constraint. The
  bridge pattern is precedented in the corpus (hooks capability spec,
  specodelic-b15). Landed in `specodelic.md` Revision 10's Reference
  Typing `guard` row; the target typing is unchanged.
- **Row 8 — reachability = tiered own-file intent** (jev 0.95, conf
  0.92): `single_root_reachable` reworded in Revision 10 — a row reaches
  its own file's intent through own-file primary linkage
  (`traces_to`/`derives_from` chains within the file); cross-file typed
  edges (`guard`/`satisfies`/`observes`) are outbound leaves, never
  reachability paths. Enforcement is tiered: cross-file-only rows are
  advisory-flagged first; hard enforcement flips after corpus
  reconciliation (`specodelic-cxq`) anchors the ~150 failures — cxq's
  first step is the strict-reachability dry-run. The strict reading of
  the old expr would fail ~150/375 corpus rows; the some-intent reading
  the shipped checker implemented would silently accept a cross-feature
  reference filed under the wrong id.
- **Row 5 — EXCL-001 dissolved** (jev 1.00, conf 1.00): no `audit`
  property kind; `kind ∈ {unit, law}` stays closed (`kinds.md` Revision
  6). The single motivating claim (`no_prose_field_parsed`) is
  mechanically testable by a planted-prose differential parse and will
  be rewritten as a runnable `unit` property with cxq;
  `linter-schema_shape.md`'s `Needs Human Review` flag is retired.
- **Row 2 — merge approval semantics** (jev 1.00, conf 1.00): "a human
  has explicitly approved" = an explicit, attributable, recorded
  operator decision (deliberate approve step + decision-trail entry);
  mechanism-agnostic floor, no CI gate or role assumed —
  `merge.md` decision-of-record Notes paragraph, constraint text
  unchanged (same treatment as rename.md row 1).

Gates: lint-specs 21/0, graph 0 dangling, openspec strict 12/12.
`specodelic-mp1` closed — **`specodelic-cxq` (corpus reconciliation) is
the unblocked P2.**

## #74 — `linter.failure_shape` checker ships: the error contract's tier-2 half (specodelic-ct5)

The three `specs/linter-failure_shape.md` rules are now enforced by
`spk lint` (previously spec-only by design D6 — normative-for-the-future,
never claims about today's linter):

- **`terminal_states_emit`** — every failure terminal (inbound
  transitions, no outbound transitions, fail-named STATE segment) must
  emit exactly one file-owned effect Constraint; a mute terminal, a
  multi-label terminal, a cross-file or non-effect emit is a finding.
- **`error_labels_unique`** — no two emitted error Constraints in a file
  share a variant head (the label's last `.` segment); cross-file
  collisions are structurally impossible by `errors.md`
  `error_expr_shape`, so the check is per-file by construction.
- **`guard_negation_total`** — every failure transition cites exactly the
  union of its success siblings' citation sets (the negated disjunction),
  or is on the recorded carve-out list — exactly `orchestrate.md`'s four
  stage-fails (`lint_fail`, `compile_fail`, `model_check_fail`,
  `verify_fail`), checked membership, never an assumption; a
  zero-citation failure guard off the list is a finding.

All three are graph-decidable per file — the same derivation the
`failure_terminals_emit_labeled_errors` fixture pins at corpus altitude
(citation sets from `transitions.guard` edges, labels from
`states.emits` edges), now walking each `Spec` in the lint pass. Nothing
reads prose (the D2a decision: classes ⟺ distinct citation sets, never a
prose judgment). v1 scope stays failure terminals only; `timed_out` /
`exploration_only` remain the stated non-goal.

DOGFOOD CATCH: the new gate flagged the archived dual-format capability
spec `openspec/specs/compile/spec.md` — its `failed` state predated the
error contract and emitted nothing (exactly the pre-contract shape the
tier-2 rules reject). Amended in place (self-contained dual format):
effect Constraint `compilation_failure`, the `failed` state's emits edge,
and a label-asserted falsifying property.

Gates: just ci + lint-specs 21/0 + lint openspec 20/0 + graph + openspec
strict 12/12; corpus findings stay zero (clean_repo_passes). 228 lib +
50 integration tests.

## #73 — `spk refactor` ships: the tidy-first split advisor (specodelic-3l7)

The last pipeline-adjacent stub is gone: `spk refactor [paths]
[--high-fan-in N] [--changeset id1,id2]` implements `specs/refactor.md` —
a non-gating advisory (exit 0 either way) that flags split candidates by
exactly the specced finding shape `{node_id, dependent_count,
unrelated_namespace_count, suggested_split}`. Every count is a `graph`
query (the advisor consumes `GraphReport`'s derived edges, never walks
markdown — `fan_in_read_from_graph`). Unrelated-namespace keying follows
row 3's decision of record (NCA == root, first-segment divergence);
`--high-fan-in` implements `threshold_is_per_repo_setting` (default 3);
`--changeset` enables `narrow_diff_heuristic` — a strict-subset edit
depending on none of the node's other owned rows flags regardless of
fan-in (only Property→Constraint `derives_from` edges carry such
dependencies legally, a constraint the typing table imposes on fixtures
too). Human rendering + docs/commands.md section added; README's stale
stub list fixed (nothing specced remains unimplemented). Dogfood: the
corpus itself flags `specodelic` as the top split candidate (12
unrelated dependents) — the exact pathology STATUS.md §2 names. CHANGELOG
numbering: this is the implementation entry for the ticket whose row-3
decision landed as #72.

## #72 — mp1 row 3 decided: refactor's unrelated fan-in keys off the id namespace (of record)

`specs/refactor.md`'s open question — does "unrelated fan-in" key off the
id namespace or physical directory placement — decided of record:
**id namespace**, test as written ("shares no namespace segment below
root"). Directory placement rejected: ids are the format's primary
identity (`id_matches_file`, typed foreign keys) and every other checker
reads relatedness from ids, while directory layout is outside the
format's ownership. Flat-namespace noise contained by the advisory's
non-gating typing plus the already-per-repo threshold; a configurable
keying mode stays a possible future widening. No constraint text
changes — decision recorded in the file's Notes. Unblocks
specodelic-3l7 (refactor tidy-first split advisor). Advised by a typed
Jev (`jev-1.13.0`) evaluation — choice 0.70, conf 0.54, impact medium
(0.85) — user-approved.

## #71 — mp1 row 1 decided: batch rename is one atomic transaction (of record)

`specs/rename.md`'s open question — is a whole-namespace batch rename
one transaction or `n` independent ones — decided of record: **one
atomic transaction**, all-or-nothing, the same
precondition→apply→reverify shape as the single rename; a partial batch
is exactly the dangling-`ref_resolves` failure `atomic_operation` exists
to prevent, and git's whole-commit atomicity makes per-file partials a
policy a tool shouldn't silently make. `atomic_operation` means the same
thing at batch scope. Batch-rename implementation is a separate
follow-up; no constraint text changes — decision recorded in the file's
Notes. Advised by a typed Jev (`jev-1.13.0`) evaluation — choice 0.83,
conf 0.74, impact medium — user-approved.

## #67 — `spk migrate`: wrap an openspec delta into the dual-format skeleton (specodelic-c32, gh#6 item 1)

The migration recipe (frontmatter + derive tables + byte-identical
`## Requirements` mirror) was fully manual — adopters converting 25 deltas
by hand named the mirror's sed one-liners as the top onboarding cost. New
`spk migrate <file> [--dry-run]` wraps a delta in place: generated
frontmatter (`id: spec`, EARS scaffold statement), wired scaffold layers
(`scaffold_constraint` / one-state model / `scaffold_property` — wired so
the file lints clean AS WRITTEN; an unwired skeleton would fail
`model_sections_paired` and orphan-island checks on first run), and a
byte-exact `## Requirements` mirror. Merge semantics: existing frontmatter
and layers pass through verbatim, only missing pieces are inserted; a file
already carrying the mirror is refused (the mirror is the migration
marker — re-running can never duplicate it). Files not named `spec.md`
get the naming-law warning, not a silently wrong id. Anti-goal held:
no prose is ever interpreted into rows — every inserted row is a marked
placeholder the author replaces.

## #71 — CI/supply-chain hardening: matrix, MSRV, cargo-deny, doc-example lint, honest dogfood baseline (specodelic-oet)

Six hardening slices, each gated:

- **serde_yaml → serde_yaml_ng**: dtolnay archived serde_yaml in
  2024-03; the frontmatter parser moves to the maintained fork (same
  API, zero behavior change — 395 tests green, corpus artifacts
  untouched).
- **MSRV**: `rust-version = "1.88"` declared and pinned in CI (edition
  2024's floor is 1.85, but the dependency tree — icu 2.3 via miette —
  requires 1.88; the declared floor matches the tree's real floor).
  A pinned test job ensures a dependency bump cannot silently raise it
  further without the declaration following.
- **Multi-OS**: macOS + Windows legs run clippy + tests alongside the
  ubuntu full-pipeline leg — the crate is a CLI users install
  cross-platform. First Windows run surfaced two latent portability
  bugs: the quick-start test's fence search was CRLF-sensitive, and
  sibling_blockers spawned WSL's bash stub. Both fixed; guard-siblings'
  hooksPath check now keys on sibling-tool CLAIMS (unset = compliant
  with a note), which is the honest reading of the AGENTS.md blocker
  and works on fresh CI clones.
- **cargo-deny**: advisories + licenses gate in CI (deny.toml,
  permissive allow-list incl. MPL-2.0 for stateright).
- **Doc-example lint** (`just lint-doc-examples`): fenced markdown
  blocks whose first non-empty line is `---` are runnable format
  artifacts — they are extracted, named per the naming law, and
  linted with the freshly built spk. First run caught real drift:
  USAGE.md's api.rate_limit worked example predated the four-layer law
  (Constraints only) — completed with Model + Properties. Regression
  suite: scripts/test_check_doc_examples.py (stdlib unittest).
- **Honest dogfood baseline** (`just lint-baseline`): corpus findings
  vs specs/.lint-baseline, shrink-only — a new finding fails, and a
  baseline entry with no live finding also fails (accepted debt must be
  retired). Baseline is empty today: the corpus lints fully clean.
- **CI drift guards** (tests/ci_wiring.rs): matrix coverage, MSRV pin,
  cargo-deny wiring, and the doc-example gate are pinned by tests, so
  the hardening cannot be silently edited away.
- **CI repair en route**: main's last three runs failed with
  `openspec: command not found` — ci.yml now installs
  @fission-ai/openspec@0.19.0 (pinned to the vendored version). The
  spike-dual-format archive fixtures were completed to full dual
  format (they carried `## ADDED Requirements` without the capability
  half — the first thing `spk lint openspec` failed on in CI).

## #70 — orchestrate.md: draft→parsed decision of record (mp1 row 4)

The `Needs Human Review` open question is resolved: the orchestrator
stays **scoped to `parsed → verified`** — parsing remains outside its
Model. The orchestrator runs the parse step and reports its outcome
(parse errors gate lint via `start_lint`'s precondition), but the
draft→parsed transition's guards stay owned by the frontmatter checker
and the parser; no Model change. Decision recorded in
`specs/orchestrate.md` Notes; unblocks closing specodelic-8kk.

## #69 — Ro5 over specodelic-8kk: orchestrate hardening

Rule-of-5 review of the orchestrate changeset (converged stage 4, all
findings verified/fixed). EDGE-001 (HIGH, TypeSafe-verified @ 0.97): a
checklist-only corpus — zero spec files, one declared manifest —
orchestrated to a vacuous `succeeded` (every stage passed over an empty
file set, the specodelic-6pi false-green class); orchestrate now
requires at least one spec file and exits 2 with a checklist-specific
hint. CORR-001: the parse-stage gate keyed on substring-matching note
prose (`contains("parse error")`) — a file whose path happens to
contain the phrase would have flipped the gate; `parse_batch` now
returns structured parse errors and orchestrate gates on
`ParseInput.parse_errors` (unit test pins that note prose never flips
the gate). CLAR-001: transitive skip reasons no longer claim a
dependency "reported failed" when it was itself skipped — the reason
carries the dependency's actual status. DRAFT-001 noted (not fixed):
cmd_orchestrate duplicates cmd_model_check's backend-validation block —
tidy candidate.

## #68 — spk orchestrate: the four-stage pipeline driver (specodelic-8kk)

`spk orchestrate` replaces its stub: the orchestrator runs lint's six
Checker Ownership checkers in dependency order (frontmatter first gate;
referential_integrity → graph_shape → model_shape; ears_syntax and
schema_shape as independent parallel branches), then compile,
model_check, and verify in sequence — halting at the first stage that
fails, with every downstream stage reported as `skipped` (never failed)
and its reason named. A failed checker's dependents are never invoked:
lint.rs now exposes per-checker family invocations (`frontmatter_findings`
et al.) so the orchestrator runs exactly the checker whose turn it is,
not one flat pass filtered for the report. Independent branches report
regardless; external_completeness runs when a checklist is declared and
never gates; the compile stage's gate is exactly the coverage checker's
verdict. The model_check stage passes only on `no_counterexample` — the
native backend's `exploration_only` honestly fails the stage, so a
native-only run halts there (TLC opt-in via `--backend tlc --tlc-jar`).
Reports are byte-stable per `deterministic_rerun`. `artifact_stem`/
`write_artifacts` moved into compile.rs (shared with the driver);
`TlcPaths` gained `Clone`. Schema_shape's entry names its basis
honestly: its rules are structural (parser + closed kind sets); the
unimplemented table-walking residue is a tracked coverage gap.

## #67 — docs: orchestrate shipped, refactor remains the only stub

`docs/src/commands.md` documents `spk orchestrate`'s stage/gate
contract; the Pipeline-stubs section now lists only `spk refactor`.

## #66 — mapping_naturality enforced: rename reaches into checklist mapped_ids (specodelic-4d5)

The follow-up specodelic-b15 deliberately deferred: a checklist sits
outside `𝒦`, so `rename.md`'s `old_id_fully_replaced` never reached its
`mapped_ids` cells — renaming a mapped id either dangled the cell
(bare spelling, caught later by `covered_maps_resolve`) or — worse —
failed the verify gate outright (`[[…]]`-wrapped cells inside table
rows were wiki-link-rewritten into a file `parse_str` then rejected as
`post-rename parse failed`). Both are fixed. `spk rename` now routes
`*.checklist.md` files through a checklist-side rewriter
(`checklist::rewrite_text`): only mapping data rows' `mapped_ids`
cells change (bare and `[[…]]` spellings alike, children following
their parent, padding and terminators byte-exact; the items list,
item/rationale cells, header, separator, and prose pass through).
The verify gate now re-runs the external-completeness checker
(`covered_maps_resolve` + manifest well-formedness) over the
post-rename corpus — a missed cell or a rename that would dangle one
is a labeled `VerifyFailed` before any byte is written
(`mapped(rename(I)) == rename(mapped(I))`). Aspirational-note updates:
`specs/linter-external_completeness.md`, `specs/STATUS.md`;
`docs/src/commands.md` rename section. RED→GREEN with regression
pinning at both the cell and file level plus a gate-rejection test.

## #63 — observability contracts: observes typing, advisory check, derived boundaries (specodelic-7l3)

The format can declare outputs (`emits`, Revision 6) and contracts
(`extension_point`/`satisfies`, Revision 7) but could not say what must
be *observable*: a spec could declare effects nothing watches and lint
clean. Landed, per `add-observability-contracts` (Ro5-reviewed twice —
originating design pass + grounding pass that caught the Revision 8
numbering collision, the corpus's live unobserved effect, and the
missing advisory-severity machinery):

- **Format** — `specodelic.md` Revision 9: Reference Typing gains
  `observes` (Constraint, any file → Constraint, `kind == effect` only),
  mirroring `satisfies`; joins no acyclic edge set (mutual cross-file
  observation is well-formed). `kinds.md` Revision 5 reconciles
  `constraint_row_shape` with optional typed reference columns (the
  table governs — resolves HITL `specodelic-mp1` row 9). `graph.md`:
  `observes` in `total_extraction`, `external_boundary_derived`.
  `linter-graph_shape.md`: the acyclic union is unchanged, stated.
  New `linter-observability.md`: universe = invocation's file set,
  self-observation does not count, dangling observes is
  `referential_integrity`'s beat, no waivers in v1.
- **Tool** — `graph`: the `observes` typing arm (effect-only) and
  `external_boundaries` (a file is one iff it hosts ≥1
  `extension_point` Constraint — derived, never authored). `lint`: rule
  `linter.observability` warns — on the success envelope's warnings
  channel, exit 0, never an Issue — for every unobserved effect in the
  invocation's file set. `Report` gains `warnings`; human output renders
  advisories.
- **Dogfood** — the corpus's own live unobserved effect
  (`refactor.advisory_finding_emitted`) is the first warning evidence
  (recorded in the change's dogfood notes; gating decision deferred to a
  follow-up change). USAGE §2.9 pattern + decision-table row; STATUS row.

## #62 — lint totality: model_shape remainder + graph_shape enforced (specodelic-b15)

Five rules that four checker files specced but no checker enforced are now
lint rules (append-only additions to the rule table, TDD red-first):

- `every_state_used` — a declared state no transition enters or leaves is
  machinery the model can never reach.
- `every_transition_valid` — `from`/`to` must name declared states; a
  dangling `to` makes the model-checker simulate a different graph than
  the author wrote.
- `no_self_ref` — a row tracing to itself via `traces_to`/`derives_from`
  has no owning purpose.
- `acyclic` — the directed graph formed by traces_to ∪ derives_from ∪
  guard-as-edge rejects cycles (rotation-normalized, reported once per
  cycle; self-loops are `no_self_ref`'s beat, never double-reported).
- `single_root_reachable` — every constraint/property/state/transition row
  connects to some intent row (undirected connectivity over traces_to,
  derives_from, guard, from/to, emits — an outbound-only reading would
  flag every non-emitting state, which no corpus satisfies). Which
  intents count (any vs the file's own) is specodelic-mp1 row 8's open
  question; this implements the checker spec text as written.

Decision of record (ticket comment): **reference typing stays
graph-layer-owned** (`spk graph`'s labeled violations + non-zero exit) —
duplicating `ref_kind_compatible` in lint would double-report the same
edges and pre-decide mp1 rows 7–9. The graph-shape rules land in lint
because `linter-graph_shape.md` owns them and neither layer enforced
acyclic/self-ref/reachability until now.

Dogfood consequences, all resolved in this change:

- `specs/` corpus: 18 files, **zero findings** under the new rules.
- `spk new` scaffold: the placeholder model was an island (empty
  constraints table + prose guard) — the scaffold now ships a connected
  minimal loop (`c1` invariant tracing to the intent, `t1` guard citing
  it, `p1` deriving from it) that lints clean out of the box and teaches
  the connected-shape law by example.
- openspec tree bridges: `hooks` capability guards carry typed citations
  (prose kept, citation appended — the typing table's guard→invariant
  law); `model-check`'s `no_fabrication` gained `[[spec]]` beside its
  prose pointer (`specs/model_check.md` — a bare prose path produces no
  edge, so the constraint and its deriving property were an island).
  `lint openspec` and `lint specs` both report zero findings.
- `external_completeness` (the fourth checker file) remains
  unimplemented: the checklist manifest format is Needs-Human-Review in
  the checker spec's own Notes (a checklist file sits outside the
  four-layer shape; a sixth kind is explicitly deferred) — filed as
  specodelic-mp1 row 10.

## #61 — bare row refs resolve in id:spec files (Option A) — specodelic-15g, gh#2.1

Adopter report (gh#2 point 1): in a dual-format delta (`id: spec`), a
bare `[[c1]]` vanished — the metasyntactic skip ate the dotless target
silently: lint stayed green, coverage counted it, but the graph edge
never existed. Decision of record (ticket comment, user-approved):
**Option A — bare row refs resolve against the file's own rows**.

- One rule: an `id:spec` file is self-contained (#42), so a dotless
  target naming one of its own rows has exactly one possible meaning —
  the local row. It now resolves (canonical `spec.<row>`) in BOTH lint
  (`total_refs`) and `graph` — the two paths share the law instead of
  graph resolving while lint skips.
- Robustness preserved: the rule is fail-loud, not fail-open — a bare
  target naming NO own row still skips as metasyntactic (template
  placeholders), and typing violations on bare refs are now VISIBLE
  (a bare `traces_to [[c1]]` resolves and then violates Reference
  Typing — reported, never swallowed).
- Dotful spellings unchanged: `[[thing.one]]`-style local ids keep
  dangling with the gh#5 self-file hint (a bare dotful target is
  genuinely ambiguous with `file.row`).
- Coverage message fixed: it claimed "bare ids do not resolve" while
  its own check accepted the bare form — the claim is gone.
- `spk explain references` updated: either spelling works inside a
  delta; files outside always use the file-qualified form.
- Corpus churn: zero — the dogfood gates lint/graph the whole openspec
  tree every commit; no dual file used bare refs.

RED→GREEN: 4 tests (edge, visible typing violation, narrowness guard,
lint agreement). Gates: just ci + lint-specs 18/0 + graph 0 dangling +
openspec strict 9/9.

## #60 — corpus discovery: consumer repos' openspec/ tree is named where bare lint/graph/doctor dead-end (specodelic-ag5, gh#2.2)

Adopter report (gh#2 point 2): in an openspec-managed consumer repo, bare
`spk lint` said "no spec files found" even with 14 dual-format files
present; the working invocation `spk lint openspec` had to be found by
trial. Now (option (b)+(c) from the ticket — minimal, composes with the
exit-code contract):

- **lint/graph zero-files failures name the tree** — when cwd carries an
  `openspec/` directory, the exit-2 hint becomes "found an openspec/ tree
  — try: specodelic lint openspec" (graph likewise). The parse-error
  branch is untouched.
- **doctor gains a `corpus discovery` check** — reports `ok (specs/)` in
  self-hosting repos, the openspec tree with a spec-file count in
  consumer repos, or the discovery rule ("pass a directory containing
  *.md specs; hidden and build dirs are skipped") otherwise. The
  consumer next-step prefers `spk lint openspec` when the tree exists.
- Decision of record (ticket comment): option (a) — a configured corpus
  path (env var / managed-block line) — deliberately deferred; (b)+(c)
  fix the discoverability gap without new config surface. Revisit only
  if adopters report the hint is not enough.

RED→GREEN: 4 tests (lint hint, graph hint, doctor openspec naming,
doctor missing-corpus rule). Gates: just ci + lint-specs 18/0 + graph 0
dangling + openspec strict 9/9.

## #59 — Ro5 review of bc39f9d..main: merge modify/delete conflicts, deletion blast radii, CRLF-safe rename

Review sweep over everything shipped since the suz Ro5 (verify, output
contract, rename, TLC backend, graph edge contract, merge, adopter
feedback r2). Findings + fixes:

- **merge: modify/delete conflicts were a false clean `merged`** — a
  file edited on one branch and deleted on the other kept the edited
  copy in the union silently (or B's version won silently), and the
  relint validated a tree the real merge would never produce. Now the
  conflict is flagged (`textual_conflict`, both directions) and the
  edited version stays only so the relint sees the tree the merge
  would actually produce. Confirmed empirically before the fix: git
  would conflict; `spk merge` said `merged`.
- **merge: deletions count as touches** — a base id missing from a
  branch's tip now enters `touched_ids`, so its dependents land in the
  blast radius (`semantic_conflict_iff_blast_radius_intersects` holds
  for deletions too).
- **merge: honest messages** — the collision finding no longer claims
  "different definitions" when both branches made identical edits (the
  verdict still fails per `no_new_id_collision`'s strict reading);
  rename-replay findings say "removed (a rename or a deletion)" instead
  of asserting a rename, and distinguish replay vs stale-reference
  cleanup.
- **merge: `--base` omission is explained** — without an ancestor every
  shared id reads as independently minted; the envelope now carries a
  warning saying exactly that.
- **rename: CRLF files keep their terminators** — `rewrite_text` used
  `lines()` + `\n` rejoin, silently normalizing every `\r\n` to `\n`
  (byte churn violating `prose_untouched_by_rename`). Terminators are
  now preserved per line.
- **model-check: TLC scratch dir is cleaned on the module-write error
  path** (was: partial residue in the temp dir).
- Removed a stray `demo-thing.md` committed at the repo root (a manual
  `spk new` leftover; tests stage their own copies in temp dirs).
- Advisory (unfixed): `spk verify`'s CargoRunner executes `cargo test`
  unbounded — a hanging predicate hangs the verify (cargo-test domain,
  not a tool bug; revisit if a wall-clock flag is ever wanted).
- Pre-existing cosmetic: the CHANGELOG has a duplicate `#56` heading
  (TLC ETXTBSY entry vs the ar2 entry) — left alone to avoid
  renumbering cited entries.

Gates: just ci + lint-specs 18/0 + graph 0 dangling + openspec strict
9/9.

## #58 — adopter feedback round 2: drift is lint-enforced, self-file ref hint, help/DX papercuts (gh#4/#5/#6)

Second adopter migration (poco, 25 deltas) surfaced three issues; all
claims verified on main before filing. Shipped:

- **`linter.requirement_drift` (gh#4)** — the dual-format mirror is now
  enforced by `spk lint` itself: a file carrying both `## ADDED
  Requirements` and `## Requirements` must hold identical requirement
  text (per-line trailing space and blank lines ignored — same
  normalization as `scripts/check_section_sync.py`, which remains the
  repo-CI form). Previously drift was silently clean; the explain
  topic's claim outran the checker.
- **total_refs self-file hint (gh#5)** — a dangling ref whose target
  names a row defined in the same file now appends: "hint: row `X` is
  defined in this file; refs must be file-qualified:
  `[[<file-id>.X]]`" — no more corpus-hunting for a local fix.
- **explain dual-format** documents `traces_to: [[spec]]` as the
  intended self-intent edge (gh#5) and credits `spk lint` (not the repo
  script) with drift enforcement (gh#4).
- **lint failure hint** no longer cites the nonexistent `Notes` field —
  it points at `spk explain lint-rules` (gh#4).
- **`spk --help`** no longer leaks the genesis CliFormat doc comment
  above Usage (fixed with an explicit `long_about`; upstream note for
  genesis pending) and the `orchestrate` stub is labeled "not yet
  implemented" in the subcommand list (gh#6).
- **explain format** documents escaped pipes (`\|`) in table cells
  (gh#6).

Already-fixed-on-main credits: gh#5's scaffold complaint was addressed
in #56 (df1671e). Filed follow-ups: `spk migrate` (gh#6 item 1,
highest-value), corpus discovery = specodelic-ag5.

## #57 — Ro5 correction: the file-qualification doc must match checker behavior (specodelic-ar2 follow-up)

Rule-of-5 over #56 caught a doc/behavior mismatch in the very paragraph
that change added: `spk explain references` claimed bare `[[C-foo]]`
"fails", but empirically a bare dotless `[[row-id]]` is skipped as
metasyntactic — lint stays green, the coverage rule's bare-id arm even
counts it as covering (`src/lint.rs` `derived.contains(*cid)`), and the
graph silently drops the edge (0 findings, 0 dangling, 0 edges on the
fixture). Only the bare-TEXT form fails loudly (no reference at all →
`no_orphan_property`).

Fixed: the references topic now states both behaviors accurately (bare
text → orphan; bare bracketed → metasyntactic silent-vanish, "nothing
fails but nothing resolves"); the `spk new` guard placeholder cell no
longer demonstrates the bare `[[id]]` form and the `emits` placeholder
is file-qualified too; STATUS.md's design-decisions section drops the
bare `[[id]]` shorthand. The checker-side gap (coverage accepting the
bare form; metasyntactic skip eating refs) is the open subject of
specodelic-15g.

## #56 — docs/lint DX: the file-qualified ref law is now taught where authors trip over it (specodelic-ar2, gh#3)

An adopter authoring a dual-format delta wrote `derives_from: C-foo`
(bare id), then bare text `spec.C-foo` — both invisible to the parser,
which collects only `[[wiki-links]]` from structured cells; every
property came back orphaned and coverage failed with no hint at the
actual fix. Now the law is documented where the author already is:

- `spk explain references` gains the file-qualification paragraph: refs
  are `[[<file-id>.<row-id>]]`, bare ids and bare text do not resolve,
  and a dual-format delta cites its own rows with the `spec.` prefix
  (`[[spec.<constraint-id>]]`); `spk explain format` no longer teaches
  the bare `by [[id]]` form in the layer walkthrough.
- `spk new` scaffold: the States/Transitions/Properties guidance
  comments render a concrete self-file example from the new spec's own
  id (e.g. `[[demo.thing.c1]]`) and state the bare-id warning. Examples
  live only in comments — a dotted ref in an actual cell would dangle
  in a single-file corpus (metasyntactic skip is dotless-only).
- Lint findings teach the fix: `no_orphan_property` now hints
  "derives_from takes a file-qualified wiki-link like
  `[[<file-id>.<constraint-id>]]`" using the finding's own file id, and
  the `coverage` message shows the required target as the literal cell
  text to type (`write `[[t.c]]` in the property's derives_from cell`).
- `specs/STATUS.md` §1 primer Model row matches the law.

Gates: just ci (140 lib + integration) + lint-specs 18/0 + graph 0
dangling + openspec strict 8/8.

## #55 — graph: the full edge contract — state edges, typing violations, supersedes cycles (specodelic-7pi)

`spk graph` now extracts every typed reference field instead of only the
wiki-link columns. A Transition's `from`/`to` cells are typed reference
fields (→ State, same file) and yield `transitions.from`/`transitions.to`
edges — the corpus graph grew from 407 to 569 edges; an unknown state in
`from`/`to` dangles rather than vanishing. The Reference Typing table
(`specs/specodelic.md`) is now enforced at graph-build time: a typed
reference column whose resolved target kind is forbidden (31 corpus
`traces_to`→Constraint rows, 4 `derives_from`, 3 `guard` citing non-
invariant Constraints) surfaces as a labeled **violation** in the report
and is never recorded as an edge (`specs/graph.md`'s
`edge_kind_matches_typing`) — previously those edges were silently
recorded. The report carries `violations` and `supersedes_cycles`; the
`supersedes` edge set is cycle-checked (`supersedes_dag`,
`specs/linter-graph_shape.md`), rotation-normalized so a cycle reports
once. The command exits 1 when anything is dangling, violated, or cyclic.
The corpus's own typing debts are tracked for reconciliation in
specodelic-cxq — until then `just graph-specs` honestly reports them.
The `spk init` managed block no longer advertises blast-radius (not yet
implemented; re-added when it ships).

## #56 — model-check: TLC shim spawn retries the `ETXTBSY` race

The TLC backend's process spawns (version probe + run) retry a few times
with short backoff on `ETXTBSY` (`Text file busy`, os error 26) —
spawning a just-written script shim raced its write-close under parallel
test load, flaking the TLC test harness ~2/15 on a clean tree with a
spurious `missing_checker`. Any other spawn error, or exhausted retries,
still surface immediately as the same labeled error — a genuinely
missing JVM binary is never masked.

## #54 — model-check: opt-in TLC backend — the JVM reference engine (specodelic-ug3)

`spk model-check --backend tlc --tlc-jar <tla2tools.jar>` runs the TLA+
TLC reference engine as a JVM subprocess over the compiled `<stem>.tla`
module, `-depth` as the stated bound. MUST held by construction: a
missing JVM binary (PATH or `SPK_TLC_JAVA` seam) or jar is a labeled
`missing_checker` error — never a `no_counterexample` result.

- Same run-report contract as the stateright default (`backend_identified`):
  the report carries `engine: "tlc"` + the version parsed from TLC's
  `-version` probe, so two backends' reports on the same compiled model
  and bound are attributable and comparable.
- Same honesty as the native backend: zero corpus invariants are
  executable (prose — Decision 3, Option A), so a completed TLC run is
  `exploration_only`, never `no_counterexample`; a depth-cut behavior
  (`The behavior up to this point is error-free`) and a wall-clock
  budget the backend enforces itself (poll + kill) report `timed_out`.
- Fail-closed classification: unrecognized output on a zero exit, a
  nonzero exit, and a violated engine invariant (TypeOK — not a
  Constraints-table id) are labeled `tlc_error` /
  `tlc_invariant_violated` errors; the counterexample leg waits on
  specodelic-mp1's predicate-fragment decision.
- `--max-states` has no TLC equivalent — labeled `unsupported_bound`,
  never silently ignored. The module runs in a scratch dir (TLC drops
  `states/` beside its input — never beside the committed artifact).
- Classification pinned by unit + integration tests through a fake-JVM
  seam; real-TLC agreement remains external evidence (25/25 differential,
  conformance suite specodelic-vv8).

## #53 — merge: pre-merge id-collision + dangling-rename check (specodelic-7oq)

`spk merge --branch <incoming-tree> [--base <ancestor-tree>] [current-tree]`
per specs/merge.md — the check that runs after git's 3-way merge succeeds,
over the two branch tips' spec trees (never runs git, writes nothing).
- `no_new_id_collision`: ids defined on both tips are flagged unless the
  ancestor defines them identically (newly minted on both, or edited on
  both with divergent bodies = collision); inherited-and-unchanged ids are
  never falsely flagged.
- `blast_radii_recorded_pre_merge` + intersection → `needs_review`: per-
  branch touched ids (files changed vs ancestor) and blast radii computed
  from each branch's OWN `graph` artifact (graph_reused_not_rederived —
  fan-in/fan-out closure, no independent markdown walk).
- `rename_replayed_onto_foreign_edits` flagged (replay itself delegated to
  `spk rename`): an id renamed away on one branch while the other branch
  mints a structured `[[old]]` reference is named with a remediation hint.
- `post_merge_relint_required`: the union tree (A wins deletions; files A
  left untouched take B's version) must re-parse, re-lint clean, and show
  zero dangling, or the merge is `failed`.
- Findings kinds: `id_collision`, `rename_replay`,
  `blast_radius_intersection`, `textual_conflict`, `relint_failure`,
  `unparsable`. Verdicts: merged (exit 0) / needs_review (1) / failed
  (exit 1). The human-approval semantics of the `resolved` transition
  remain open in specodelic-mp1.

## #52 — rename: atomic id rename with link rewrite (specodelic-ams)

`spk rename <old_id> <new_id> [files|dirs]` per specs/rename.md. The
single-`(old_id, new_id)` rename lifecycle: validate → apply → verify.
- Owner lookup by QUALIFIED id (intents by frontmatter id; rows by
  `intent.id` + local cell id — links carry the qualified id, table
  cells the local one; this distinction is the whole lookup).
- Rewrites: `[[old_id]]` and child refs `[[old_id.x]]` anywhere (link
  syntax only — prose mentioning the old id in words is untouched,
  `prose_untouched_by_rename`); exact-id table cells and state bullets
  in the definition file only; the frontmatter `id:` line. A row rename
  must stay in its file's namespace (`intent.id.<new-local>`).
- Intent renames also rename the file per `-` ⇔ `.` (writes new file,
  removes old last — the definition is never momentarily missing).
- Atomicity by construction: the full write set is computed and
  verified in memory (re-parse + `linter.referential_integrity` zero
  dangling — the two checkers specs/rename.md's Notes select) before
  any write, so collisions (`new_id_available`), unknown ids, and
  verify-gate rejections leave the repo byte-identical. Identity
  rename (`a → a`) is a no-op success (rename_naturality identity).
- 4 integration tests (row rename + cross-file refs + prose untouched,
  intent rename + file move, collision rollback byte-identical, unknown
  id) + 2 unit tests; dogfooded on the real corpus (row + intent
  renames, 0 dangling / 0 lint issues after).
- Exit codes: 0 renamed, 1 labeled rejection, 2 no files ingested.

## #51 — Ro5 review of the hostile-input hardening set: hostile notes ride every verb's empty failure envelope (suz follow-up)

Rule-of-5 review of #46 (`f3e27f5` + `6f99b4a`): verdict READY WITH_NOTES,
converged at Stage 4. One MEDIUM fixed (CORR-001): the labeled
hostile-input/parse notes were attached to the `specs.is_empty()` failure
envelope only in lint and graph — compile, model-check, and verify still
dropped them into a bare "no spec files to …" failure, the exact
vanishing-diagnostic class #46 closed for lint. Now all five verbs carry
the notes on the empty path (new integration test pins it for the three
fixed verbs). LOW residuals documented, not fixed: the metadata→read
gap is a TOCTOU race (airtight fix = open with `O_NOFOLLOW | O_NONBLOCK`
+ fstat on the fd — not worth the platform surface for a local CLI), and
a duplicated doc-comment line above `MAX_INPUT_BYTES` was removed
(CLAR-001).

## #50 — output-contract residuals: exit codes for invalid invocations, human text everywhere (specodelic-7rr follow-up)

Pre-release sweep of the remaining 7rr-class warts:

- **Exit codes**: unknown `explain` topics and `new`-onto-an-existing-file
  now exit **2** (invalid invocation — uniform with clap argument errors
  and the no-files case) instead of 1; `1` stays reserved for findings /
  tool-level failures.
- **`--human` is Debug-free repo-wide**: `explain` (topic list renders as
  `id — title` lines), `new` (unquoted `created <path>`), `init` (one
  block-outcome line), and `hooks install/uninstall` (outcome + gate
  dry-run verdict) all render real text; every failure path now prints
  nothing to stdout — message, notes, and footer ride stderr. The old
  `emit` helper is gone; every verb routes through `emit_report`.

## #49 — docs drift: STATUS revision row, USAGE quick-start lint-clean (specodelic-vpx)

- `specs/STATUS.md`'s corpus table recorded `specodelic.md — Done —
  Revision 7` while the binary embeds Revision 8 (and the same file
  already referenced Revision 8 two sections later). Row corrected to
  Revision 8; the §2.6 citation of Revision 7 stays — it dates when
  `extension_point`/`satisfies` was introduced.
- The `USAGE.md` §1 quick-start example failed the tool's own lint: it
  never told the reader the filename (`order.cancel` must live in
  `order-cancel.md` — the `id_matches_file` law), and `refund_timely`
  had no deriving property (`linter.coverage`). The example now names
  the file up front and gains the `refund_within_term` unit property.
- Kept honest without the CI-hardening ticket's doc-example lint step:
  `usage_quick_start_example_is_lint_clean` extracts the §1 markdown
  block from `specs/USAGE.md` and lints it — the new user's first
  copy-paste can no longer rot silently.

## #48 — output contract: exit codes 0/1/2, real human text, ok:false errors (specodelic-7rr)

The output contract tightened on four fronts (the `ok:false` half rides
upstream — see below):

- **Exit codes** are pinned and documented (`--help` after-help,
  README, [docs/src/commands.md](../docs/src/commands.md)): `0` =
  success (lint with zero findings counts); `1` = the stage produced
  findings or a tool-level failure; `2` = invocation error — nothing
  was processed (path not found, no spec files matched, unreadable
  input). All five report verbs now exit 2 on an empty corpus — `graph`
  previously returned exit 0 with an empty report on a typoed path, a
  silent green. The JSON envelope's `envelope_kind` agrees (`"error"`).
- **`--human` renders real text**: new `src/human.rs` — one formatter
  per report verb (lint/graph/compile/model-check/verify/doctor), each
  rendering from the same report values the JSON envelope carries, so
  the human story and the JSON story cannot drift. The Rust `{:?}`
  Debug dump is suppressed (verbosity-threshold trick, the `explain`
  pattern); failure paths print nothing to stdout — message, notes, and
  footer go to stderr. Snapshot regression tests pin "no Debug markers"
  for every report verb.
- **Error envelopes carry `ok:false`**: root cause was upstream —
  genesis `Envelope::success` hardcoded `ok:true` even when
  `to_envelope` passed `kind = Error`. Fixed in genesis (genesis-r13,
  hint commands also now carry the bare runnable command — no more
  `→ Run: run:` doubling). The local regression test
  (`error_envelope_serializes_ok_false`) is `#[ignore]`d until the
  fixed genesis release lands in Cargo.toml; verified RED against
  0.8.1.

## #47 — verify: both gates, really executed (specodelic-1pv)

`spk verify <files>` — the `model_checked → verified` transition. The
guard `no_counterexample ∧ properties_pass` was named in
`specodelic.md` but never enforced; now both gates are evaluated and
conjoined, and every deviation is a labeled failure:

**Properties gate** (`*_props.rs`): staleness is detected by comparing
the artifact's block-metadata fingerprint (`// id:` / `// case:` /
`// generator:` / `// predicate:` comments, parsed back out) against
what the current spec regenerates — a hand-translated predicate body
keeps the fingerprint, a spec edit changes it, and a stale artifact is
never executed (`properties_pass_reflects_latest_run`). Execution is
real, not interpreted: the artifact is staged into a per-invocation
scratch cargo crate (unique test filename so parallel verifies sharing
the persistent `CARGO_TARGET_DIR` never collide on a test binary) and
run via `cargo test`; libtest output is parsed per block. An
un-translated `todo_predicate!` panics and reports honestly as
`properties_failed` — with proptest's shrunk minimal failing input
(`failure_reports_shrunk_counterexample`) — never as a skip. Zero
property rows pass vacuously without invoking the runner.

**Model gate** (`<stem>.check.json`): the report must parse, its
`artifact_sha256` must match the current `<stem>.tla` (a missing module
fails closed as stale — `rerun_on_model_change`), and its outcome must
be `no_counterexample`. The native backend only ever reports
`exploration_only`, which is explicitly not clean — `verified` is
reachable only through a backend that actually executes invariants.

**CLI**: `spk verify <files> --out-dir <dir>`; single-file
`.data.status` is `verified` exactly under the conjunction, else the
first blocking stage. Both gates are always evaluated; the payload
carries both so no stage is hidden.

Gates: just ci + lint-specs + openspec strict; 117 lib + 56 integration
tests (incl. the cargo-backed honest-failure path).

## #46 — hostile-input hardening: ingestion gate + codegen identifier sanitizer (specodelic-suz)

Two trust boundaries read or emitted from files that may not be what
they claim:

**Ingestion** (`parse_batch`, shared by lint/compile/verify/model-check):
`std::fs::read_to_string` with no file-type or size check — `spk lint` on
a FIFO named `*.md` blocks forever (exit 124 under timeout); `/dev/zero`
reads unbounded and OOM-kills (confirmed mechanism, tested under ulimit).
Now: metadata is checked before any read — only regular files under a
**2 MiB cap** (corpus files are ~10-50 KB) are ingested; everything else
is labeled, names the offending path, and is skipped. When nothing else
was linted, the labeled notes ride the failure envelope (they previously
vanished into the generic "no spec files found" — parse-error notes had
the same silent fate, now fixed).

**Codegen** (`sanitize_ident`): the row-id → Rust-identifier sanitizer
covers hostile ids — leading digits and unicode were already handled
(pinned); reserved words (`fn fn(…)` does not compile) are prefixed with
`_`, the empty id becomes `_`, and ids beyond a 64-char cap are truncated
with a stable **FNV-1a** hash suffix (std's `DefaultHasher` is
release-unstable; emitted artifacts are byte-stable) so same-prefix ids
never collide.

Gates: just ci + lint-specs 0 + graph 0 dangling + openspec strict 8/8;
corpus artifacts byte-identical (corpus ids are short — no fn-name churn).

## #45 — artifact consistency checks edges and Output, not just ids (specodelic-8nt)

`assert_artifact_consistent` compared only the compiled `.tla` module's
`StateValues` line and the disjunct-comment **id set** against the IR —
so a hand-edited or stale module whose code edges differed from its
comments (an edited to-state, a deleted code line with its comment left
behind) passed silently, and the run report attached the modified file's
SHA-256 as provenance. The checker explored the IR's transitions while
claiming to have checked the on-disk module.

Fixed in three comparisons, all against the IR extracted from the live
spec: (1) the **edge multiset** parsed from the `\/ vpc = "…" /\\ vpc' =
"…"` code disjuncts; (2) the **Output function** verbatim — `ModelIr`
gains `emits_values` (state → the value the emitter writes: the
effect-Constraint's `expr`, computed through the same `constraint_exprs`
map `model_to_tla` reads so IR and artifact cannot diverge); (3) the
existing states/id-set checks. Anything outside the emitted shape is
fail-closed `artifact_unreadable` (a `\/` line that doesn't parse, a
malformed Output entry).

Refactor: extraction split into `parse_artifact_shape` (pub(crate)
`ArtifactShape`) so the TLC backend (specodelic-ug3) reuses the same
parse. `run()` validates IR integrity (`invalid_model`) before IR↔artifact
agreement. Dogfood catch: the specodelic-len artifact regeneration
covered `.tla`/`.check.json` but left `compile.toml`, `*_props.rs`, and
`model_check.tla`'s StateValues stale against the same commit's spec
edits — all regenerated here (the exact staleness class the new gate
exists to catch).

Gates: just ci + lint-specs 0 + graph 0 dangling + openspec strict 8/8;
corpus model-check 18/18 exploration_only under the new gate.

## #44 — `exploration_only`: a completed model-check run is never a clean verdict (specodelic-len)

`spk model-check` reported `outcome: no_counterexample` with
`invariants_checked: []` — a clean verdict over an empty invariant set,
i.e. verification that did not happen (visible in every committed
`specodelic/*.check.json`). Fixed honestly, in the direction of the
spec: the native backend interprets guards as prose (Decision 3,
Option A) and executes zero invariant predicates, so a completed
exhaustive exploration now terminates in a new `exploration_only`
outcome — explicitly NOT clean, and consumers (verify's gate) must
treat it like `timed_out`. `no_counterexample` stays reserved for a
backend that actually executed invariant predicates; the CORR-002
cap+1 confirmation re-run survives, now distinguishing "space ends
within the bound" (`exploration_only`) from "genuinely truncated"
(`timed_out`) instead of gating a clean claim.

En route, the `.tla` emission was fixed for opt-in TLC: the module
header is now the single-line TLA+ form (`---- MODULE merge ----`) —
the previous three-line box was not a parseable header — and `Next`
carries a closing `UNCHANGED vpc` stuttering disjunct so terminal
states don't read as engine-side deadlocks. All 18 corpus artifacts
regenerated; `specs/model_check.md` gained the `exploration_only`
state + `finish_exploration` transition + `exploration_run_is_not_a_clean_verdict`
property; `specs/compile.md`'s `model_to_tla` wording and
disjunct-count property updated.

## #43 — `spk hooks install`/`uninstall`: the dual-format gate wired into the hook chain

Dual-format drift is now caught at commit time, not just in CI:
`spk hooks install` wires `spk lint openspec` into the repo's
`pre-commit:` stage as a marker-guarded managed block (`# <!--
SPK:START/END -->` comment lines), built on genesis 0.8 `git_hooks`
(framework detection, marker conventions). Design points grounded in
verified behavior:

- Never claims `core.hooksPath`, never writes `.git/hooks/*` or
  `.beads/hooks/*` — the beads → lefthook chain keeps flowing; the
  lefthook config is the sanctioned extension point.
- Two-case anchor with children-indent inference: lefthook 1.13.6
  rejects duplicate `commands:` keys (verified: `mapping key
  "commands" already defined`) and mixed-indent keys within one
  mapping, so the entry is inserted *inside* an existing `commands:`
  mapping at the existing entries' indent; the full `commands:`
  wrapper is injected only when the stage has none.
- genesis `lefthook::ensure_wired` is NOT used for injection: verified
  it glues the END marker onto the next existing line (its own tests
  pin `END  parallel: true`), which with comment-prefixed markers
  turns that line into a YAML comment — silently deleting the
  following key. Local injection keeps markers on their own lines;
  upstream consolidation filed (specodelic-x56:
  `ensure_command_wired`).
- Install reports a gate dry-run over the envelope
  (`data.gate_dry_run`); a failing gate is a warning carrying the
  failure summary and an `spk hooks uninstall` escape hint — never a
  commit trap.
- Uninstall strips only the marked block; an install-appended stage is
  left as a documented empty section; uninstall on an unwired repo is
  a successful no-op.
- New capability spec `hooks` (dual format) lands under `openspec/`
  at archive time; husky and prek repos get labeled refusals with
  manual-wiring hints.

Gates: 82 unit + 46 integration tests green; change at the approval
gate: `openspec/changes/add-hooks-install/`.

## #42 — total_refs file-scoped for `id: spec` files: the self-containment law is enforced

A Rule-of-5 review of the unification change set demonstrated that the
spec-integration law "deltas stay self-contained — wiki-refs resolve
only within the file" was unenforced: all `id: spec` files shared one
resolution bucket, so a dotted ref resolved against ANY dual-format
file's rows — a typo colliding with any row anywhere passed CI
(demonstrated empirically). Fix: `total_refs` and the graph's dangling
detection resolve `id: spec` files against the file's OWN rows only
(other file ids keep corpus-wide resolution; all current dual-format
refs are local, zero churn). Also: the capability-format check's
remediation now distinguishes deltas (mirror the ADDED text) from
capability specs (keep ## Requirements), and the migrated model-check
spec cites `specs/model_check.md` by prose path instead of a
metasyntactically-skipped `[[model_check]]` link. Residual, by design:
a single-segment file-id ref from a dual-format file (e.g.
`[[specodelic]]`) is skipped as metasyntactic rather than flagged —
none exist; revisit if one appears.

## #41 — `spk explain dual-format`: the protocol + migration recipe served offline

The embedded primer gained a seventh topic: the spec/openspec dual-
format protocol — both grammars, the `id: spec` naming law, the
enforcing rule (`linter.dual_format_valid`), and the migration recipe —
so a consumer hit by a dual_format_valid finding can act without repo
access. Topic ids grow at the end, never renumbered (OCP bias). README
and docs pages updated (six → seven topics). CHANGELOG #40 added the
CI-side capability-format check; this closes the primer-side gap the
same Rule-of-5 review flagged (DRAFT-001).

## #40 — Migration recipe + capability-format CI check (Rule-of-5 review of the unification)

A Rule-of-5 review of the unification code asked whether migration to
specodelic compliance is clearly guided. Findings applied: the
migration recipe (frontmatter → specodelic tables → mirrored
Requirements → gates) now lives in `openspec/project.md`, with worked
examples; the `dual_format_valid` missing-half message points at it
(previously at the sync script, which cannot help when a half is
absent); stale facts fixed (`ddl` → `spk`, genesis-vibes 0.8); and the
enforcement gap closed — a capability spec under `openspec/specs/
` without frontmatter or specodelic tables now FAILS CI
(`just sync-sections` capability-format check, stdlib-unittest-tested
by `just sync-sections-test`), because `spk lint` parse-skips
frontmatter-less files and would never see it. Archived deltas stay
exempt (pre-protocol evidence).

## #39 — `linter.dual_format_valid`: the dual-format protocol is tool-enforced

A new lint rule recognizes dual-format files structurally: any file
carrying an `## ADDED Requirements` section must declare `id: spec`
(openspec hard-requires the `spec.md` filename) and pair it with a
sibling `## Requirements` section — a half-format file is now a lint
finding, not a convention. The parser records both marker headings, so
the rule needs no disk re-reads; plain corpus specs (no ADDED section)
are exempt. The two spike fixtures in
`openspec/changes/archive/2026-09-28-spike-dual-format/` were completed
to full dual format (mirrored `## Requirements` siblings) so the
protocol's own evidence lints clean. Unification of the two spec systems
is now closed end to end: `add-dual-format-deltas` archived via the
verbatim recipe (`spec-integration` capability), `add-model-check`
archived and migrated to dual format, and the protocol enforced by
`spk lint` (#38 archived the changes).

## #38 — Unification closed: spec-integration + model-check capability specs archived

The two remaining complete-but-unarchived openspec changes were archived,
completing the dual-format unification: `add-dual-format-deltas` via the
verbatim recipe (`just archive-change` — its dual-format delta is now
byte-identical at `openspec/specs/spec-integration/spec.md`), and
`add-model-check` via plain archive (its plain delta seeded the new
`openspec/specs/model-check/spec.md`). `spk feedback` gained `--title`
passthrough for genesis-vibes 0.8's `FeedbackArgs.title`.

## #37 — Dual-format protocol: openspec engineering truth is specodelic-lintable

The repo's two spec systems now share requirement content instead of
duplicating it. Every openspec change delta is a *dual-format file* —
frontmatter + `Constraints`/`Model`/`Properties` tables alongside the
openspec `## ADDED Requirements`/`## Requirements` grammar — authored
once, validated by both parsers (`openspec validate --strict` and
`spk lint`), and lintable after archive. Because openspec hard-requires
the filename `spec.md`, dual-format files declare `id: spec`; the
reference index in `lint.rs` and `graph.rs` was fixed to aggregate row
sets per file id (several `id: spec` files previously overwrote each
other, dangling every cross-row link — found by linting the real openspec
tree). Archive runs `openspec archive <id> --skip-specs` and copies the
delta verbatim into `openspec/specs/<cap>/spec.md` (`just
archive-change`), bypassing the archiver's lossy regeneration; the
section-sync check (`scripts/check_section_sync.py`, wired into `just
ci`) fails any drift between a file's ADDED and Requirements sections.
All four archived capability specs (compile, doctor, embedded-guide,
lint-findings) are migrated to dual format — `just ci` now gates
`openspec validate --all --strict`, `spk lint openspec`, and section
sync (beads specodelic-3gd, openspec add-dual-format-deltas).

## #36 — `spk model-check`: the model_check step with the native stateright backend

`spk model-check <files>` runs `specs/model_check.md`'s run state machine
against compile's output — it never re-compiles: a missing
`<stem>.tla` artifact is a labeled `missing_artifact` error with a
`spk compile` hint, never a silent run. The native default backend
(stateright, embedded — no external binary) interprets the compiled
`ModelIR` as a program-counter model: initial state = first listed,
transitions as always-enabled actions, exactly the semantics the
committed `.tla` emission commits to. The run explores exhaustively
within the stated bound (`--max-depth` default 100, `--max-states`,
`--timeout-secs`) and reports `no_counterexample` or `timed_out` — a
reached cap means exhaustiveness cannot be proven, so it is reported
`timed_out`, never collapsed into clean.

Honesty note (openspec `add-model-check` Decision 3, Option A — approved):
the corpus language has no executable predicate semantics, so the native
backend checks no user invariants and reports `invariants_checked: []`
rather than implying a semantic check that never ran; the counterexample
leg of model_check's contract awaits a predicate-fragment decision
(beads specodelic-mp1). Run reports persist as `<stem>.check.json`
next to the compile artifacts, carrying the consumed module's SHA-256 —
`verify` (specodelic-1pv) reads that hash to reject stale clean results
(`rerun_on_model_change`). TLC stays opt-in and moves to
specodelic-ug3. Dogfood: all 18 corpus files check clean
(`no_counterexample`) within the default bound (beads specodelic-nx7).

Rule-of-5 review of the implementation set (converged stage 4, verdict
READY WITH_NOTES → all fixes applied): (1) artifact-consistency guard —
the run interprets the live spec's IR, so the committed `.tla` is parsed
(StateValues set + Next disjunct ids) and compared before any run; a
mismatch is a labeled `stale_artifact` error with a `spk compile` hint,
closing a drift hole where a run would check the new model while hashing
the old artifact (reproduced before the fix); (2) `depth_reached == cap`
ambiguity resolved — a confirmation re-run at cap+1 (depth-cap-only runs)
proves exhaustiveness, so `--max-depth` equal to the model's diameter now
reports `no_counterexample` instead of a false `timed_out`; (3)
`RunReport`/`Backend`/`Bound`/`Outcome` derive `Deserialize` (verify's
read path); (4) the SPECODELIC managed block advertises
`spk model-check`; (5) cycle/self-loop fixture test; design.md Decision
2/3 carry the post-review amendments.

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

## #64 — external completeness implemented: the five linter.external_completeness rules (specodelic-b15)

The optional checker's manifest format was the last open question (mp1
row 10) — decided of record in `linter-external_completeness.md`'s Notes:
a checklist is a dedicated `*.checklist.md` artifact OUTSIDE `𝒦` (flat
`## Items` list, `## Mapping` table with exactly
`item`/`status`/`mapped_ids`/`rationale` columns, presence-of-file = the
declaration), rejecting the degenerate-spec-file option that would have
grown `𝒦` after all. Implemented TDD-style: a lenient manifest parser
(`src/checklist.rs` — a manifest that cannot be read emits
`checklist_well_formed` findings, never silently vanishes) and the five
rules in `src/lint.rs` — `checklist_well_formed`,
`every_item_accounted` (also owns the status set), `covered_maps_resolve`
(mapped ids must resolve to real constraint/property rows — a file id or
section anchor is machinery, not a claim), `waiver_has_rationale`, and
`no_duplicate_claim` (one claim per item, never double-reported as
unaccounted). Non-gating by construction: the pass runs only when a
checklist is declared, never touches any file's `linted` state, and
`spk compile`'s precondition gate deliberately runs `lint_corpus` without
it. `mapping_naturality` stays aspirational — the rename tool reaching
into `mapped_ids` is deferred to its own ticket (until then a renamed id
dangles loudly, `covered_maps_resolve` fires and names it). Corpus
unchanged (no checklist declared: `not_applicable`), gates green.

## #65 — external completeness hardening: Rule-of-5 fixes over specodelic-b15

The Rule-of-5 review of the shipped change set found one CRITICAL-class
bug and five advisory gaps; all fixed. CORR-001 (false green): the
missing-sections guard keyed on the LAST header seen, so a manifest with
`## Mapping` but no `## Items` — or vice versa — reported zero defects
and linted clean; replaced with section-visibility flags + regression
tests (TypeSafe-verified @ 0.78, deterministic binary reproduction).
Advisories: UTF-8 BOM tolerated at manifest parse (EDGE-001); a manifest
declaring zero items is itself a `checklist_well_formed` defect (EDGE-002
— recorded in the decision text); `### `-depth headers inside a section
are prose, never section switches (EDGE-003); status/item mapping cells
tolerate surrounding backticks, matching the header row's leniency
(CLAR-002); a bare single-segment mapped id's finding now teaches the
dotted `file_id.row_id` spelling (CLAR-003); the lint payload carries
`checklists_declared` so a consumer can tell an empty pass from a
skipped one (EXCL-002). Gates green.

## #66 — the error contract is a published spec file (`specs/errors.md`)

add-error-contract phase 1 (beads specodelic-uie, tasks 1.1–1.4): the
cross-cutting output contract moves from CHANGELOG lore (#48's
`specodelic-7rr` exit-code/envelope work) into `specs/errors.md` — three
`extension_point` rows (`envelope_error_kind`, `exit_code_mapping`,
`remediation_hint_present`) plus eight invariant rows (namespaced label
shape, emitting failure terminals, graph-decidable failure classes,
typed negation guards with an orchestrate carve-out, single labeled
failure, label-falsifying properties, contract-exclusivity, enforcement
routing). The RED is a graph fixture
(`error_contract_rows_are_published`) asserting the three rows are
published and property-covered; it failed before the file existed and
passes after. Enforcement is routed three-tiered (existing checkers /
future `linter-failure_shape` / pipeline fixtures) — no tier-2 rule
claims today's `spk lint`. No core Revision consumed (D7: Revision 9
stays free for add-observability-contracts). Corpus artifacts
regenerated (errors.{toml,tla,props,check.json}); dogfood green.

## #67 — every failure terminal now emits a labeled, falsifiable error

add-error-contract phase 2 (beads specodelic-uie, tasks 2.1–2.5): the
corpus's 15 mute failure terminals are restructured — `compile.md`
splits `failed` into `extract_failed`/`emit_failed` per its own
`compile_is_total` two-class argument, and every file with ≥2 failure
transitions carrying differing negation citation sets splits its failure
state per class (rename ×3, merge ×3, linter-schema_shape ×3, coverage/
referential_integrity/ears_syntax/external_completeness/model_shape/kinds
×2 each); single-class files keep `failed` and gain the `emits` edge.
Every failure terminal emits exactly one file-owned effect-kind error
Constraint whose label is file-id-namespaced
(`compile.extraction_failure(row_id, reason)` …) and points `satisfies`
at the three `errors.md` contract rows; every label carries a unit
property asserting the exact label. The `¬x_ok.guard` prose idiom is
replaced by typed negations citing exactly the negated sibling's
citation set — `orchestrate.md`'s four stage-fails stay prose on the
recorded D2 carve-out, now one failure state per stage
(`lint_failed`/`compile_failed`/`model_check_failed`/`verify_failed`).
The RED fixture `failure_terminals_emit_labeled_errors` derives
terminals/citation sets/label sets mechanically from `spk graph` and
failed pre-change (15 mute terminals, 0 emitting). Corpus artifacts
regenerated (20 × {toml, props, tla, check.json}); all `just ci` gates
green.

## #68 — `linter-failure_shape.md`: the error contract's tier-2 checker, spec-only

add-error-contract phase 3 (beads specodelic-uie, tasks 3.1–3.3): the
checker file lands on the `linter-external_completeness.md` shape —
`terminal_states_emit` (v1 scope: failure terminals only; `timed_out`/
`exploration_only` are the stated non-goal, keeping the exit-code
question visible for a later Revision), `error_labels_unique` (per-file
variant-head uniqueness — trivially green corpus-wide today because
error labels are file-id-namespaced; exists so a format change cannot
silently drop the namespacing law), and `guard_negation_total`
(citation-set union equality against success siblings, or membership of
the recorded carve-out list — currently only orchestrate.md's four
stage-fails). The class rule stays the graph-decidable form (D2a); no
Checker Ownership row until implementation (D6). Implementation ticket
filed: specodelic-ct5 (notes AGENTS.md's stale 6pi blocker to the
maintainer). STATUS §1 inventory gains `errors.md` +
`linter-failure_shape.md`; docs SUMMARY updated; corpus artifacts
regenerated (21 files). Fixture hardening en route: the RED fixture's
terminal/sibling heuristics now key on the STATE segment of the node id
— `linter.failure_shape`'s own file id contains "failure".

## #69 — Rule-of-5 fixes over the error contract: the guard rule's scope and the label shape

TypeSafe-verified Ro5 pass (jev-1.13.0, 3/3 HIGH findings verified
@ 0.92/0.87/0.96) over the add-error-contract change set; all fixes
applied. CORR-001 (HIGH): `guard_negation_typed`'s "zero **intra-file**
constraints" scope made `rename.md`'s typed `reject` (cross-file
citations) malformed as written, and the singular "the success
transition it negates" didn't cover `linter-schema_shape.md`'s negated
disjunction — the rule now counts all cited ids and states the
union-of-branches reading (errors.md row + delta rows + both mirrored
Requirement sections; design D2 carries a post-review correction note).
CORR-002 (HIGH): all 31 error Constraint exprs carried a prose suffix
after the variant head, literally violating `error_expr_shape`'s
"expr IS `<file-id>.<variant_head>(field, …)`" — suffixes stripped;
exprs are now exactly the label shape. CORR-003 (HIGH): the delta's
tier-1 list claimed "graph citation-set comparison" is enforced today —
no such checker exists (it is ct5's future derivation); corrected to
`coverage`, matching `errors.md`. CORR-004 (MEDIUM, unverified):
"failure terminal" now has an operational definition (terminal state;
failure = `*fail*`-named state segment) in `linter-failure_shape.md`.
CLAR-003: STATUS errors.md row cites #66–#68. Gates: just ci, lint-specs
21/0, sync-sections, openspec strict; artifacts regenerated.
