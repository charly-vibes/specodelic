# Claim-guidance coverage checklist

Declaring this manifest is the coverage-lint turn-on for the
external-claim guidance (the established `*.checklist.md` mechanism,
mp1 row 10 — see `linter-external_completeness.md`): once present, the
`linter.external_completeness` rules check that every item below stays
explicitly `covered` by real constraint rows. This follows the
language-tag change precedent — a change turns coverage on for the
guidance files it touches (add-min-expr-kernel §5.3, specodelic-ats).

## Items

- **commands_claim_path**: docs/src/commands.md documents the `kernel.binding` claim path beside the two-binding-layers passage — compile extracts the column verbatim, an external checker claims constraints through contract-TOML `flags`, specodelic never interprets the binding text (design D4)
- **usage_claim_path**: specs/USAGE.md documents the external-checker claim path in §2.6 — a checker conforms without editing your file, through its own contract bindings, with the binding column carried as an opaque string

## Mapping

| item | status | mapped_ids | rationale |
|------|--------|------------|-----------|
| commands_claim_path | covered | [[compile.constraint_table_to_toml]] | |
| usage_claim_path | covered | [[compile.constraint_table_to_toml]] | |
