---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-ats-add-min-expr-kernel-ss5-3-external-claim-guidance-binding-path-coverage-lint-turn-ons-no-contract-tomls, pipeline-step:red]
---

RED: specodelic-ats — §5.3 (TIDY). Desired behavior not yet in place. Evidence
the gates will fail until the docs land: (1) coverage-lint turn-on rows for the
affected guidance files are absent — `grep -rn 'specodelic-ats\|coverage' scripts/guards/*.checklist docs/src/llms/ llm.txt` shows no coverage turn-on
rows referencing the external-claim guidance files; (2) the claim path next to
existing espectacular binding docs (language-tag change precedent) is not
documented in the narrative guidance files; (3) negative guard already in place
(no contract TOMLs for claim guidance — bf5's established grep guard, not
regressed by this task). Test cases (from plan artifact
.wai/plans/2026-10-07-plan-specodelic-ats-ss5-3-tidy-desired-behavior.md):
(1) just lint-specs passes with coverage lint turned on for the affected
guidance files; (2) grep guard — no *.toml under .espectacular/contracts
referencing claim guidance; (3) just test green; (4) openspec validate
--strict add-min-expr-kernel. Narrow: just lint-specs. Full: just test &&
openspec validate --strict add-min-expr-kernel.
