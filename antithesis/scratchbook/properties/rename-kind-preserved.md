# rename-kind-preserved

## Invariant

A row rename changes the row's `id` cell and every id-bearing cell, bullet, or
link that names the local id, and **no other cell of any row**. In particular,
all `kind` cells (Constraints: `invariant` / `advisory` / `effect`; Properties:
`unit` / `law`) are byte-identical before and after, so every row's assigned
kind is preserved (`rename.kind_unchanged`).

## Why it matters

`kind_unchanged` is a named constraint in the rename spec, and the kinds
capability depends on it: a row whose kind cell becomes an arbitrary new id is
no longer a row of 𝒦, so lint, coverage, and model extraction all change
meaning. Unlike a dangling link, this corruption leaves the post-rename corpus
parseable and lint-clean, so it can survive the verify gate silently.

## Check mode

`assert`. Universal over generated `(corpus, local id, new local id)` triples,
with the generator deliberately including local ids that equal a kind keyword.

## Public API / seam

`rename::run` → `writer::apply` → `local_id_repls` (`src/acset/writer.rs:378-419`)
→ `collect_cell_repls` (`:435-457`).

`collect_cell_repls` iterates **all** entries of `Row.cells`:

```rust
for (col, value) in cells {
    if value == lo {
        ... repls.push((sp, ln.to_string())) ...
    }
}
```

There is no column filter. Constraints and Properties rows both pass their full
cell map (`writer.rs:386-391`). The rename spec describes the rewrite surface as
"structured id fields (frontmatter id, table id/traces_to/derives_from/guard/
from/to cells)" (`openspec/specs/rename/spec.md`, `prose_untouched_by_rename`),
not "every column".

## Generator sketch

- Build one spec file with the full family: several constraint rows (some
  `kind = invariant`), several property rows (some `kind = unit`).
- Choose the row to rename so that its local id is drawn from
  `{c0, invariant, effect, unit, law, ...}`. The parser imposes no id-shape rule
  on row ids, and the linter's `no_conjoined_id` / `no_universal_in_id` rules
  apply to the **Intent** id only (`src/lint/rules.rs:466-490`), so these are
  lint-clean corpora.
- Call `rename::run` with `old = "{intent}.{local}"` and
  `new = "{intent}.renamed"`.
- Oracle: parse every emitted write; for each row, compare all cells except
  `id` with the pre-rename cells. Assert the `kind` column is unchanged for
  every row, including rows other than the renamed one.

Concrete predicted-red case:

```
| id        | kind      | expr    | traces_to |
| c1        | invariant | `holds` | [[demo]]  |
| invariant | effect    | `x`     | [[demo]]  |
```

Renaming `demo.invariant` → `demo.renamed` rewrites row `c1`'s `kind` cell from
`invariant` to `renamed` as well as the target row's id cell.

## Predicted at HEAD `c76e2b0c`

**RED** (source-derived, not executed). Mechanism:

1. `local = ("invariant", "renamed")`.
2. `collect_cell_repls` sees `kind == "invariant"` in row `c1` and rewrites it.
3. `verify_roundtrip`'s table-cell oracle counts **rows whose any cell equals
   the id** (`writer.rs:576-591`). Before: row `c1` (kind) + row `invariant`
   (id) = 2. After: row `c1` (kind now `renamed`) + row `invariant` (id now
   `renamed`) = 2. The stale check passes, the count check passes.
4. `verify_would_be_repo` checks only parse + dangling links. Kind changes do
   not dangle, so the rename returns `Ok`.

The probability is generator-dependent; a uniform draw over row ids rarely
picks a keyword, so the existing suite's fixed `r{n}` / `renamed{n}`
distribution never draws it. The defect is deterministic once the id is
chosen.

## What would make it red

Already predicted red. It becomes green by restricting the rewrite to
id-bearing columns (at minimum excluding `kind`, and likely every column that is
not an id/reference column), or by making `verify_roundtrip` compare
column-by-column instead of row counts.

## Existing instrumentation

Missing. `tests/acset_writer_apply.rs:409-478` fixes `old_local = r{pick}` and
`new_local = renamed{pick}`, and builds every kind cell as the literal
`invariant`, so the keyword-collision cell is outside the input distribution.
The generated `kinds_props.rs` law only exercises identity with a materialized
fixture (out of scope, and it would not catch this).

## Investigation Log

- Examined: `writer::local_id_repls`, `collect_cell_repls`,
  `verify_roundtrip` cell counting, `spec::Row.cells`, parser row construction
  (`src/spec.rs:377-475`), lint id-shape scope (`src/lint/rules.rs:449-490`).
- Found: no column filter; row ids are not shape-restricted; lint's id rules
  target the Intent id only.
- Not done: no execution of the case. The prediction is a code-path argument;
  the property run is the missing confirmation.

## Assumptions and Open Questions

- [ASSUMPTION] A row id equal to `invariant`/`unit` is a legitimate corpus for
  the property. If the product intends to forbid such ids, the fix is a new
  lint rule, and the property would then document that exclusion instead.
- Open: is the any-cell rule intentional? If yes, the invariant should be
  narrowed to "kind cells are excluded" and the code changed accordingly.
  (needs human input)