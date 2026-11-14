# Decision: kernel grammar is gated at compile, not lint

*Decision note, 2026-11-14 (specodelic-cbh). Owner instruction: record the
decision for the two findings in the ticket.*

## What is decided

- **`spk lint` stays grammar-agnostic for kernel cells.** Lint validates
  schema shape, reference resolution, typing, coverage, and the model —
  never the syntax of a `**kernel:**` expression.
- **`spk compile` is the kernel-grammar gate.** `parse_kernel_str` /
  `grammar_failures` reject an empty marker, an unclosed parenthesis, a
  non-member atomic (`∨`, `forall c in Constraint: …`), a wrong morphism
  argument (`resolves(nonexistent_col)`), and a non-endo `acyclic` call
  (`acyclic(traces_to)`) with a labeled failure naming the atomic and the
  closed set. `spk model-check`'s `kernel_grammar` pre-check inherits the
  gate inside `just ci`; `spk verify` never re-compiles and sees a
  kernel-cell edit only as stale evidence.
- **Contributor guidance:** a lint-clean kernel cell is not compile-valid.
  Gate on `just ci` (whose `model-check` stage runs the `kernel_grammar`
  pre-check), or run `spk compile` directly; do not treat `spk lint` as
  proof that a kernel expression parses.
- **`TYPED_REFERENCE_COLUMNS` is derived, not hardcoded.** The graph's
  typed-reference set is computed from
  `acset::schema::canonical`'s morphisms minus the explicit non-graph
  columns `from`, `to` (the transitions walk owns them) and `uses`
  (omitted from `graph.md`'s `total_extraction` enumeration; an outbound
  leaf per `packs.md`'s `uses_is_outbound_leaf`). A new Schema column
  is therefore extracted automatically; if a column must stay out, it
  joins `NON_GRAPH_COLUMNS` and the
  `typed_reference_columns_derive_from_schema` gate makes that choice
  explicit. Graph output is byte-identical (the derived set is exactly
  the previous seven columns).

## Why

The kernel parser is the single grammar authority. Duplicating its
grammar in the linter would create a second, drifting definition of what
a valid kernel cell is — the exact pathology this repo's format exists to
prevent. Lint's contract is structural integrity; compile's contract is
extraction. Separating them keeps each checker decidable and
single-purpose, and the graph derivation closes the analogous
hardcoded-list drift risk on the reference side.

## Consequences and limits

- A lint-only gate can pass a malformed kernel cell. That is now
  documented in `kernel.md`'s Notes and in `openspec/specs/AGENTS.md`,
  not hidden.
- Adding a genuinely non-graph typed column requires an explicit
  `NON_GRAPH_COLUMNS` entry; the coverage test fails until that decision
  is made.
- No lint rule, parser, or graph-edge behavior changes with this decision.