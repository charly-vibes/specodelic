## MODIFIED Requirements

### Requirement: Dual-format delta
Every deployed capability spec under `openspec/specs/<cap>/spec.md` SHALL
be simultaneously a valid openspec capability spec and a lint-clean
specodelic spec file (frontmatter plus Constraints, Model, and Properties
sections), carrying the REAL id derived by the naming law — a `spec.md`
file takes its id from its parent directory name (`-` ⇔ `.`; Revision 18)
— and pairing its carried delta sections with a sibling `## Requirements`
mirror. A change's delta file under `openspec/changes/` is a plain
openspec delta (openspec validate is its gate; it carries no frontmatter
and no specodelic tables), so a repo needs exactly one spec vocabulary.

#### Scenario: Both parsers accept
- **WHEN** `openspec validate <change> --strict` and `spk lint <file>` run on any deployed capability spec
- **THEN** both exit 0 with zero issues

#### Scenario: Coexisting identical ids
- **WHEN** two dual-format files claiming the same intent id (the
  transitional pair: an active change's delta and its deployed capability
  spec) are linted together
- **THEN** the lint reports no unique-id violation (uniqueness is per-file);
  command paths (model-check, verify, orchestrate) refuse the same pair
  with `duplicate_corpus_identity`

#### Scenario: Modified delta is dual-format
- **WHEN** a deployed capability file carries `## MODIFIED Requirements`
  without the sibling mirror
- **THEN** `spk lint` reports the `dual_format_valid` finding, exactly as
  it does for an ADDED-carrying file
