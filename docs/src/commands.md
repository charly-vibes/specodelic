# Command Reference

**Exit codes** (uniform across verbs): `0` = a clean run; `1` = the stage
produced findings or a tool-level failure; `2` = invocation error —
nothing was processed (path not found, no spec files matched, unreadable
input). clap argument-parse failures also exit 2. The JSON envelope's
`envelope_kind` is `"error"` and `ok` is `false` whenever the run is not
a clean run — findings and failures never ride a success-shaped envelope
(specs/errors.md `envelope_error_kind` + `exit_code_mapping`).

All commands emit through the genesis envelope: JSON for pipes,
human-readable for TTYs (`--human` / `--format json` to override).
Failures carry a remediation hint. Hit jargon? The
[glossary](glossary.md) defines every term a cold read needs.

## `spk lint <files|dirs>`

**Use when:** you want the gate — before committing, in CI, or wired as
the pre-commit hook (`spk hooks install`). It is the first stage every
other verb assumes (`spk explain lifecycle`).

The gate. Runs every rule in the catalog over spec files (recursively;
files without frontmatter are skipped with a warning). Each finding
carries `rule_id` (`linter.<name>`) and `rule_semantics`.

- Spec: [linter rules](specs/linter-frontmatter.md) (one file per rule family)

When nothing was found and the repo carries an `openspec/` tree (consumer
repos keep their corpus under `openspec/changes/*/specs/`), the failure
hint names it — `spk lint openspec` (gh#2.2).

Ingestion is hostile-input hardened (specodelic-suz): only regular files
under a 2 MiB cap are ever read — a FIFO, device file, or oversized file
named `*.md` is labeled, named, and skipped (never blocking or
unbounded-memory); when nothing else was linted, the labeled notes ride
the failure envelope. A parse error (malformed frontmatter) fails the
stage (exit 1) even when the rest of the batch linted clean — a
corrupted spec is never a silent pass (specodelic-in9: the pre-commit
gate must not let it commit); the error names the offending file and
the file was NOT linted.

## Domain packs

A workspace extends the format with **domain packs**: `kind: profile` spec
files whose six manifest tables (Sections, Kinds, References, Checkers,
Floors, Requires) declare typed sections, fiber kinds, reference fields,
checkers, and floors. Lint discovers packs by corpus scan (no config file),
activates them when a file's vocabulary matches, and reports:

- `linter.pack_shape` findings — a malformed pack manifest is a labeled failure
- pack-activation notes (warnings channel, exit 0) and `orphan_vocabulary`
  labeled failures — pack vocabulary used with no pack discovered or declared
  is never silent: a `uses` edge naming an absent pack, or a pack-qualified
  token in a kind/field position whose namespace has no discovered pack,
  names the candidate pack and both remediations
- lifecycle/revision-skew advisories (draft packs activate advisory-first;
  a pack pinned to an older base `format_revision` is named)

The three in-repo standard packs live in [`packs/`](https://github.com/charly-vibes/specodelic/tree/main/packs)
(`data.lineage`, `numeric.predicates`, `empirical.registry`); a consuming
file opts in explicitly via a `uses` reference to the pack's frontmatter id.

- Spec: [packs](specs/packs.md)

A `*.checklist.md` file in the linted tree declares an external checklist
(specs/linter-external_completeness.md — mp1 row 10's manifest format):
a flat `## Items` list plus a `## Mapping` table
(`item`/`status`/`mapped_ids`/`rationale`). The five
`linter.external_completeness` rules then check every item is explicitly
`covered` (mapped ids resolve to real constraint/property rows) or
`waived` (with a stated rationale). This pass is optional and never
gates a lifecycle stage — no checklist declared, nothing checked;
`spk compile`'s precondition gate deliberately does not run it.

## `spk graph <files|dirs>`

**Use when:** you need to see who references what — dangling references,
fan-in/fan-out, typing violations, supersedes cycles — before making
rename, merge, or refactor decisions (the advisors build on it).

Derives the reference graph from every typed reference field: nodes,
edges (including a Transition's `from`/`to` state edges), fan-in/fan-out
per row, dangling references, typing violations (the Reference Typing
table — forbidden edges are reported, never recorded), and supersedes
cycles. The rename/merge/refactor advisors build on it.

**Text projections** (`--format edges|dot|mermaid`): `edges` writes the
raw six-column TSV edge list (source_id, source_kind, field, target_id,
target_kind, annotation — every recorded edge one row, every typing
violation one annotation row); `dot` and `mermaid` write byte-stable
Graphviz/Mermaid text projections (visual grammar: solid = state
machine, dashed = guards, bold = `emits`, dotted = traceability, red
dashed = dangling/violations). Rendering stays external — pipe into
`dot -Tsvg` or paste into a mermaid renderer; there is no `--render`.
The projections are the one documented exception to the envelope:
raw text straight to stdout, so awk/jq pipelines consume it without an
envelope parser. `--view wiring` (requires `--format`, composes with
all three) selects the wiring view: `constraints.satisfies` edges
project to file level (consumer file → the producer's published
contract), self-loops are dropped, and the remaining cross-file
instances aggregate per distinct pair; a corpus with no remaining
edges emits a labeled `no_wiring` element, never a silently clean
diagram. Derived diagrams of this repo's own corpus (wiring, states,
trace, schema) live in [graph views](graph-views.md); `spk explain
graph-views` carries the same taxonomy and one-pipe recipes for your
own corpora.

- Spec: [graph](specs/graph.md)

Same discovery hint as `lint`: with an `openspec/` tree present, the
zero-files failure suggests `spk graph openspec`.

Exit codes (both modes): 0 clean, 1 findings (typing violations —
the annotation rows still ride along in projections), 2 invocation
error (no spec files on an existing path is clean-empty, but a
nonexistent or unreadable path is an error — a typo'd path is not a
clean empty corpus).

## `spk new <id>`

**Use when:** you are starting a fresh spec and want the four-layer
skeleton plus the per-layer guidance comments instead of a blank file.

Scaffolds a spec file from the template — per-layer HTML-comment
guidance (valid kinds, what a guard may cite, law-case requirements),
rendered from the same constants the linter enforces.

## `spk migrate <file> [--dry-run]`

**Use when:** you have an existing openspec delta to wrap (plain form)
or — with `--rekey` — a 0.6.0-era dual-format file still carrying
`id: spec` that must be re-keyed to its real parent-dir-derived id for
the 0.6.0 → 0.7.0 corpus migration (`spk lint <tree>` finds them).

Wraps an existing openspec delta file in place into the dual-format
four-layer skeleton: generated frontmatter (the real id derived by the
naming law — a `spec.md` file from its parent directory, Revision 18 —
plus an EARS scaffold statement), wired scaffold layers (`scaffold_*` placeholder rows —
constraint, one-state model, deriving property), and a byte-identical
`## Requirements` mirror of `## ADDED Requirements`. Existing sections
pass through verbatim; only missing pieces are inserted. A file that
already carries the mirror is refused, never rewritten — re-running is
safe. The scaffold lints clean as written; keep it green while you
replace the placeholders. Files not named `spec.md` get a naming-law
warning (openspec requires the delta filename `spec.md`). `--dry-run`
prints the resulting content without writing. `--rekey` instead rewrites
an `id: spec` dual-format file (0.6.0-era) to its Revision 18 real id —
derived from the parent directory — re-keying every `[[spec.*]]` ref;
a file already carrying its real id is an idempotent no-op, and a plain
delta or undeducible id is refused without rewriting. This is the
sanctioned 0.6.0 → 0.7.0 corpus migration path: `spk lint <tree>` finds
the files, `spk migrate <file> --rekey` per file.

## `spk compile <files> --out-dir <dir>`

**Use when:** a spec is lint-clean and you want its executable artifacts
— the Constraints TOML, proptest scaffolding, and TLA+ module that
`model-check` and `verify` consume.

Compiles a lint-clean spec into three artifacts: `<stem>.toml`
(Constraints), `<stem>_props.rs` (proptest scaffolding), `<stem>.tla`
(TLA+ module). Total or labeled failure — never a silent partial result.

Property and invariant fragments opt in per cell with a `**lang:**`
marker whose tag is drawn from the closed set `{rust, py, ts}`; a tag
outside the set (e.g. `**go:**`) is a labeled extraction failure naming
the tag and the closed set, never silence. `**rust:**` is the only
executable tag — a `**py:**` or `**ts:**` fragment gates compile with a
labeled failure naming its emitter follow-up
(`add-py-fragment-emission` / `add-ts-fragment-emission`), never a
silent fall-through to Rust emission.

The tag shape is lowercase-initial (`[a-z][a-z0-9_-]*` before `:**`):
ordinary bold prose (`**Note:**`) stays prose, but a lowercase bold
label in fragment position (`**note:**`) is tag-shaped and therefore a
labeled extraction failure naming the tag and the closed set — keep
prose labels capitalized, or move them out of fragment position.

Two binding layers, never cross-wired:

1. **Artifact scaffolds (specodelic).** compile emits the fragment
   verbatim into the artifact and records it in metadata comments and
   the `.check.json`/manifest staleness keys. Those comments are
   staleness records, not contracts (CORR-001's correction).
2. **Contract bindings (espectacular).** ah binds existing tests to
   deployed scenarios via contract-TOML entries (`[[tests.cargo]]`,
   `[[tests.pytest]]`, `[[tests.shell]]` flags). Binding is not
   compilation — the layers have different wall-clock, caching, and
   toolchain stories (design D3 of the language-neutral binding
   change).

The claim path between them is `kernel.binding`: an invariant-kind
Constraint may carry a `kernel.binding` column, and compile extracts
the cell as an opaque string into the Constraints TOML artifact —
verbatim, never parsed, validated, or interpreted (design D4 of
add-min-expr-kernel: no registry; the name is namespaced against pack
binding columns). An external checker claims a constraint through
contract-TOML `flags` — binding its own tests to the deployed
scenarios — while specodelic never learns the checker's language: the
bridge carries the claim, it never absorbs the checker.

The boundary between the tools is tracing vs semantics: espectacular's
`property-untraced` finding is bookkeeping and may grow similar
tracing findings, but ah must never grow semantic findings — a
hypothetical `property-uncompiled` would enforce compile.md's
semantics from ah, crossing the sibling-tool boundary (read-only over
`openspec/`, never enforcing against the `specs/` corpus).

- Spec: [compile](specs/compile.md)

## `spk model-check <files>`

**Use when:** you want bounded evidence that the compiled model's
required claims hold (`spk explain lifecycle`: model_check requires a
complete Model section). Read the outcome honestly —
`claim_aggregate_governs` (specs/model_check.md) fixes the priority:

- `no_counterexample` — clean: a nonempty required set all verified over
  a completed bounded exploration (the native backend earns it when the
  corpus carries executable claims; it is not TLC-only).
- `counterexample_found` — a required claim was refuted; exit 1, the
  finding names the claim id.
- `timed_out` — the depth/state/time bound expired before the
  exploration completed; exit 0, but not clean.
- `exploration_only` — the exploration completed but required claims
  were unknown, unsupported, or missing — including a prose-only corpus
  whose required set is empty; exit 0, but not clean.

Only `no_counterexample` satisfies `spk verify`'s model gate.

Runs the compiled model through a model-check backend within a stated
bound (`--max-depth`, `--max-states`, `--timeout-secs`). Two backends:

- **stateright** (default) — embedded BFS exploration over the compiled
  model, executing the claims opted into evaluation: `**rust:**`
  predicate fragments compiled into the module run inside the
  exploration, and `**kernel:**` claims evaluate over the kernel
  grammar. Prose guards are never interpreted.
- **tlc** (opt-in, `--backend tlc --tlc-jar <tla2tools.jar>`) — the TLA+
  TLC reference engine as a JVM subprocess over the compiled module,
  `-depth` as the stated bound (`--max-states` has no TLC equivalent and
  is a labeled error). The JVM binary comes from `PATH` or `SPK_TLC_JAVA`;
  a missing binary or jar is a `missing_checker` error, never a verdict.

**Claim gates** (model_check.md's `required_claims_classified` +
`claim_aggregate_governs`): a run partitions each parsed file's
invariant-kind Constraints into a **required** set — those opted into
evaluation (`**rust:**` fragment, `**kernel:**`, or citation) — and an
explicitly **unchecked** set of prose-only invariants: prose wording, a
constraint's name, or a deriving Property never implies evaluation, and
advisory/effect/extension-point rows are never required. The outcome is
governed by the required claims' statuses with fixed priority: any
refuted required claim is `counterexample_found` naming it; absent
refutation, an exhausted time/state/depth budget is `timed_out`;
otherwise any unknown, unsupported, or missing required claim is
`exploration_only` with reasons — and so is an empty required set (a
prose-only corpus has nothing required, so a completed run stays
`exploration_only`). Only a nonempty required set all verified over a
completed bounded exploration reports `no_counterexample` — the native
backend earns it when the corpus carries executable claims; it is not
TLC-only. A refuted kernel claim (`counterexample_found` — CHANGELOG
#116's failing aggregate outcome) is findings: exit 1 with an error-kind
envelope (`ok:false`), the payload intact.

Evidence: the run report persists as `<stem>.check.json`
(`claim_schema_version 1`) with the consumed module's SHA-256, the
backend engine + version, canonical claim records (id, evaluator kind,
status, reason where not verified), `expected_claim_ids` /
`unchecked_claim_ids`, and a `scope_sha256` digest of the parsed
structured content and the consumed compiled artifacts. The digest
binds structured content, not filesystem paths: reordering CLI inputs
or editing prose alone preserves it, while two inputs differing in
invariant content never share one. Verify (and orchestrate's verify
stage) recompute the digest and the required claim set from the current
inputs and reject stale, foreign, or malformed reports with a rerun
hint naming the model-check command — a stored report is never
rewritten to manufacture evidence. A command's parsed inputs must
claim unique intent ids: two files with the same id fail
`duplicate_corpus_identity` with a rename hint before compilation,
evaluation, or report writes (Revision 18: dual-format files carry
real ids and compose like any corpus — the former `id: spec`
isolated-scope rule retired with `id: spec` itself). A `**kernel:**` constraint
cell that breaks the closed grammar is a labeled `kernel_grammar`
failure before any run — the same validation compile applies — so a
stale compile can never silently demote a malformed kernel claim to
unchecked and report clean. The honest non-failure
outcomes keep exit 0 — `timed_out` (the bound expired) and
`exploration_only` (a completed exploration over a corpus with no
executable claims) are real terminal outcomes, not failures
(model_check.md); `spk verify`'s model gate is what rejects them as
not-clean.

- Spec: [model_check](specs/model_check.md)

## `spk verify <files>`

**Use when:** you want the conjunction — executed proptest properties
*and* a current clean model verdict (`no_counterexample`). **Choose**
`verify` over re-running `model-check` when a current `.check.json`
exists and you want the full gate without re-exploring the model —
verify never re-compiles and never re-runs the model checker; choose
`model-check` when you need fresh bounded exploration evidence itself.
Neither is a substitute for the application's own test suite
(`spk explain lifecycle`).

Executes the emitted proptest scaffolding and rejects stale clean
results via the `.check.json` hash.

Both gates must hold — `verified` is the conjunction, never a single
green stage:

- **Properties gate** — the `*_props.rs` artifact is staleness-checked
  against the current spec by block-metadata fingerprint (ids, cases,
  generators, predicates — hand-translated predicate bodies keep the
  fingerprint, spec edits change it), then really executed: the artifact
  is staged into a scratch cargo crate and run via `cargo test`, so an
  un-translated `todo_predicate!` fails honestly (`properties_failed`),
  never skipped. Failing blocks report proptest's shrunk minimal input.
- **Model gate** — the `<stem>.check.json` run report must be current
  (its `artifact_sha256` matches the on-disk `<stem>.tla`; a missing
  module fails closed as stale), its `scope_sha256` digest and required
  claim set must match the current inputs (verify recomputes them —
  stale, foreign, or malformed reports fail with a rerun hint, never
  rewritten), and its outcome must be `no_counterexample` — earned only
  when every required claim verified over a completed bounded
  exploration (`exploration_only` and `timed_out` are explicitly not
  clean).

`--timeout-secs <N>` (default 600; `0` = unbounded) bounds the whole
cargo run on a wall-clock clock: a hanging (likely pathological)
predicate is killed and reported as a labeled `properties_timed_out`
block — never a silent hang, and never indistinguishable from a test
failure (no block verdicts exist after a kill, so the bound is raised
and the run repeated, not misread as a failing property).

`.data.status` is `verified` exactly under the conjunction, else the
first blocking stage (`missing_properties_artifact`,
`stale_properties_artifact`, `properties_failed`,
`properties_timed_out`, `properties_uncompilable`, `runner_unavailable`,
`missing_model_run`, `stale_model_run`, `model_not_clean`). Verify never
re-compiles and never re-runs the model checker. A blocked verify is
findings, never a success-shaped envelope: exit 1 with `ok:false` /
`envelope_kind:"error"`, the payload (and `.data.status`) intact.

Scope note: verify's stored scope digest binds the run's structured
*inputs* (contributing files, claim records, artifact SHA-256), not
result statuses — a hand-forged `check.json` with invented `verified`
statuses does not yield a clean model gate: verify recomputes the scope
and the required claim set from the current inputs, and any digest,
record-shape, or artifact mismatch is rejected with a rerun hint
(specs/verify.md `evidence_scope_bound`; the persisted report is never
rewritten).

The properties gate's scratch crate lives under the system temp dir
(`specodelic-verify/`), with a shared `target/` dir so proptest
compiles once per machine. A retention policy keeps it bounded:
orphaned per-invocation crate dirs (from killed runs) are pruned after
24 hours, and if the shared `target/` dir exceeds 4 GiB it is dropped
whole — the next verify pays one cold proptest rebuild. Set
`SPECODELIC_VERIFY_SCRATCH` to relocate the scratch base (e.g. off a size-capped tmpfs).

- Spec: [verify](specs/verify.md)

## `spk conform <file> --oracle scenarios.jsonl [--closed-world]`

**Use when:** you have evidence from *outside* the spec — traces recorded
from a legacy system, a curated case set, or a reference implementation —
and you want each trace classified against what the spec declares and can
actually execute. Every other verb evaluates the spec's own claims;
conform is the one verb that checks the spec against reality. It is
strictly **read-only** (help text states it): it never advances the
lifecycle, is never an `orchestrate` stage, and writes nothing.

Each JSONL trace record (one per line, carrying `id`, an optional `setup`
state, and a `trace` of actions with observations) receives exactly one
verdict from a closed set — never collapsed to pass/fail:

- `permitted` — a declared, executable claim covers the trace and it agrees
- `forbidden` — the trace *violates* a declared executable claim (in any
  invocation mode), or — only under `--closed-world` — nothing declared
  covers it
- `underspecified` — no declared Model element or claim covers the trace
  (open-world default: a spec gap, never a prohibition)
- `unknown` — a covering claim exists but is prose-only (prose is never
  silently checked)
- `unsupported` — the covering claim's evaluator kind is not executable
  in this run

The run report carries a conform-local `report_schema_version`, per-trace
records sorted by scenario id (no timestamps — reruns over identical
inputs are byte-identical), a `scope_sha256` binding the parsed spec and
the consumed corpus bytes, and a fixed `evidence_scope` field in **both**
the JSON envelope and the `--human` view — "agreement on the supplied
corpus; not a proof of behavioral equality". That scope is data, not a
disclaimer: a finite trace corpus agreeing with the spec never proves the
spec describes the system.

Exit codes: `0` = no `forbidden`/`unsupported` verdict; `1` = any such
verdict; `2` = invocation error or gate refusal (stale artifacts,
lint-dirty file, malformed corpus) with **zero** verdict records emitted.
`unknown` and `underspecified` surface as counts but never fail the run —
they are spec-gap findings, not refutations.

- Book: [conform — external oracle evidence](conform.md) (spec page lands
  with the corpus archive; see [book: verification boundaries](verification-boundaries.md))

## `spk parse <file>`

**Use when:** a downstream tool or script needs one file's structured IR
as JSON — syntax-only, it succeeds even where lint would fail (chain
`spk lint` yourself).

Emits the parsed Spec IR for one file as a JSON envelope (specodelic-9rv):
`data` carries the full structured layer — intent, constraints rows,
states, transitions, properties rows (cells as keyed maps, not generated
text), and structured links. Parse is syntax-only: it succeeds on files
that fail `spk lint` and embeds no lint status — chain `spk lint`
yourself (the envelope's hint names it). Exactly one file per invocation
(no globbing — consumers loop); unparseable or missing input is a labeled
error envelope with a remediation hint and a non-zero exit.

## `spk doctor`

**Use when:** onboarding a repo or after a format-revision bump — it
classifies the workspace and warns (never fails) on corpus revision lag.

Classifies the workspace (`self_hosting` vs `consumer`), checks the
local corpus's `Revision N` against the binary's embedded
`format_revision` — warns, never fails, on lag. A `corpus discovery`
check names where the specs live (`specs/`, an `openspec/` tree with a
file count, or the discovery rule).

## `spk explain [topic]`

**Use when:** you are offline or want the canonical wording — the
embedded format guide, no repository access required. Nine topics:
`format`, `ears`, `kinds`, `references`, `lifecycle`, `lint-rules`,
`dual-format`, `packs`, `graph-views`.

## `spk init`, `spk feedback`

**Use when:** provisioning an agent-facing repo (`init` writes the
SPECODELIC managed block) or filing an upstream issue (`feedback`).

`spk init` writes/refreshes the `SPECODELIC` managed block in the
repo's `AGENTS.md` (rule catalog + format revision). `spk feedback`
opens a prefilled feedback channel for the tool.

## `spk hooks install`, `spk hooks uninstall`

**Use when:** you want the dual-format lint gate (`spk lint openspec`)
to run on every commit — marker-guarded and additive in the lefthook
chain, never a commit trap.

`spk hooks install` wires the dual-format gate (`spk lint openspec`)
into the repo's pre-commit chain as a marker-guarded managed block in
`lefthook.yml` — purely additive (beads' `core.hooksPath` → lefthook
chain keeps flowing), idempotent, indentation inferred from the
stage's own children. Install runs the gate once and reports the
outcome (`data.outcome`, `data.gate_dry_run`); a failing gate is a
warning with the failure summary and an escape hint (`spk hooks
uninstall`), never a commit trap. Requires a lefthook config (never
created from scratch) and an `openspec/` tree; husky and prek repos
get a labeled refusal with a manual-wiring hint. `spk hooks
uninstall` strips only the managed block.

## `spk archive-companion <CHANGE_ID>`

**Use when:** archiving an openspec change whose deltas carry the
dual-format layer — the default `openspec archive` would strip it
(`spk explain dual-format` has the migration recipe).

`spk archive-companion` archives an openspec change while preserving
its dual-format layer (specodelic-fzo, GH#7): default `openspec
archive` regenerates deployed specs from parsed deltas and strips
frontmatter + the Constraints/Model/Properties tables — the companion
invokes the archiver with spec application skipped, verifies each
archived delta still carries the layer, and deploys it verbatim to
`openspec/specs/<cap>/spec.md`. Fail-closed: a delta lacking the layer
is refused before any copy, with the `spk explain dual-format`
migration recipe as the hint — the tool never deploys a stripped spec.
Re-running on an already-archived change skips the openspec invocation
and re-restores; a change with no spec deltas succeeds with an empty
restored list; `--dry-run` resolves and verifies the plan without
invoking openspec or writing anything. `just archive-change` delegates
to this command.

## `spk rename <old_id> <new_id> [files|dirs]`

**Use when:** an id changes and every referencing `[[link]]` (plus
checklist `mapped_ids` and the filename itself) must follow — computed
and verified in memory before any byte is written, so a failed rename
leaves the repo byte-identical.

The atomic rename (specs/rename.md): updates the defining row (an
Intent frontmatter id or a table row's qualified id) and every
referencing `[[link]]` — including child refs like `[[old_id.child]]` —
as one transaction. Declared checklists reach into the same law: every
`*.checklist.md`'s `mapped_ids` cells are rewritten too (bare and
`[[…]]` spellings alike), and the verify gate re-runs the
external-completeness resolution over the post-rename corpus, so a
missed cell is a labeled rejection, not a silent dangle. A file whose
Intent id is renamed also gets its filename renamed per the `-` ⇔ `.`
naming law. Everything is computed and verified in memory (re-parse +
zero dangling) before any byte is written, so a failed rename leaves
the repo byte-identical: collisions, unknown ids, and verify-gate
rejections are labeled failures with remediation hints.

- Spec: [rename](specs/rename.md)

## Merge check

**Use when:** two branch tips both touched spec ids and you want what a
textually clean merge cannot see — id collisions, dangling renames,
intersecting blast radii — checked *before* joining the trees.

`spk merge --branch <incoming-tree> [--base <ancestor-tree>] [current-tree]`
checks what a textually clean 3-way merge cannot see
([merge](specs/merge.md)), over the two branch tips' spec trees:

- **id collisions** — an id minted on both branches (absent from the
  ancestor, or divergently edited on both) is flagged as a collision.
- **dangling renames** — a rename on one branch plus a fresh reference
  to the old name on the other is flagged for replay (`spk rename` does
  the rewriting after the trees are joined).
- **blast radii** — each branch's touched ids get a fan-in/fan-out
  closure computed from that branch's own graph artifact; intersecting
  radii require human review (`needs_review`) even when the file diffs
  don't overlap.
- **relint gate** — the union tree must re-lint clean with zero dangling
  references before the merge reports `merged`.

Exit 0 = merged, 1 = findings (verdict `needs_review` or `failed`),
2 = invalid invocation (missing `--branch`, empty incoming tree).

- Spec: [merge](specs/merge.md)

## `spk orchestrate <files|dirs> [--out-dir <dir>] [--backend tlc --tlc-jar <jar>]`

**Use when:** you want the whole pipeline — parse → lint → compile →
model_check → verify — in the fixed stage order with each gate enforced,
halting at the first failing stage, in one invocation.

Drives the full pipeline — `parse → lint → compile → model_check →
verify` — in the order [orchestrate](specs/orchestrate.md) fixes, gating
each stage transition exactly as [specodelic](specs/specodelic.md)
specifies and halting at the first stage that fails:

- **Lint stage** — the six Checker Ownership checkers run in dependency
  order: frontmatter first, then referential_integrity → graph_shape →
  model_shape (branch A), ears_syntax (branch B), schema_shape (branch
  C). A failed checker's dependents are **skipped** (never invoked,
  never reported as failed) while independent branches still run and
  report. `linter.external_completeness` runs when a checklist is
  declared and never gates.
- **Compile stage** — advances past the coverage checker's verdict only
  (`compile_gate_matches_coverage`), never a looser or stricter check.
- **Model_check stage** — passes only when every file's outcome is
  `no_counterexample`; `exploration_only` and `timed_out` are honestly
  not clean, so a corpus whose required claims never all verify halts
  here — a prose-only corpus (empty required set) always reports
  `exploration_only`, while corpora with executable claims can pass on
  the native backend (use `--backend tlc --tlc-jar` for the TLC
  reference engine over TLA+ modules).
- **Verify stage** — the conjunction of both gates (see `spk verify`).

`.data.stages` lists every stage with a status (`passed` / `failed` /
`skipped`) and the failed stage's exact findings; every skip carries its
reason. Rename is never part of a run. Re-running against an unchanged
repo is byte-identical.

- Spec: [orchestrate](specs/orchestrate.md)

## Refactor advisor

**Use when:** you are planning a tidy-first split — advisory findings
(high unrelated fan-in, changeset subset ownership) that never gate a
run.

`spk refactor [paths] [--high-fan-in N] [--changeset id1,id2]` is the
tidy-first split advisor ([refactor](specs/refactor.md)): it flags nodes
whose incoming cross-file dependents share no namespace segment below the
root (nearest common ancestor = root — decision of record 2026-09-30),
and — with `--changeset` — nodes whose owned rows the changeset touches
only as a strict subset without depending on the rest. Every count comes
from `spk graph`'s derived edges, never an independent markdown walk. The
high-fan-in threshold is per-invocation configuration
(`threshold_is_per_repo_setting`; default 3). Findings are advisory:
exit 0 whether or not anything is flagged.

- Spec: [refactor](specs/refactor.md)
