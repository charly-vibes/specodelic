---
id: linter.schema_shape
kind: intent
statement: "WHEN a spec file's frontmatter has passed, THE linter SHALL reject it if any 𝒦-governed id-set (a variant table, a Constraint/Property kind value-set, or the Reference Typing table's field set) shrinks, reorders, or silently retypes an existing member across revisions, if a Constraint or Property row's own kind value falls outside its closed set, or if the parser has touched content inside a rationale/description field."
---

# Linter: Schema Shape Check

Runs after `linter.frontmatter` only, in parallel with
`linter.ears_syntax` and the referential/graph/model branch — per
`specodelic.md`'s Checker Ownership table, this checker and
`linter.ears_syntax` are independent terminal nodes that both feed into
`linted`, not a sequential continuation of one another.

Note this checker's scope narrowed relative to its first mention in the
decomposition plan: `no_boolean_columns` ended up owned by
`linter-model_shape.md` instead, scoped specifically to state/transition
rows where a boolean column would actually manifest as an ad-hoc state
machine. What's left here is schema evolution discipline
(`append_only_variants`), the closed-set discipline on the `kind` column
that Constraint and Property rows each separately carry (see `kinds.md`),
and the parser's own restraint around prose (`prose_untouched`) — three
properties about how the *tooling* behaves across time, across the
`kind` column specifically, and across fields generally, rather than
about any single file's current shape.

## Constraints

