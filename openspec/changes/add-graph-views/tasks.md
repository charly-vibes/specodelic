# Tasks: add-graph-views

Each phase is one red→green→refactor cycle; the failing test is written
and observed to fail before the implementation that makes it pass.
Tidying commits are separate from feature commits.

## 1. Edge-list projection (`--format edges`)

- [ ] 1.1 **RED**: CLI tests in `tests/cli.rs` for the projection contract:
      sorted TSV shape (six columns), byte-identical re-runs, endpoints are
      frontmatter ids only (assert no label-qualified endpoints using a
      fixture corpus that triggers them), zero-file directory exits 0 with
      empty output, single-intent corpus yields a well-formed row set.
      Run `just test` — all new tests must fail (flag does not exist).
- [ ] 1.2 **GREEN**: canonical-id normalization (D2) — normalize
      label-qualified nodes at or before projection in `src/graph.rs`;
      add `--format edges` to the `Graph` command in `src/main.rs`
      emitting the TSV through the envelope conventions (raw TSV to
      stdout, overriding `--json`/`--human` per D3's flag-precedence
      rule). Decide the annotation column content — full reason text vs
      class code (D3 open question) — against real output and document it
      in the flag's help.
- [ ] 1.3 **RED→GREEN**: violation annotation rows — fixture corpus with
      known typing violations; assert one annotation row per violation and
      zero on a clean corpus (`violations_survive_projection`,
      `clean_corpus_no_annotations`).
- [ ] 1.4 **TIDY**: extract projection formatting into a testable unit in
      `src/graph.rs`; dead-flag and clippy sweep (`just ci`).

## 2. Guide JSON and transform prototype (`scripts/graph_views.py`)

- [ ] 2.1 **RED**: CLI test asserting `spk guide --json` serves the
      closed value sets (kinds, row shapes, reference fields with allowed
      targets) plus `format_revision` as a JSON envelope. Run — must fail
      (no such command).
- [ ] 2.2 **GREEN**: implement the `guide --json` subcommand in
      `src/main.rs` serializing `src/guide.rs`'s constants; schema-view
      fixture test consumes its output.
- [ ] 2.3 **RED**: script tests against three checked-in fixture corpora —
      empty (`empty_corpus_projection`), single-intent
      (`single_intent_sane`), violation-bearing — plus the scope gate
      (`out_of_scope_refused`: corpus failing lint or intentless exits
      non-zero with a remediation hint). Assert: parse of the TSV
      contract, valid Mermaid output, violations rendered as annotated
      elements, exit 0 on empty input.
- [ ] 2.4 **GREEN**: implement per-file state-machine view (transition
      edges grouped by owning file; guards annotated; files without
      transitions skipped cleanly; fan-in counts distinct targets per
      D2's multiplicity rule).
- [ ] 2.5 **GREEN**: implement file-level traceability view (collapse to
      intents, fan-in annotation).
- [ ] 2.6 **GREEN**: implement schema view from `spk guide --json`
      output, labeled with the format revision (D4). Assert the view
      changes when a fixture guide gains a field — no renderer edit.
- [ ] 2.7 **TIDY**: shared rendering helpers; prose-independence test
      (`views_from_artifact_only`: perturb prose blocks of a fixture, view
      output byte-identical).

## 3. Build wiring and docs

- [ ] 3.1 `just docs-graphs` recipe: regenerate all views into
      `docs/src/views/` (gitignore it first — D6); assert `git status`
      clean after a full build (RED first: fails while recipe absent).
- [ ] 3.2 Docs page under `docs/src/` consuming the generated includes:
      the views for this repo's own corpus plus the revision-labeled
      schema view; one sentence of philosophy — views are never more
      current or more correct than the graph artifact.
- [ ] 3.3 Wire `docs-graphs` into the docs build path (`justfile`); do NOT
      add it to `just ci` gates in v1 (rendering is build-time only).

## 4. Dogfood and follow-ups

- [ ] 4.1 Run the full pipeline over this repo's corpus; eyeball the ~40
      annotated violations in the rendered views — confirm nothing reads
      as silently clean.
- [ ] 4.2 Run the pipeline over `../bajan/specs` (out-of-CI, manual check
      recorded in the change notes): views derive with zero
      corpus-specific code.
- [ ] 4.3 ~~File the bajan corpus-feedback beads issue~~ **done ahead of
      implementation — bajan-ac8 filed 2026-09-29 (issue-review pass); the
      implementer only verifies it's still open and cross-references it**.
- [ ] 4.4 Verify `tasks.md` all checked; `just ci` and
      `openspec validate add-graph-views --strict` pass.
