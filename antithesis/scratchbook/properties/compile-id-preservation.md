# compile-id-preservation

## Invariant

For every `spec` and every `id in spec.defined_ids()`, `id` appears unchanged in
at least one compiled artifact:

- TOML: `source` (the Intent id) or a `[[constraints]]` entry's `id`;
- props: a `// id: <id>` line;
- ModelIR: a state id, transition id, or emit key.

No id is silently dropped, renamed, or transformed by compilation.

## Why it matters

`compile.compile_preserves_ids` is the bridge between the corpus's typed ids and
the generated artifacts that `verify`, `model_check`, and downstream tooling
key on. A dropped state or transition id makes the TLA+ module and the ModelIR
disagree with the spec while every artifact still looks well-formed.

## Check mode

`assert`, over generated specs with all id-bearing families.

## Public API / seam

`compile::compile_spec` id-set check (`src/compile.rs:2016-2045`),
`Spec::defined_ids` (`src/spec.rs:178-191`), `ir_contains_id` (`:2052-2060`).

## Generator sketch

- Specs with Intent id, 1–3 constraints, 1–3 states, 1–3 transitions, 1–3
  properties.
- Include ids with unusual but parser-valid shapes (dots, underscores, long
  names) and ids that sanitize to the same Rust fn name (e.g. `p.1` and `p_1`)
  to exercise the disambiguation path.
- Compile; collect ids from the three artifacts exactly as the implementation
  does; diff against `defined_ids()`.
- The property should be independent of the implementation's collection: parse
  the TOML with `toml::Value`, grep `// id:` from props, and read `model_ir`.

## Predicted at HEAD `c76e2b0c`

**GREEN — gate.** `compile_spec` computes the artifact-id set and returns
`Err(stage = "compile_preserves_ids")` if any defined id is missing. A green
`Ok` therefore implies the property.

The implementation's check has one subtlety: property ids are recovered from
`// id:` comments, so a sanitization collision that drops a comment would be
caught. The fn-name disambiguation suffixes (`:1308-1319`) change the function
name, not the `// id:` line.

## What would make it red

- Emitting a property block without its `// id:` metadata comment (the id
  becomes invisible to the gate and to downstream tooling).
- Qualifying row ids in one artifact and leaving them local in another, so the
  set comparison fails or, worse, two ids collapse.
- `defined_ids` growing a family that no emitter covers (e.g. a new table
  section) without updating the artifact-id collection.
- A spec with duplicate local ids in one file: `defined_ids` yields duplicates,
  and the set comparison hides the duplicate. That is a lint violation, not a
  compile failure; the property should use lint-clean inputs.

## Existing instrumentation

Generated `id_preservation_holds` (out of scope) and the compile spec's
property mirror. No independent artifact-parsing oracle.

## Assumptions and Open Questions

- [ASSUMPTION] Ids are compared as exact strings; qualified vs local spelling
  is part of the id, and the emitters currently preserve the parsed spelling.
- None that change the invariant.