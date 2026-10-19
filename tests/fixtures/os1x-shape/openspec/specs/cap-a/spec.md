---
id: cap.a
kind: intent
statement: "WHEN x THEN THE format SHALL y"
---
# cap-a

## Purpose

cap-a ships the capability that demonstrates the deployed-spec shape end to end.

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| c1 | invariant | `x` | [[cap.a]] |

## Requirements

### Requirement: One holds
The system SHALL hold one when x holds.

#### Scenario: One
- **WHEN** x
- **THEN** y
