---
reviews: 2026-10-07-red-confirmed-pre-edit-specodelic-ats-ss5-3-jus.md
verdict: pass
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-ats-add-min-expr-kernel-ss5-3-external-claim-guidance-binding-path-coverage-lint-turn-ons-no-contract-tomls, pipeline-step:ro5u-review]
---

RO5U review: specodelic-ats §5.3 — diff 44ae671 (docs guidance + coverage turn-on)

Critical: none.

High: none.

Medium 1 — checklist mapped_ids anchor: specs/claim-guidance.checklist.md
maps both items to [[compile.constraint_table_to_toml]] (the deployed
constraint the kernel.binding widening extends). The kernel.binding
requirement itself is still change-phase; linter-external_completeness
only resolves mapped_ids inside the linted specs/ tree, so this is the
only resolvable deployed anchor today. Deferred with reason: re-anchor
after the archive (specodelic-bxk) if the requirement deploys as its
own row.

Low 1 — specs/CHANGELOG.md entry: not added; the file is outside the
ticket's hard-scope allowed list, and this is a guidance-only change
(not a corpus Revision). Orchestrator may add a line at wrap-up.

Accuracy: verified against landed §5.1/5.2 behavior (ConstraintToml
binding Option<String>, invariant-kind rows only, opaque verbatim
contents, backtick fence per kernel_cell_content precedent) and design
D4 wording (no registry, namespaced, bridge-never-absorb). Scope guard
clean: no .espectacular/, src/, tests/, openspec/specs/ edits; tasks.md
touched only for the §5.3 checkbox. Negative guard green (0
kernel.binding references under .espectacular/). just lint-specs green
with checklists_declared 0→1, 0 issues, no new advisory class; just
test 731 passed 0 failed; just ci green (exit 0); openspec validate
add-min-expr-kernel --strict valid; ah check 0 findings.

