# rename-composition-associative

## Invariant

For a link-closed corpus `I`, a shape-valid owner id `a`, and two collision-free
fresh ids `b` and `c` in the same owner family (all Intent ids, or all
`intent.row` ids of the same intent):

```
apply(rename(I, a, b)) then rename(·, b, c)
  == rename(I, a, c)
```

Equality is over the final file map (paths, bytes) and the removal set. The
intermediate repo is materialized by applying the first outcome's writes and
removal, not by reusing stale spans.

## Why it matters

Composition is the associativity half of `rename_naturality`. It is what lets a
batch rename be specified as one transaction over a sequence of pairs, and it
catches span/offset bugs that identity cannot: the second rename reparses text
produced by the first, so any padding, CRLF, or link-rewrite drift accumulates.

## Check mode

`always`. The equality is checked after the second step, with the intermediate
corpus itself checked to parse and remain link-closed (otherwise the comparison
is vacuous).

## Public API / seam

`rename::run` (`src/rename.rs:65`), `plan_rename` (`:195-286`),
`writer::apply` (`src/acset/writer.rs:234`). The property driver applies
`RenameOutcome.writes`/`remove` to the input map and calls `run` again.

## Generator sketch

- Corpus: 1–2 spec files with all editable families; every structured link
  resolves (generate links only to defined ids).
- Draw `a` from defined ids. For an Intent `a`, draw `b` and `c` from fresh
  dotted ids that collide with nothing; for a row `a = intent.local`, draw fresh
  locals and build `b = intent.b_local`, `c = intent.c_local`.
- Generate optional CRLF, padded cells, backticked id cells, and links to
  `a` / `a.<child>` in other files.
- Reject (assume) any pair where `b` or `c` already appears in the corpus.

## Predicted at HEAD `c76e2b0c`

**GREEN** for link-closed corpora, moderate confidence. Each step is span-local
and the second step reparses, so link and cell rewrites should compose
byte-for-byte; the existing fixed `edits_compose` test covers c1→c2→c3.

Two caveats:

1. **Dangling alias.** If the corpus contains a pre-existing dangling link
   `[[b]]` and `a` is renamed to `b`, the first rename makes that link resolve.
   Renaming back then rewrites it, while the direct identity `a→a` would leave
   it dangling. This is a precondition violation (link-closed), but it shows the
   law is not true for arbitrary `&str` corpora.
2. **Kind-keyword ids.** If `b` is `invariant` or `unit`, the first step hits the
   `rename-kind-preserved` defect and composition differs from the direct
   rename. That is the same root cause, not an independent composition bug.

## What would make it red

- A writer that leaves stale spans or offsets so the second parse disagrees with
  the first emission.
- An intent rename whose filename mapping is not idempotent under chaining
  (`. → -` twice).
- `plan_rename` treating the intermediate id differently from the final id
  (e.g. owner lookup by local id failing after the first rewrite).
- Any accepted input where the first rename is not link-closed at the end,
  making the second rename's collision/verify behavior differ.

## Existing instrumentation

Missing at this generality. `tests/acset_writer_apply.rs:361-384` checks
`apply` composition for one fixture and one id triple; `tests/rename_writer_parity.rs`
checks parity, not composition.

## Assumptions and Open Questions

- [ASSUMPTION] "Same owner family" is required: a row rename's `new_id` must
  stay under its intent's namespace (`src/rename.rs:266-280`), so a composed
  Intent-then-row sequence has no single direct equivalent.
- Open: should the product guarantee composition for arbitrary corpora, or only
  lint-clean ones? Current verify only checks the post-state.
  (needs human input)

## Investigation Log

- Examined: `plan_rename`'s namespace/collision rules (`src/rename.rs:241-280`),
  `verify_would_be_repo` (`:288-352`), the writer's exact-raw link rule
  (`src/acset/writer.rs:345-376`), and the existing `edits_compose` test.
- Found: the spec states composition only in the abstract; the implementation's
  accepted-input precondition is "no dangling link that aliases the target",
  and nothing enforces it before the first step.
- Not done: no execution of the dangling-alias counterexample.