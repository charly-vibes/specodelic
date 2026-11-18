# rename-write-set-realizes-target

## Invariant

If `rename::run(I, old, new)` returns `Ok(outcome)`, then applying
`outcome.writes` and `outcome.remove` to `I` produces a corpus `I'` where:

1. `new` is defined (`I'.defined_ids()` contains it, or the Intent id is `new`);
2. no structured **link** and no id-bearing cell still names `old`;
3. every structured link in `I'` resolves (zero dangling);
4. for an Intent rename, the moved file's path is the `id.replace('.', "-")`
   filename under the original parent, and `remove` names the old path.

## Why it matters

This is the operational meaning of `rename.old_id_fully_replaced` and
`atomic_operation` at the caller boundary: the tool returns a prescription, and
this property checks the prescription actually yields the renamed instance. It
is the bridge between the in-memory write-set contract and the corpus-level
guarantee.

## Check mode

`assert`. The driver applies the outcome, reparses, and checks all four clauses
per generated case.

## Public API / seam

`rename::run` (`src/rename.rs:65`), `run_via_writer` (`:83`), the write-set
construction (`:98-176`), and the verify gate `verify_would_be_repo`
(`:288-352`). The filename mapping is in `writer::apply`
(`src/acset/writer.rs:310-331`).

## Generator sketch

- Corpus: 1–3 specs with all editable families, unique ids, all links resolving;
  optionally a checklist manifest whose mapped ids name the renamed row.
- Rename shapes: Intent rename (`a` → fresh `b`) and row rename
  (`a.c1` → `a.c2`), both with fresh targets.
- Optional syntax variants: CRLF, padded cells, backticked id cells, links in
  other files.
- Driver applies writes/removal to a `BTreeMap<PathBuf, String>`, reparses all
  specs, and checks clauses 1–4. Clause 2 examines `Spec.links` targets and
  exact id cells (not prose).

## Predicted at HEAD `c76e2b0c`

**GREEN.** This is close to the verify gate's own promise:
`verify_would_be_repo` reparses the would-be corpus and checks zero dangling
before returning. Clause 2 follows from rewriting every link whose raw form is
exactly `[[old…]]`; a padded `[[ old ]]` link is left alone but still resolves
by its trimmed target, and if it would dangle the gate refuses. Clause 4 is
covered by `filename_follows_intent_id`.

The property is not fully tautological because it checks the write-set is
sufficient (the gate checks the text, not that the caller receives the right
paths/removal), and because clause 2's exact-id-cell scope is narrower than the
gate.

## What would make it red

- A writer path that emits the new id but omits a write (e.g. a changed file not
  added to `writes`).
- An Intent rename whose `remove` is missing or points at the new path.
- A link whose raw bytes are not exactly `[[old]]` but whose trimmed target is
  `old`: the gate would refuse if it dangles, so an accepted rename should
  never leave it stale. A parser change that stops including such a link in
  `spec.links` could break clause 2.
- A checklist mapped id that is not rewritten. The run loop handles checklists
  through `checklist::rewrite_text`; the verify gate lints them. A missed cell
  is predicted to be a refusal, not a green outcome.

## Existing instrumentation

Partial. `tests/rename_writer_parity.rs` and
`tests/snapshots/rename_baseline/` pin write-set bytes for fixed fixtures;
`rename.rs` unit tests cover checklist mapped ids. No generated
apply-and-reparse oracle exists for `rename::run`.

## Assumptions and Open Questions

- [ASSUMPTION] Clause 2 is scoped to structured references, not prose. The
  rename spec explicitly leaves prose mentions alone.
- Open: should the property require that `old` is absent from **all** cell
  values, or only id-bearing columns? The current code's any-cell rewrite makes
  the stronger version true for most ids but is the cause of
  `rename-kind-preserved` when ids alias keywords. (needs human input)

## Investigation Log

- Examined: `run_via_writer`'s per-file loop (`src/rename.rs:98-176`),
  `verify_would_be_repo`, and the writer's filename mapping
  (`src/acset/writer.rs:310-331`).
- Found: the verify gate checks parse + dangling + checklist resolution; it does
  not check that the write-set is sufficient on the caller's side, which is what
  this property adds.
- Not done: no execution; the predicate is a code-path reading.