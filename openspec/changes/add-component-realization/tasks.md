# Tasks — add-component-realization

Each numbered step is one red→green cycle; refactors are separate commits.
`just test-smart` during iteration; `just ci` before merge. TDD-ro5
pipeline governs implementation when it starts.

## 1. Schema — the component row kind

- [ ] 1.1 **RED**: fixture corpus with a `component` row → `spk lint` fails
  (unknown row kind), naming the kind.
- [ ] 1.2 **GREEN**: `kinds/spec.md` declares the kind (fields, closed
  level set `{context, container}`, `realizes` typing row, compatible
  target kinds); parser accepts well-formed rows.
- [ ] 1.3 Contract tests: `level: code` refused naming the closed set +
  stale-levels remediation; undeclared field → `row_shape_closed`; Revision
  23 bump lands with `AGENTS.md` block refresh (`spk init --force`).

## 2. Realization edges — resolution and typing

- [ ] 2.1 **RED**: dangling `realizes` target → no finding today.
- [ ] 2.2 **GREEN**: `total_refs` resolves corpus-wide for `realizes`;
  `ref_kind_compatible` rejects incompatible targets naming compatible
  kinds; `no_self_ref` covers self-realization.
- [ ] 2.3 **RED→GREEN**: `acyclic` union includes `realizes` — a two-node
  cycle is a finding naming members.
- [ ] 2.4 Component rows reach their own intent (`single_root_reachable`
  via `traces_to`); cross-file-only components warn per the tiered rule.

## 3. Completeness — realized-or-external

- [ ] 3.1 **RED**: container component, no `realizes`, no marker → no
  finding today.
- [ ] 3.2 **GREEN**: `component_realized_or_external` finding names the
  component and both remediations; `external: true` context actor passes;
  unreadable level/external values fail closed.

## 4. Projection and view

- [ ] 4.1 **RED→GREEN**: `spk graph --format edges` emits component nodes
  and realization edges; byte-stability contract test on the fixture corpus
  (rustfmt-style byte-identity, per the compile.rs precedent).
- [ ] 4.2 **RED→GREEN**: build-time C4 view (levels 1–2) rendered from the
  artifact alone; digest-stamped; byte-identical re-render; existing views
  unchanged ( characterization test asserts prior views byte-identical).
- [ ] 4.3 Unrealized container renders annotated (dashed), never silent;
  levels-3–4 request refused naming the closed set + advisory lane.

## 5. Self-hosting and docs

- [ ] 5.1 `specodelic` capability declares its own components (context:
  packs, sibling tools as external; containers: kernel, linter family,
  verbs); `just lint-specs` green.
- [ ] 5.2 Contract TOMLs staged for every delta scenario under
  `.espectacular/component-realization/`; `ah check --changes
  add-component-realization` clean; sibling-blockers guard untouched
  (espectacular read-only over `openspec/` holds).
- [ ] 5.3 Docs: `USAGE.md` pattern-catalog entry (component realization);
  `CHANGELOG.md` Revision 23 entry. No "second authoritative copy" or
  independence claims in any wording (2026-11-18 amendments).

## 6. Gates

- [ ] 6.1 `just ci` green (guard-espectacular included).
- [ ] 6.2 `openspec validate --all --strict` green.
- [ ] 6.3 Archive via `just archive-change id=add-component-realization`
  (dual-format companion recipe — never stock archive).
