# compile-artifact-stem-collision-refusal

## Invariant

Given a corpus of specs with paths:

- if two distinct canonical files produce the same `artifact_stem`, the check
  returns a message naming both display paths and the
  `duplicate_artifact_stem` label, plus a remediation hint naming the escape
  (rename one file or use separate `--out-dir` runs);
- if two entries are the same canonical path collected twice, the check returns
  `None` (one file, one artifact set);
- the result is independent of the order in which specs are supplied.

## Why it matters

`compile.artifact_stem_unique` defines the refusal that happens before lint and
before any artifact write. A missed collision means one file's artifacts
overwrite another's in the out-dir — silent data loss that no later lint can
detect because the artifacts are not specs.

## Check mode

`assert` over generated path pairs/triples, including permutations.

## Public API / seam

`compile::artifact_stem_collision` (`src/compile.rs:1825-1853`),
`canonical_artifact_path` (`:1821-1823`), `artifact_stem` (`:1854-1871`).
Note `artifact_stem` has a special case: a file named `spec.md` uses the Intent
id with dots replaced by dashes, not the file stem.

## Generator sketch

- Paths drawn from: same basename in different directories (`a/pipeline.md`,
  `b/pipeline.md`), same canonical path spelled differently (`./x.md`, `x.md`,
  a symlink if available), `spec.md` files whose Intent ids collide or differ,
  and unique stems.
- Supply specs in multiple orders; assert the same result (presence, message
  contents, first/second path roles may differ — the property should compare
  the named set, not the order).
- Include a pathless synthetic spec (`path = None`) whose stem falls back to
  the Intent id with dots→dashes.

## Predicted at HEAD `c76e2b0c`

**GREEN.** A `BTreeMap` keyed by stem stores the first claimant's canonical
identity and display spelling; a second claimant with the same canonical
identity is skipped, otherwise the function returns the pair. Iteration over
input specs is deterministic for a fixed input order, and the check itself is a
fold, so permutations produce the same collision set (though the "first"
spelling in the message may differ).

## What would make it red

- `canonical_artifact_path` failing (nonexistent path) and falling back to the
  raw path, so two spellings of the same file are treated as different files.
  The property must create real files or accept the fallback semantics.
- `spec.md`'s Intent-id stem colliding with another file's stem (`pipeline.md`
  vs a `spec.md` whose id is `pipeline`) — the special case should catch it.
- Input order changing whether a pathless spec or a real file is recorded as the
  first claimant, changing the message but not the collision set. The property
  should not require message role stability unless the spec does.
- A duplicate canonical path where one entry is pathless (cannot be equal), so
  it is treated as a distinct file. That is correct by design; the generator
  must not expect otherwise.

## Existing instrumentation

Fixed/generated tests cover a collision and a repeated path
(`duplicate_stem_refused`, `repeated_path_is_one_file`, out of scope generated
artifacts) plus unit-level expectations. No order-permutation property.

## Assumptions and Open Questions

- [ASSUMPTION] "Same artifact stem" is the `artifact_stem` function's output,
  including the `spec.md` Intent-id special case.
- None that change the invariant.