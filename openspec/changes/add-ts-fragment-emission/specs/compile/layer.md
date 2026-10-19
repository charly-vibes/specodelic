---
id: compile.layer
kind: intent
statement: "WHEN an expr cell opts into kernel translation, THE toolchain SHALL extract the expression under the closed kernel grammar per cell, SHALL fail labeled on grammar violations, SHALL compile prose expr cells byte-identically to before, and SHALL extract the kernel.binding cell as an opaque claim carrier the toolchain never interprets."
---

## Constraints

| id                       | kind      | expr                                                                                                                                                                                                                                                              | traces_to |
|--------------------------|-----------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| process_lifecycle | invariant | `the capability advances through its declared lifecycle states under the repo's change process — each stage transition fires only when its stage gate holds` | [[compile.layer]] |
| kernel_opt_in_extracted  | invariant | `an expr cell may carry a kernel expression in fragment position per cell — the fragment-position rule (specodelic-sd1) carries over: an occurrence outside fragment position is a mention and never extracts`                                                          | [[compile.layer]]  |
| kernel_grammar_violation_labeled | invariant | `an expr cell whose kernel expression uses a non-member atomic is a labeled extraction failure naming the atomic and the closed set — never treated as prose, never extracted, never silently ignored`                                                               | [[compile.layer]]  |
| prose_back_compat        | invariant | `unmarked legacy generator cells retain emitted bytes; unchanged Rust and prose cells retain extraction; explicit generator opt-ins use typed semantics and are excluded from artifact byte preservation`                               | [[compile.layer]]  |
| kernel_binding_extracted | invariant | `an invariant-kind Constraint's kernel.binding cell extracts as an opaque string, surfaced verbatim and never interpreted; the claim path for external checkers is contract-TOML flags, not a toolchain registry`                                                     | [[compile.layer]]  |

## Model

### States
- `prose_expr`
- `kernel_expr`
- `claimed`

### Transitions

| id        | from        | to         | guard                                                                           |
|-----------|-------------|------------|---------------------------------------------------------------------------------|
| opt_in    | prose_expr  | kernel_expr | `an expr cell carries a kernel expression per [[compile.layer.kernel_opt_in_extracted]]`  |
| reject    | prose_expr  | prose_expr | `a non-member atomic fails labeled per [[compile.layer.kernel_grammar_violation_labeled]], cell unchanged`  |
| claim     | kernel_expr | claimed    | `an external checker claims the constraint per [[compile.layer.kernel_binding_extracted]]`  |

## Properties

| id                        | kind | derives_from                             | generator                              | predicate                                                                  |
|---------------------------|------|------------------------------------------|----------------------------------------|----------------------------------------------------------------------------|
| process_lifecycle_checked | unit | [[compile.layer.process_lifecycle]] | `lifecycle_model_present()` | `check(file) == passed` |
| kernel_expr_extracts      | unit | [[compile.layer.kernel_opt_in_extracted]]         | `expr_cell_with_kernel_expression()`   | `expression extracted under the closed grammar with position rule intact`   |
| nonmember_atomic_rejects  | unit | [[compile.layer.kernel_grammar_violation_labeled]] | `expr_cell_with_nonmember_atomic()`   | `extraction fails labeled, naming the atomic and the closed set`            |
| prose_unchanged           | unit | [[compile.layer.prose_back_compat]]               | `pre_revision_unmarked_generator_fixtures()` | `legacy artifact byte-identical; marked generator migration tested separately`   |
| midspan_mention_ignored   | unit | [[compile.layer.kernel_opt_in_extracted]]         | `cell_with_midspan_kernel_marker()`    | `no expression extracted; cell compiles exactly as before`                  |
| binding_extracts_verbatim | unit | [[compile.layer.kernel_binding_extracted]]        | `constraint_with_arbitrary_binding_text()` | `binding cell contents extracted and surfaced verbatim, never interpreted` |

