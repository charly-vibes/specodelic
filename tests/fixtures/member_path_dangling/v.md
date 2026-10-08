---
id: v
kind: intent
statement: "dangling member path — v.norow is not defined, so the member arm must not fire and the link stays dangling on its labeled path"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| row | invariant | `true` | [[v]] |
| mem | invariant | `true` | [[v.norow.deep]] |
