# rename-collision-refused

## Invariant

If `new_id` is defined anywhere in the parsed corpus — as an Intent id or as a
qualified `intent.row` id — `rename::run` returns
`Err(RenameError::Collision { new_id, holder })` and produces no write-set. The
input bytes are unchanged.

## Why it matters

`rename.new_id_available` is the constraint that prevents a rename from
silently merging two distinct rows or shadowing an Intent. A collision refusal
is also the precondition for every other rename law: composition and identity
are only stated for collision-free requests.

## Check mode

`assert`. Universal over generated corpora and candidate targets; `Always`
equivalent.

## Public API / seam

`plan_rename` collision scan (`src/rename.rs:241-264`). It builds each spec's
ids as `intent.id` plus `intent.id.row` for every `defined_ids()` entry and
scans for `== new_id` before the owner is rewritten. `holder` names the file.

## Generator sketch

- Corpus of 1–3 specs with unique Intent and row ids.
- Target: draw an existing id uniformly from the corpus's id index. Also draw
  a *qualified* row id (`intent.row`) that exists, and an id that exists only as
  a state or transition id (`defined_ids` includes both).
- Assert `Err(Collision)` and that the holder path equals the file defining the
  target.
- A second leg asserts the negative: a fresh shape-valid id does not collide.

## Predicted at HEAD `c76e2b0c`

**GREEN.** The scan happens before any write-set is constructed, and the error
is returned directly. The identity case is not a collision because it
short-circuits earlier (and identity is the one case where `new_id` necessarily
exists).

## What would make it red

- Omitting states or transitions from the scan (they are in `defined_ids`, so a
  regression there would be caught).
- Comparing only local row ids, not qualified `intent.row` ids, so renaming a
  row to another file's local id succeeds.
- Treating a checklist mapped id as a definition (it is a reference; the current
  scan correctly ignores it, but a future "checklist defines ids" change could
  flip the law).
- A collision with a *dangling* link target: links are not definitions, so the
  rename is allowed. That is correct but worth stating in the generator's
  oracle.

## Existing instrumentation

Partial. `collision_rejected` is a spec row; in-module tests cover invalid
shapes and checklist rewriting, but no generated collision corpus exists.

## Assumptions and Open Questions

- [ASSUMPTION] Checklists are not id holders; `checklist::parse_str` produces
  mapped references, not definitions.
- Open: should a rename to an id that is *referenced but undefined* be refused?
  Current behavior allows it, and the post-rename verify may then make the
  dangling link resolve. (needs human input; see `rename-composition-associative`
  caveat 1)

## Investigation Log

- Examined: `plan_rename`'s collision scan (`src/rename.rs:241-264`),
  `Spec::defined_ids` (`src/spec.rs:178-191`), and checklist parsing
  (`src/checklist.rs:299`).
- Found: definitions are Intent ids plus row/state/transition ids; checklist
  mapped ids are references and never enter `defined_ids`, so they are
  correctly excluded. Dangling link targets are also not definitions.
- Not done: no product statement on whether undefined-but-referenced targets
  should be reservable; this is a policy question, not a code gap.