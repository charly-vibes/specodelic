---
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-mlc-consolidate-the-outbound-leaf-carve-out-into-one-normative-statement, pipeline-step:green]
---

GREEN-FIX: the Revision 22 bump moved the current-revision pins, so updated the tests that assert the live revision: feedback_init (AGENTS block), migrate_upgrade (target 21->22, known range 2 to 22, --to-revision 21->22), model_check (three format_revision asserts), parse_misc and packs_fixtures (skew advisory names the corpus revision 22). Historical Revision 21 comments/tests in law_case_bodies are unchanged. All affected tests pass.
