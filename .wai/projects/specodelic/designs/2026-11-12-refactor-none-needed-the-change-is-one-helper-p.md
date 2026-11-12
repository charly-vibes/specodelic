---
tags: [pipeline-run:tdd-ro5-2026-11-12-specodelic-q7uo-compile-pipe-unescape, pipeline-step:refactor]
---

REFACTOR: none needed — the change is one helper plus two call sites; the constraints_doc closure cannot fold to a fn item (needs the &String -> &str deref), and no duplication or naming debt emerged. Manufacturing churn would violate the surgical-scope guard.
