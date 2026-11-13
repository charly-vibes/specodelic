---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-1nhe-law-kind-per-case-fragment-bodies, pipeline-step:red]
---

RED: command=cargo test --test law_case_bodies — 7 failed / 4 passed. Failing for the expected missing behavior: per-case extraction (the current law_case_labels regex counts the **rust:** marker as a case label — 5 blocks instead of 3), per-case emission (no body reaches a case's block), placeholder fallback (the marker pollutes the case set), the four labeled failures (no law_case_bodies rule exists; the current failure text is the unrelated 'more than one marker' / 'unknown tag' message), the empty-label floor (the marker is counted as a label), and the byte-identical legacy golden (kept passing). The 4 passing tests pin behavior that must survive: the marker is never a case label, a body without labels still fails law_cases, a case without a body keeps the placeholder, and the pre-Revision-21 artifact is byte-identical.
