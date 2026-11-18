# writer-edit-composition

## Invariant

For a source `x`, a parsed `spec`, and two chained renames
`f: a→b`, `g: b→c` with `b` fresh:

```
let step1 = apply(f, path, x, &spec)?;
let spec1  = parse(step1.text);
let step2  = apply(g, step1.path, step1.text, &spec1)?;
let direct = apply(compose(f, g), path, x, &spec)?;
step2.text == direct.text
```

for row renames and Intent renames separately. The intermediate path is used as
the input to the second step, so an Intent rename's file move is part of the
composition.

## Why it matters

`acset.writer.edits_compose` is the algebra law behind batch operations and
behind `rename` composition. It is the property most likely to catch offset
drift: the second edit's spans come from reparsing text produced by the first,
so any padding/terminator/link-rewrite mismatch accumulates visibly. It also
checks the file-move path is compositional.

## Check mode

`always` — after each step, assert the intermediate parses and still defines
`b`; at the end, assert byte equality with the direct edit.

## Public API / seam

`writer::apply` (`src/acset/writer.rs:234`), `Edit::Rename` (`:210-226`).
Composition is not a library function; the property composes two calls and
constructs the direct edit by chaining the ids.

## Generator sketch

- Full-family source, 1–3 rows; ids drawn from ordinary tokens, excluding kind
  keywords (see `rename-kind-preserved` for why that exclusion matters).
- Row leg: `a = intent.c{i}`, `b = intent.mid`, `c = intent.fin`; ensure none
  collide.
- Intent leg: `a = intent`, `b = intent2`, `c = intent3`; the direct edit maps
  `a → c` and expects the `c` filename.
- Optional padding, backticks, CRLF.
- Assert `step2.writes[0].0 == direct.writes[0].0` and byte equality, plus
  removals equal for the Intent leg.

## Predicted at HEAD `c76e2b0c`

**GREEN** for ordinary ids. The existing fixed test `edits_compose`
(`tests/acset_writer_apply.rs:361-384`) covers c1→c2→c3 on one fixture. The
writer's spans are local to the current text; reparsing refreshes them; the
link matcher's exact-raw rule applies identically at each step.

Caveat: if `b` is a kind keyword (`invariant`, `unit`), step 1 triggers the
`rename-kind-preserved` corruption, and composition breaks for that reason.
That is a shared root cause, not an independent composition defect.

## What would make it red

- A span recorded against the original source reused for the second edit
  without reparsing (the property's reparse makes this impossible if the driver
  is written correctly; a library-level `compose` that reused spans would be
  vulnerable).
- Intent filename mapping not idempotent under chaining (`a` → `b` writes
  `b.md`; `b` → `c` writes `c.md`; direct writes `c.md` — currently equal).
- Link rewriting that depends on the previous name, e.g. a link to `a.child`
  that becomes `b.child` then `c.child`; the direct path should also produce
  `c.child`. A prefix-matching bug in `wiki_link_repls` would show up here.
- A collision/collision-adjacent case where the intermediate id is not fresh;
  the generator must exclude it (the property is about composition, not
  refusal).

## Existing instrumentation

Partial: one fixed example. No generated pair, no Intent-rename composition,
no CRLF composition.

## Assumptions and Open Questions

- [ASSUMPTION] Composition is defined within one owner family. An Intent rename
  followed by a row rename has no single `Edit::Rename` with the same effect,
  because a row edit's `old` is already qualified by the old Intent id.
- Open: should the library expose `Edit::compose` so callers cannot compose
  incorrectly? Currently composition is a caller pattern.
  (needs human input)

## Investigation Log

- Examined: `apply`'s replacement collection and splice
  (`src/acset/writer.rs:234-333, 420-433`), `wiki_link_repls`, and the fixed
  `edits_compose` test (`tests/acset_writer_apply.rs:361-384`).
- Found: spans are recomputed from the reparsed intermediate; the link matcher
  is exact-raw, so chained rewrites are deterministic. Intent filename mapping
  is a pure `replace('.', "-")` and chains.
- Not done: no generated pair, no Intent-rename composition execution.