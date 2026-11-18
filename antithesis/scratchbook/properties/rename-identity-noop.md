# rename-identity-noop

## Invariant

For every parseable corpus `I` and every shape-valid id `a`,
`rename::run(I, a, a)` returns `Ok(RenameOutcome { writes: [], remove: None, .. })`
and applying that outcome leaves every byte unchanged.

## Why it matters

Identity is the unit of `rename_naturality` (`openspec/specs/rename/spec.md`)
and the floor that makes composition meaningful: if the identity case wrote
anything, a caller could not treat a no-op request as free. It also guards the
caller contract that an empty write-set means "nothing changed", which matters
because the caller applies the set as a transaction.

## Check mode

`assert`. The law is universal over generated corpora and ids, so it is an
`Always` equivalent. No liveness or reachability question is involved.

## Public API / seam

`rename::run` (`src/rename.rs:65`) delegates to `run_via_writer` (`:83`), whose
identity branch is `:88-96`. Note the order: `validate_shape(new_id)` runs
first, then `old_id == new_id` returns. The law therefore holds for
shape-valid ids; `run(files, "a b", "a b")` is an `InvalidNewId` refusal, not an
identity success.

## Generator sketch

- Corpus: 1–3 spec files with frontmatter + Constraints + Model + Properties,
  each id unique; optionally one checklist file.
- `a`: draw from `arbitrary_defined_id(corpus)` — Intent ids and qualified
  `intent.row` ids — plus shape-valid ids that do not exist, to cover the
  unknown-id path separately.
- For the identity leg, only defined ids are used (an unknown id + identity
  still returns `Ok` by the early return, which is a separate interesting case:
  `run(I, "ghost", "ghost")` succeeds without checking existence).

## Predicted at HEAD `c76e2b0c`

**GREEN.** The branch returns before parsing, owner lookup, collision scanning,
or verification. The only possible red is a future refactor that moves the
identity check after `plan_rename`, which would make identity fail on corpora
where `a` is unknown.

## What would make it red

- Removing the early return, so identity is routed through `plan_rename` and
  fails `UnknownId` on a corpus that does not define `a`.
- Returning a write-set that rewrites the id to itself (byte-identical content
  but non-empty writes would still violate `writes.is_empty()`).
- Relaxing `validate_shape` ordering: the law is stated only for valid ids, so
  a generator that draws invalid ids and expects identity would be a bad
  property, not a bug.

## Existing instrumentation

Partial. `identity_rename_writes_nothing` (`src/rename.rs:467-477`) is one fixed
example with a single file. No generated corpus.

## Assumptions and Open Questions

- [ASSUMPTION] "Shape-valid" means accepted by `validate_shape`
  (`src/rename.rs:354-366`).
- Open: should `run(I, unknown, unknown)` return `Ok` (current) or `UnknownId`?
  The identity law does not decide it; the early return makes it `Ok`.
  (needs human input if the API contract cares)

## Investigation Log

- Examined: `run_via_writer`'s identity branch (`src/rename.rs:88-96`) and
  `validate_shape` ordering (`:354-366`), plus the fixed identity test.
- Found: identity is checked before parsing, so an unknown id still returns
  `Ok`; shape validation is the only precondition.
- Not done: no product statement decides whether an unknown identity should
  refuse; the law itself is unaffected.