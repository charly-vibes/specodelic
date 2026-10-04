---
id: small
kind: intent
statement: "A minimal clean baseline corpus that triggers NO violation, NO dangling, NO cycle."
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| clean_row | invariant | `true` | [[small]] |
