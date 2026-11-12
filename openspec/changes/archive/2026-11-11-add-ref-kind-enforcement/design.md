# Design: add-ref-kind-enforcement

## Context

`ref_kind_compatible` exists as a declared invariant in
`linter-referential_integrity/spec.md` with a declared property
(`wrong_target_kind`) and a declared scenario, but the linter has no
rule behind it. Three surfaces already promise it: RULE_TABLE prose
(`guard_required`'s semantics), `guide.rs`'s `REFERENCE_TYPING` const,
and `spk graph`'s violation report (which reads the same table as data).
The linter is the gate surface (`spk lint`, `just lint-specs`, Checker
Ownership stage `lint`) — without a rule there, kind-incompatible
references pass the gate that the graph command flags on the same tree.

## Goals / Non-Goals

- Goals: the linter fires `linter.ref_kind_compatible` on every typed
  field's kind-incompatible resolved target; the decision is
  Schema-as-data (no per-field checker code); the corpus stays green;
  graph/oracle agreement surfaces stay coherent.
- Non-Goals: widening any Reference Typing row (append-only; no row
  needs widening — corpus probe is clean); changing `spk graph`'s output
  or `TYPED_REFERENCE_COLUMNS`; demoting the table to advisory prose
  (rejected — see Decision 1); modeling `from`/`to` (already
  `every_transition_valid`'s beat); touching `guide.rs`'s
  `REFERENCE_TYPING` wording.

## Decisions

- **Decision 1: enforce, not demote; severity = error (issues channel).**
  The checker file declares the constraint as an `invariant` whose
  transition `accept` gates on it — advisory severity would contradict
  the checker's own model (advisories live on the warnings channel and
  never gate: observability, skew_advisory). Demotion would be a
  documented-behavior change across the catalog, guide, and graph
  surfaces — a different change with its own justification. Enforcement
  makes the gate match what `spk graph` already labels.
- **Decision 2: the Schema is the decision, not new match arms.**
  The rule calls `acset::schema::typing_violation(&canonical(), …)` —
  the same data-driven checker `spk graph` uses (`typing_table_is_data`,
  add-acset-core). Adding a Reference Typing row is adding a schema row;
  this rule and the graph cannot disagree about what the table means.
  Endpoint construction reuses `graph::NodeKind`'s
  `object()`/`kind_cell()`/`describe()` vocabulary so violation reasons
  are byte-identical to the graph's.
- **Decision 3: resolution mirrors `total_refs`, typing matches the
  oracle.** Skips: non-typed columns (a column with no schema rows —
  this includes frontmatter, `expr`/`predicate`/`generator` cells),
  metasyntactic dotless targets, section anchors (`model.state`,
  `model.transition`), and unresolved targets (`total_refs` fires
  there — typing a dangling target would double-report). Member-arm
  targets (`file.row.member`) canonicalize to `file.row` and are typed
  as the row: the independent oracle resolves member arms to the row's
  kind, and lint is the gate, so lint matches the stricter of the two
  existing derivations (graph skips member arms; no graph behavior
  changes).
- **Decision 4: `uses` targets type by frontmatter kind.** The schema's
  `uses` row targets an Intent with refinement `kind == profile`.
  The graph's `kind_index` maps every file id to a bare Intent and loses
  the frontmatter kind, so the rule's endpoint index records file-id
  targets as Intent with the file's own `intent.kind` (`intent`/
  `profile`). A `uses` link to a non-profile file fires; the corpus's
  pack-enablement links stay clean. `orphan_vocabulary` keeps its own
  contract (no discovered pack carries the id) — the two findings cover
  different failures and may legitimately co-fire.
- **Decision 5: corpus conformance by probe, not by editing.**
  Pre-implementation probe (2026-11-12): `spk graph openspec/specs`
  violations = 0. The cxq reconciliation (2026-09-30) already conformed
  the corpus. Should the new rule surface a violation the graph missed
  (member arms, `uses` frontmatter kinds), the corpus row is conformed
  and cited in tasks.md — widening a typing row would additionally need
  a `specodelic.md` delta in this change, which is not expected.

## Risks / Trade-offs

- Risk: double-reporting with `spk graph` violations. Accepted: lint is
  the gate, graph is the report; they answer different commands and the
  graph's `edge_kind_matches_typing` requirement is untouched.
- Risk: false positives on `uses` (a pack id that is also a row id).
  Mitigated by resolution order (exact file id first, then rows) — the
  same order `total_refs` and the graph use.
- Risk: fixture-catalog coverage test
  (`catalog_covers_every_rule_the_linter_can_emit`) forces the new rule
  to fire from `fixture_corpus()`. Planned for: one fixture fires it
  (a guard citing a non-invariant constraint), keeping the catalog
  test's invariant intact.

## Open Questions

- None. The enforce-vs-demote call is the ticket's recorded decision
  (enforce), pre-validated against the probe results.
