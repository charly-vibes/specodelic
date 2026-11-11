# Subagent brief: specodelic-edrv — audit slice 3: binds_nothing vacuity advisory (tasks 3.1-3.2)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-edrv` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=specodelic`
- Read, in order: `openspec/changes/add-evidence-audit/design.md` (esp. the
  **`binds_nothing` on the warnings channel** decision — vacuity exposes
  eliminable concepts AND correct phase-models are legitimately sparse, so
  advisory-only, file-scoped, never exit-nonzero; the closed audit-local
  vocabulary; the open question that mandates the CHEAP check first:
  substring-free citation-vs-declared-ids, graph-backed fan-in deferred),
  `tasks.md` §Slice 3 (your tasks are 3.1 + 3.2 ONLY),
  the delta spec `openspec/changes/add-evidence-audit/specs/audit/spec.md`
  §"Audit honesty channels" (all three scenarios), and the delta layer
  `openspec/changes/add-evidence-audit/specs/audit/layer.md` rows
  `binds_respect_sparse` + `honesty_channels` (invariants) and `p_sparse` +
  `p_honesty` (property shapes your tests should anticipate).
- `src/audit.rs` as it stands at HEAD — slices 1-2 landed: oracle rows +
  pack-check rows with unified emission (`audit_row`/`recount`/`sort_rows`),
  `AuditClass` = {Current, Stale, Unchecked, Contradicted, New},
  `AuditClass::of(verdict)`, `exit_code`, `run()`, `cmd_audit` CLI.
  **`binds_nothing` is NOT yet in the enum — you add it.**
- `wai search "binds_nothing"`, `wai search "vacuity"`, and
  `wai search "h5ks"` — prior records. Supersession context: this ticket
  supersedes the MODEL-BINDING half of open `specodelic-h5ks` (catitas A2
  vacuity advisories; bowling rev1 was green yet vacuous). Do NOT close or
  edit `specodelic-h5ks` — its invariant side stays separate.
- Start your tdd-ro5 pipeline and execute each step's prompt literally:
  `wai pipeline start tdd-ro5 --topic="specodelic-edrv: binds_nothing"`
  Advance ONLY via `wai pipeline next` — never hand-edit the run YAML under
  `.wai/pipeline-runs/`.

## What to build

tasks.md 3.1 (red) + 3.2 (green): a fragment or claim citing NO declared
model element yields `binds_nothing` on the warnings channel; a legitimately
sparse phase-model with declared elements (e.g. declared bad states) must
NOT flag; never exit-nonzero.

RED (3.1) — write the failing tests first, pinning:
- Vacuity flags: an evidence unit (fragment/claim) whose text cites none of
  the spec's declared model-element ids ⇒ a `binds_nothing` row, surfaced
  in the class counts, warnings channel, exit 0, counts in BOTH views.
- Sparsity respected: a model that declares elements a fragment does not
  exercise produces NO `binds_nothing` flag — the check is citation-vs-
  declared-ids, never coverage-of-declared-ids.
- Honesty mix: a report whose rows are `unchecked`/`binds_nothing` exits 0.

GREEN (3.2) — make them pass:
- `AuditClass::BindsNothing` variant + `as_str()` "binds_nothing". The
  closed set in the delta spec always included it — completing the enum is
  not a schema change; check `parse_prior_report` still round-trips.
- The citation-vs-declared-ids check: substring-FREE (whole-id match, not
  substring) — fragment/claim text vs the spec's declared ids. Cheap check
  only; NO graph fan-in signal (design open question defers it).
- Wire into the audit walk; `exit_code` (src/audit.rs:526) untouched —
  binds_nothing rides the warnings channel exactly like unchecked.

Design tension to resolve deliberately: `binds_nothing` is evidence-level
vacuity derived from citation analysis — it is NOT `AuditClass::of(verdict)`
and cannot be. qqgn's invariant "class always of-derived" needs a SCOPED,
documented amendment (e.g. the citation check overrides/annotates the
verdict-derived class where the evidence unit cites nothing). Document the
amendment in a code comment at the override site and in your report.
Where it sits in `rank()` (the `--before` overlay order): warnings channel
argues rank 1 alongside unchecked — document your choice either way.

Sourcing pointers: property cells carry `**rust:**` fragments (verify
machinery; see `src/verify.rs:695` evaluator note); kernel claims attach to
Constraints-table rows (`src/kernel.rs:1111`); declared model elements are
the spec's Model-section states plus its declared row ids (the `&Spec` the
audit walk already has). The exact row-wiring (which evidence units get
checked) is your design job — let the red tests pin it, keep it minimal.

- Do NOT start 3.3 (declared-ids index sharing with lint — that is ticket
  l6md; a LOCAL index collection in audit.rs is fine), 4.x, or 5.x.
- tasks.md checkboxes 3.1 and 3.2 only.

## Hard scope guard

- Allowed files: `src/audit.rs` (variant + check + unit tests),
  `tests/audit_*.rs` (only if a CLI-level pin needs adding — prefer
  in-module unit tests),
  `openspec/changes/add-evidence-audit/tasks.md` (checkboxes 3.1 + 3.2 only).
- Never edit: `openspec/specs/`, `.espectacular/`, `pretender.toml`,
  `specs/` (the corpus), `.wai/resources/**`, this template,
  `src/conform.rs`, `src/main.rs`, `src/lib.rs`, `src/lint/**` (3.3 is a
  different ticket), `src/kernel.rs`, `src/verify.rs`, `src/parse.rs`.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Output discipline**: every command emits through
  `genesis::guide::Output::emit`; errors carry a remediation hint.
- **beads**: issues live in `.beads/issues.jsonl`; the orchestrator closes
  the ticket — you do NOT run `bd close`.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`; contract TOMLs for
  change-phase scenarios are orphans until archive — author them in the
  archive commit, never mid-change.
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers skill convention).
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.
- Test runner during iteration: `just test-smart` (falls back to
  `cargo test`); full `just ci` gate before you report done.
  **`export TMPDIR=/var/tmp/specodelic-xch1` (create it; it exists) is
  MANDATORY for every cargo run** — /tmp tmpfs OOM-killed cargo twice in
  the 8ebr run; don't wait for it to bite you.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `just test` → <result>
- `ah check` → <result>
- `openspec validate --all --strict` → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">
