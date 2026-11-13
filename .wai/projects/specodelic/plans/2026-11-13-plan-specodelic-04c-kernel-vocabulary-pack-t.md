---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-04c-kernel-vocabulary-pack, pipeline-step:plan]
---

# PLAN — specodelic-04c: kernel vocabulary pack (TDD)

## Desired behavior
A workspace whose Constraints/Properties table carries a `kernel.binding`
column lints **clean** (no `orphan_vocabulary` finding) because the corpus
ships a `kind: profile` pack in namespace `kernel` — discovered by corpus
scan alone (no config, no registry, no linter carve-out). The pack:

- lives at `openspec/specs/kernel-pack/spec.md`, frontmatter
  `id: kernel.pack`, `kind: profile` (naming law: `spec.md` derives its id
  from the parent directory, `-` ⇔ `.`);
- carries the six manifest facet tables (`## Sections`, `## Kinds`,
  `## References`, `## Checkers`, `## Floors`, `## Requires`), each
  non-empty with two-column rows, `## Kinds` rows pack-qualified;
- declares the `kernel.binding` **field** in `## References` (the facet the
  vocabulary rule matches: `Manifest::vocabulary` = kinds + sections +
  references);
- pins the current base revision (`specodelic.md Revision 20`) in
  `## Requires` (no skew advisory);
- is four-layer clean (Constraints / Model / Properties), every constraint
  covered by a deriving property, every transition guarded;
- leaves every file that does not use the pack's vocabulary byte-identical
  (no activation, no finding).

## Out of scope
- No `src/**` change (the rule already accepts the pack — the gap was the
  missing declared vocabulary).
- No linter carve-out, no base closed-set narrowing, no py/ts pull-in.
- No pack *checker execution* (checkers stay declarations, honest-empty).
- No `packs/*.md` inventory change (would force pack-count prose churn on
  five doc surfaces).
- No change to the existing `openspec/specs/kernel/spec.md` capability
  (the column semantics are already specified there).
- No beads state change (the orchestrator owns bd).

## Test cases (RED first)
New file `tests/kernel_pack_vocabulary.rs`:

1. `kernel_binding_column_lints_clean_when_kernel_pack_discovered` —
   workspace fixture (under the repo's `target/`, so the git-toplevel
   anchor finds the repo corpus) carrying a `kernel.binding` column:
   exit 0, **no** `orphan_vocabulary` issue, `data.packs` contains
   `kernel.pack` (base_pin 20), and an activation warning names the pack.
   RED: exit 1 with the labeled orphan finding (candidate pack `kernel.*`).
2. `file_without_binding_column_is_untouched_by_the_pack` — a fixture
   without the column: exit 0, zero issues, and **no** warning naming the
   kernel pack (the pack does not touch files that use none of its
   vocabulary).
3. `corpus_discovers_the_kernel_pack_by_scan_alone` — linting
   `openspec/specs` lists `kernel.pack` (lifecycle `published`,
   base_pin 20) among the discovered packs and keeps the corpus at zero
   issues.

Existing pins updated: `tests/cli/lint.rs`
`lint_corpus_is_fully_clean` — `files_linted` 48 → 49.

## Narrow test command
`TMPDIR=/var/tmp/specodelic-xch1 cargo test --test kernel_pack_vocabulary`

## Full verification command
`TMPDIR=/var/tmp/specodelic-xch1 cargo test` (the orchestrator owns the
full gates); locally: `cargo run -q -- lint openspec/specs`,
`python3 scripts/check_summary_completeness.py .`,
`python3 scripts/check_lint_baseline.py openspec/specs`,
`python3 scripts/check_section_sync.py openspec`.

## Files likely touched
- `openspec/specs/kernel-pack/spec.md` (new — the pack).
- `tests/kernel_pack_vocabulary.rs` (new — the enablement proof).
- `tests/cli/lint.rs` (corpus file-count pin).
- `docs/src/SUMMARY.md` (one line — the `summary-completeness` gate
  requires every corpus spec to be linked; deviation from the brief's
  allowed-file list, recorded).
- `.espectacular/kernel-pack/*.toml` (contract TOMLs for this change's
  scenarios, if the scenarios are authored).

