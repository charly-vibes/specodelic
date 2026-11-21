# Tasks — add-component-realization

Each numbered step is one red→green cycle; refactors are separate commits.
`just test-smart` during iteration; `just ci` before merge. TDD-ro5
pipeline governs implementation when it starts.

## 1. Schema — the sixth object and the `realizes` morphism

- [ ] 1.1 **RED**: `specodelic::acset::schema::canonical()` has no
  `Component` object → fixture with a component row fails to parse.
- [ ] 1.2 **GREEN**: `src/acset/schema.rs` — `CLOSED_OBJECTS` gains
  `Component`; the canonical schema gains the `realizes` morphism
  (source `Component`, target `Constraint`/`Intent`) and the widened
  `traces_to` source (Constraint **and** Component, mirroring the
  `supersedes` two-source precedent). `canonical_schema_is_valid` and the
  "closed five" fixture move to six.
- [ ] 1.3 `openspec/specs/specodelic/spec.md` — new Revision heading
  (next free: 23 as of 2026-11-21) carrying: the Reference Typing row
  `realizes | Component | Constraint, or Intent`; `traces_to`'s "Appears on"
  widened to `Constraint, Component`; the sixth kind acknowledged.
  `openspec/specs/acset-core/spec.md` — `objects_closed` names six.
- [ ] 1.4 Contract tests: `level: code` refused naming the closed set +
  stale-levels remediation; `scope: shared` refused naming
  `{internal, external}` (no bool column type exists); undeclared field →
  `row_shape_closed`; `schema_matches_typing_table` green (doc table and
  Schema value agree row for row).

## 2. Kinds — reopening the closed five under the Revision heading

- [ ] 2.1 **RED**: `kinds.kind_enum_closed` rejects a `## Components`
  section → the fixture fails to lint.
- [ ] 2.2 **GREEN**: `openspec/specs/kinds/spec.md` — `component` joins
  the kind set (six kinds, six top-level sections); `component_row_shape`
  added to the row-shape family with the closed field set
  `{id, kind, level, scope, realizes}`.
- [ ] 2.3 **RED→GREEN**: `kinds.unknown_kind_rejected` — its generator
  moves from "a sixth section" to the **seventh** section (`## Actions`
  stays the rejected example; `## Components` is now legal). The property
  must not reject the corpus's own new kind.
- [ ] 2.4 `linter.schema_shape` rule table: `kinds.component_row_shape`
  registered alongside the other row-shape rules.

## 3. Realization edges — resolution, typing, cycles

- [ ] 3.1 **RED**: dangling `realizes` target → no finding today.
- [ ] 3.2 **GREEN**: `total_refs` resolves corpus-wide for `realizes`;
  `ref_kind_compatible` rejects incompatible targets naming compatible
  kinds; `no_self_ref` covers self-realization.
- [ ] 3.3 **RED→GREEN**: `acyclic` reaches the new kind — two components
  realizing each other is a finding naming the members.
- [ ] 3.4 Component rows reach their own intent via `traces_to`
  (`single_root_reachable`); a component with cross-file-only targets
  warns per the tiered rule (advisory) unless it has no path to ANY
  intent (hard-fail).

## 4. Completeness — the new rule family

- [ ] 4.1 **RED**: container component, `scope: internal`, no `realizes`
  → no finding today.
- [ ] 4.2 **GREEN**: new checker spec file
  `openspec/specs/linter-component_shape/spec.md` (`id:
  linter.component_shape`) with the `component_realized_or_external` rule;
  registered in the rule→file table at `openspec/specs/specodelic/spec.md`.
- [ ] 4.3 The rule fires on an internal container with no realization
  (naming the component and both remediations); an `external` container
  passes; a `context` component needs no realization. Fail-closed on
  unreadable `level`/`scope` values.

## 5. Projection and view

- [ ] 5.1 **RED→GREEN**: `spk graph --format edges` emits component nodes
  and realization edges; byte-stability contract test on the fixture
  corpus (byte-identity, per the compile.rs precedent).
- [ ] 5.2 **RED→GREEN**: build-time C4 view (levels 1–2) rendered from the
  artifact alone; digest-stamped; byte-identical re-render **over an
  unchanged graph artifact**; existing views unchanged (characterization
  test asserts prior views byte-identical).
- [ ] 5.3 Unrealized internal container renders annotated (dashed), never
  silent; a levels-3–4 request is refused naming the closed set — the
  remediation stays format-level (no sibling-tool names in the message).

## 6. Self-hosting, docs, and the archive sidecar

- [ ] 6.1 `specodelic` capability declares its own components (context:
  packs and sibling tools, marked external; containers: kernel, linter
  family, verbs) in a `## Components` section; `just lint-specs` green.
- [ ] 6.2 **Layer sidecar**: author `specs/component-realization/layer.md`
  (frontmatter + Constraints/Model/Properties tables) beside the plain
  delta — the archive companion refuses a sidecarless delta for a
  capability with no deployed spec to inherit a layer from
  (`archive-companion`'s `fail_closed_unverifiable`). Model the
  `add-preservation-check` sidecar's shape.
- [ ] 6.3 Contract TOMLs staged for every delta scenario under
  `.espectacular/component-realization/`; `ah check --changes
  add-component-realization` clean; sibling-blockers guard untouched
  (espectacular stays read-only over `openspec/`).
- [ ] 6.4 Docs: `USAGE.md` pattern-catalog entry (component realization);
  `CHANGELOG.md` Revision entry naming the heading it claims. No "second
  authoritative copy" or independence claims in any wording (2026-11-18
  amendments).

## 7. Gates

- [ ] 7.1 `just ci` green (guard-espectacular included).
- [ ] 7.2 `openspec validate --all --strict` green.
- [ ] 7.3 Archive via `just archive-change id=add-component-realization`
  (dual-format companion recipe — never stock archive).
