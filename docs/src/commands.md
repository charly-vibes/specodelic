# Command Reference

**Exit codes** (uniform across verbs): `0` = success (lint with zero
findings counts); `1` = the stage produced findings or a tool-level
failure; `2` = invocation error — nothing was processed (path not found,
no spec files matched, unreadable input). clap argument-parse failures
also exit 2. The JSON envelope's `envelope_kind` is `"error"` whenever
the exit code is nonzero-by-invocation.

All commands emit through the genesis envelope: JSON for pipes,
human-readable for TTYs (`--human` / `--format json` to override).
Failures carry a remediation hint.

## `spk lint <files|dirs>`

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
the failure envelope.

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

Derives the reference graph from every typed reference field: nodes,
edges (including a Transition's `from`/`to` state edges), fan-in/fan-out
per row, dangling references, typing violations (the Reference Typing
table — forbidden edges are reported, never recorded), and supersedes
cycles. The rename/merge/refactor advisors build on it.

- Spec: [graph](specs/graph.md)

Same discovery hint as `lint`: with an `openspec/` tree present, the
zero-files failure suggests `spk graph openspec`.

## `spk new <id>`

Scaffolds a spec file from the template — per-layer HTML-comment
guidance (valid kinds, what a guard may cite, law-case requirements),
rendered from the same constants the linter enforces.

## `spk migrate <file> [--dry-run]`

Wraps an existing openspec delta file in place into the dual-format
four-layer skeleton: generated frontmatter (`id: spec`, EARS scaffold
statement), wired scaffold layers (`scaffold_*` placeholder rows —
constraint, one-state model, deriving property), and a byte-identical
`## Requirements` mirror of `## ADDED Requirements`. Existing sections
pass through verbatim; only missing pieces are inserted. A file that
already carries the mirror is refused, never rewritten — re-running is
safe. The scaffold lints clean as written; keep it green while you
replace the placeholders. Files not named `spec.md` get a naming-law
warning (deltas must be `spec.md` with `id: spec`). `--dry-run` prints
the resulting content without writing.

## `spk compile <files> --out-dir <dir>`

Compiles a lint-clean spec into three artifacts: `<stem>.toml`
(Constraints), `<stem>_props.rs` (proptest scaffolding), `<stem>.tla`
(TLA+ module). Total or labeled failure — never a silent partial result.

- Spec: [compile](specs/compile.md)

## `spk model-check <files>`

Runs the compiled model through a model-check backend within a stated
bound (`--max-depth`, `--max-states`, `--timeout-secs`). Two backends:

- **stateright** (default) — embedded BFS exploration; interprets guards
  as prose and executes no invariant predicates, so it reports
  `exploration_only` (space exhausted within the bound) or `timed_out`.
- **tlc** (opt-in, `--backend tlc --tlc-jar <tla2tools.jar>`) — the TLA+
  TLC reference engine as a JVM subprocess over the compiled module,
  `-depth` as the stated bound (`--max-states` has no TLC equivalent and
  is a labeled error). The JVM binary comes from `PATH` or `SPK_TLC_JAVA`;
  a missing binary or jar is a `missing_checker` error, never a verdict.

The same prose-predicate honesty applies to both: no corpus invariant is
executable yet (Decision 3, Option A), so a completed run is
`exploration_only` — never `no_counterexample`, which is reserved for a
backend that actually executed invariant predicates. Run reports persist
as `<stem>.check.json` with the consumed module's SHA-256 and the
backend engine + version, so backends' reports are attributable and
comparable.

- Spec: [model_check](specs/model_check.md)

## `spk verify <files>`

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
  module fails closed as stale) and its outcome must be
  `no_counterexample` — which the native backend never reports
  (`exploration_only` is explicitly not clean).

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
re-compiles and never re-runs the model checker.

The properties gate's scratch crate lives under the system temp dir
(`specodelic-verify/`), with a shared `target/` dir so proptest
compiles once per machine. A retention policy keeps it bounded:
orphaned per-invocation crate dirs (from killed runs) are pruned after
24 hours, and if the shared `target/` dir exceeds 4 GiB it is dropped
whole — the next verify pays one cold proptest rebuild. Set
`SPECODELIC_VERIFY_SCRATCH` to relocate the scratch base (e.g. off a size-capped tmpfs).

- Spec: [verify](specs/verify.md)

## `spk doctor`

Classifies the workspace (`self_hosting` vs `consumer`), checks the
local corpus's `Revision N` against the binary's embedded
`format_revision` — warns, never fails, on lag. A `corpus discovery`
check names where the specs live (`specs/`, an `openspec/` tree with a
file count, or the discovery rule).

## `spk explain [topic]`

The embedded format guide: `format`, `ears`, `kinds`, `references`,
`lifecycle`, `lint-rules`. No repository access required.

## `spk init`, `spk feedback`

`spk init` writes/refreshes the `SPECODELIC` managed block in the
repo's `AGENTS.md` (rule catalog + format revision). `spk feedback`
opens a prefilled feedback channel for the tool.

## `spk hooks install`, `spk hooks uninstall`

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
  `no_counterexample`; the native backend's `exploration_only` is
  honestly not clean, so a native-only corpus halts here (use
  `--backend tlc --tlc-jar` for the predicate-executing reference
  engine).
- **Verify stage** — the conjunction of both gates (see `spk verify`).

`.data.stages` lists every stage with a status (`passed` / `failed` /
`skipped`) and the failed stage's exact findings; every skip carries its
reason. Rename is never part of a run. Re-running against an unchanged
repo is byte-identical.

- Spec: [orchestrate](specs/orchestrate.md)

## Refactor advisor

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
