# Changelog

Append-only. Past entries are never edited — a correction gets a new entry,
the same discipline `specodelic.md`'s own `append_only_variants` rule
requires of spec files themselves. Displayed newest first; numbered
chronologically ascending (`#1` = oldest) so a new entry always gets the
next integer regardless of where it's inserted in the display order.

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