| id                     | kind      | expr                                                                                        | traces_to | satisfies |
|--------------------------|-----------|--------------------------------------------------------------------------------------------------|-----------------------------------------------------|
| id_set_grows_only         | invariant | `∀ 𝒦-governed id-set S (a variant-table's ids, a Constraint/Property row's own kind value-set, or the Reference Typing table's field set), revision r < r': S(r) ⊆ S(r'), appended only under a new Revision heading — no member removed or renumbered; a member's typing may narrow only in the same Revision that introduces the kind-split it depends on, and only if it invalidates nothing valid at r` | [[specodelic.append_only_variants]]  |          |
| id_set_order_stable       | invariant | `∀ 𝒦-governed id-set S, revision r < r': the relative order of members present in both r and r' is unchanged` | [[specodelic.append_only_variants]]  |          |
| constraint_kind_closed    | invariant | `∀ Constraint row: row.kind ∈ {invariant, advisory, effect, extension_point}` — see [[kinds.constraint_row_shape]]   | [[specodelic.constraint_kind_closed]] |          |
| property_kind_closed      | invariant | `∀ Property row: row.kind ∈ {unit, law}` — see [[kinds.property_row_shape]]                        | [[specodelic.property_kind_closed]]   |          |
| no_prose_field_parsed     | invariant | `the parser's AST never branches on the text content of a rationale/description field`            | [[specodelic.prose_untouched]]        |          |
| prose_field_passthrough   | invariant | `rationale/description content is stored verbatim and emitted verbatim in the compiled TOML`      | [[specodelic.prose_untouched]]        |          |
| kind_check_failure | effect | `linter.schema_shape.kind_check_failure(detail) — the label names its owning file per error_expr_shape` | [[linter.schema_shape]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |
| diff_failure | effect | `linter.schema_shape.diff_failure(detail) — the label names its owning file per error_expr_shape` | [[linter.schema_shape]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |
| parser_audit_failure | effect | `linter.schema_shape.parser_audit_failure(detail) — the label names its owning file per error_expr_shape` | [[linter.schema_shape]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |

## Model

### States
- `unchecked`
- `kind_checking`
- `diffing`
- `parser_audited`
- `passed`
- `kind_failed` (emits: `[[linter.schema_shape.kind_check_failure]]`)
- `diff_failed` (emits: `[[linter.schema_shape.diff_failure]]`)
- `parser_audit_failed` (emits: `[[linter.schema_shape.parser_audit_failure]]`)

### Transitions

| id                 | from            | to               | guard                                                                                  |
|--------------------|-----------------|------------------|-------------------------------------------------------------------------------------------|
| begin              | unchecked       | kind_checking    | `file passed linter.frontmatter`                                                          |
| kind_ok            | kind_checking   | diffing          | [[linter.schema_shape.constraint_kind_closed]] ∧ [[linter.schema_shape.property_kind_closed]] |
| kind_fail | kind_checking | kind_failed | `¬([[linter.schema_shape.constraint_kind_closed]] ∧ [[linter.schema_shape.property_kind_closed]])` |
| diff_ok            | diffing         | parser_audited   | [[linter.schema_shape.id_set_grows_only]] ∧ [[linter.schema_shape.id_set_order_stable]] |
| diff_skip          | diffing         | parser_audited   | `no prior revision exists in repo history` — nothing to diff against yet                    |
| diff_fail | diffing | diff_failed | `¬([[linter.schema_shape.id_set_grows_only]] ∧ [[linter.schema_shape.id_set_order_stable]])` |
| accept             | parser_audited  | passed           | [[linter.schema_shape.no_prose_field_parsed]] ∧ [[linter.schema_shape.prose_field_passthrough]] |
| reject | parser_audited | parser_audit_failed | `¬([[linter.schema_shape.no_prose_field_parsed]] ∧ [[linter.schema_shape.prose_field_passthrough]])` |


## Properties

| id                         | kind | derives_from                                     | generator                                                    | predicate                                                                 |
|------------------------------|------|-------------------------------------------------------|-------------------------------------------------------------------|-----------------------------------------------------------------------------|
| shrunk_id_set_rejected         | unit | [[linter.schema_shape.id_set_grows_only]]             | `(revision_n, revision_n_plus_1)` where a member was removed from a variant table, a kind value-set, or the Reference Typing table | `check(revisions) == failed` — one generator, parametrized over which of the three id-sets it targets |
| reordered_id_set_rejected      | unit | [[linter.schema_shape.id_set_order_stable]]           | `(revision_n, revision_n_plus_1)` where two existing members of any 𝒦-governed id-set swapped order | `check(revisions) == failed`                                            |
| silent_narrowing_rejected      | unit | [[linter.schema_shape.id_set_grows_only]]             | `(revision_n, revision_n_plus_1)` where a Reference Typing field's target narrowed with no accompanying kind-split cited | `check(revisions) == failed`                                |
| constraint_kind_invalid_rejected | unit | [[linter.schema_shape.constraint_kind_closed]]      | `constraint_row_with(kind: "made_up")`                              | `check(file) == failed`                                                    |
| property_kind_invalid_rejected | unit | [[linter.schema_shape.property_kind_closed]]          | `property_row_with(kind: "audit")`                                  | `check(file) == failed`                                                    |
| kind_closed_passes             | unit | [[linter.schema_shape.constraint_kind_closed]]        | `arbitrary_file_with(only_closed_kind_values: true)`                | `check(file) == passed`                                                    |
| prose_roundtrips               | unit | [[linter.schema_shape.prose_field_passthrough]]       | `arbitrary_unicode_string()` as rationale content                  | `compile(parse(s)).rationale == s` — no normalization, no truncation       |
| appended_id_set_member_passes  | unit | [[linter.schema_shape.id_set_grows_only]]             | `(revision_n, revision_n_plus_1)` where only new members were added, to any of the three id-sets | `check(revisions) == passed`                                    |
| constraint_kind_advisory_passes | unit | [[linter.schema_shape.constraint_kind_closed]]       | `constraint_row_with(kind: "advisory")`                             | `check(file) == passed`                                                    |
| constraint_kind_effect_passes  | unit | [[linter.schema_shape.constraint_kind_closed]]        | `constraint_row_with(kind: "effect")`                               | `check(file) == passed`                                                    |
| constraint_kind_extension_point_passes | unit | [[linter.schema_shape.constraint_kind_closed]] | `constraint_row_with(kind: "extension_point")`                      | `check(file) == passed`                                                    |
| parser_ast_never_reads_prose    | unit | [[linter.schema_shape.no_prose_field_parsed]]         | `parser_audit_over_every_ast_construction_site()`                    | `no branch condition references the text of a rationale/description field` — the check itself is an implementation audit, see Notes |
| kind_check_failure_label_asserted | unit | [[linter.schema_shape.kind_check_failure]] | `kind_check_failure_raised()` | `error_label == "linter.schema_shape.kind_check_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
| diff_failure_label_asserted | unit | [[linter.schema_shape.diff_failure]] | `diff_failure_raised()` | `error_label == "linter.schema_shape.diff_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
| parser_audit_failure_label_asserted | unit | [[linter.schema_shape.parser_audit_failure]] | `parser_audit_failure_raised()` | `error_label == "linter.schema_shape.parser_audit_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
## Notes

**Resolves the follow-up `kinds.md` recorded when it landed:** that file
defined `property_row_shape`'s `kind ∈ {unit, law}` and
`constraint_row_shape`'s `kind ∈ {invariant}`, and `specodelic.md`
Revision 4 added the corresponding top-level invariants
(`constraint_kind_closed`, `property_kind_closed`), but no checker
enforced either yet. This file now does, as the first gate after
`linter.frontmatter` and before the existing revision-diffing — a Property
row with an invalid `kind` should fail here regardless of whether the file
has any revision history to diff against at all, which is why
`kind_checking` sits ahead of `diffing` rather than folded into it: the
two checks have genuinely different preconditions (`kind_checking` needs
only the current file; `diffing` needs a prior revision, hence the new
`diff_skip` edge for a file with none).

`no_prose_field_parsed` is a property of the *parser's implementation*, not
of any spec file's content — there is no generator that can produce a
"bad" spec file to test this against, only an audit of whether the parser
code path for `rationale`/`description` ever appears on the left side of a
conditional. Marking this `Needs Human Review` rather than forcing it into
the generator/predicate shape the other rows use: a static analysis of the
parser's source (does the AST-walk function ever pattern-match on prose
field contents) is the actual verification method, not a PBT run.

Checker Ownership table in `specodelic.md` is now fully accounted for
except `linter-coverage.md`, the last remaining file.

**Previous pass (resolving `specodelic.md` Revision 5's two pending
items):** `constraint_kind_closed` widened to `{invariant, advisory}` in
step with `kinds.md` Revision 2, and a `reference_table_grows_only`
constraint was added for the Reference Typing table specifically.

**This pass (`specodelic.md` Revision 6):** that separate
`reference_table_grows_only` constraint, along with the older
`variant_set_grows_only`/`variant_order_stable` pair, is retired and
replaced by the two rows above — `id_set_grows_only` and
`id_set_order_stable` — stated once, over *any* 𝒦-governed id-set, rather
than three times over three id-sets that all follow the identical rule.
No enforcement was lost: the same three id-sets (variant tables, kind
value-sets, the Reference Typing table) are still covered, by the same
`diffing` state and the same `diff_skip` fallback for a file with no prior
revision to diff against — only the naming collapsed from three
constraints to two. `constraint_kind_closed` widens again, to
`{invariant, advisory, effect}`, in step with `kinds.md` Revision 3.
`no_prose_field_parsed` remains the one row here still marked `Needs Human
Review`; nothing in this pass touches it.

**This pass (`specodelic.md` Revision 7):** `constraint_kind_closed`
widens a third time, to `{invariant, advisory, effect, extension_point}`,
in step with `kinds.md` Revision 4. `constraint_kind_extension_point_passes`
added as the matching acceptance case, same shape as the `advisory`/
`effect` rows above it. Nothing else in this file changes: `id_set_grows_only`
already covers any Constraint-kind value-set widening generically, so an
`extension_point` addition is diffed by the same `diffing` state as any
other, with no new branch.
