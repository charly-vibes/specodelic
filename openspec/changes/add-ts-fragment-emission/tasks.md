# Tasks: TypeScript properties

Reviewed and ticketed at the user’s request; owner epic specodelic-aby,
blocked on l8l. Each feature cycle and separate TIDY ticket is mapped below.
Implementation has not begun; keep refactoring commits separate. Keep Purpose/Responsibilities/Rationale headers current.

## 1. TypeScript receives the same declared values
- [ ] 1.1 RED: generator-domain and deterministic artifact fixtures including
      non-BMP Unicode and signed 32-bit boundary integers. Reuse direct,
      aliased and nested mixed-choice rejection fixtures and homogeneous
      success fixtures; pin number/string/Array<T> predicate inputs.
- [ ] 1.2 GREEN: fast-check mappings and TypeScript unit-property emitter.
- [ ] 1.3 TIDY: separate ticket/commit reuses shared manifest helpers.

## 2. Property execution is complete and bounded
- [ ] 2.1 RED: CLI fixtures for passing/failing/shrunk properties, missing
      local tooling, timeout, skipped block, stale input and mixed languages.
- [ ] 2.2 GREEN: project-local Vitest adapter, structured results, process-tree
      cleanup, freshness and shared all-block gate; no package auto-install.
- [ ] 2.3 TIDY: separate ticket/commit removes duplicate runner plumbing.

## 3. Preserve prior contracts
- [ ] 3.1 Run shared generator conformance and existing Rust/Python kernel
      agreement checks; do not skip an unavailable backend in its CI job.
- [ ] 3.2 Rebase full compile snapshot after Python archive; update domain
      specs under Revision rules, docs capability matrix and SUMMARY.
- [ ] 3.3 Author scenario contracts to real tests; strict validation,
      section sync, lint, ah checks and just ci; dual-format archive only.

## Implementation ticket map

Created at the user’s request after the proposal review fixes. Checkboxes
remain unchecked until the linked behavior is implemented and verified.
Dependencies are recorded in beads; this table maps scope, not completion.

| Tasks | Ticket | Outcome |
|-------|--------|---------|
| 1.1, 1.2 | `specodelic-aby.1` | TypeScript predicates receive the shared declared value domains |
| 1.3 | `specodelic-aby.2` | Keep TypeScript property manifests aligned with other languages |
| 2.1, 2.2 | `specodelic-aby.3` | Mixed Rust Python and TypeScript runs verify only when complete |
| 2.3 | `specodelic-aby.4` | Keep mixed-language execution outcomes consistent during cleanup |
| 3.1, 3.2, 3.3 | `specodelic-aby.5` | TypeScript support preserves the deployed Python and kernel contracts |
