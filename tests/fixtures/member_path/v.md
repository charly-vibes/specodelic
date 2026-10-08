---
id: v
kind: intent
statement: "member path repro — the mem row's link resolves through the member arm to the label-qualified v.row (deep), which must intern canonically as v.row"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| row | invariant | `true` | [[v]] |
| mem | invariant | `true` | [[v.row.deep]] |
