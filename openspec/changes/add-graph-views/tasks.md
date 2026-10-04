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
      in the flag's help. If full reason text is chosen, pin reason
      strings tab-free (or escaped) so the six-column TSV contract holds.
- [ ] 1.3 **RED→GREEN**: violation annotation rows — fixture corpus with
      known typing violations; assert one annotation row per violation and
      zero on a clean corpus (`violations_survive_projection`,
      `clean_corpus_no_annotations`).
- [ ] 1.4 **TIDY**: extract projection formatting into a testable unit in
      `src/graph.rs`; dead-flag and clippy sweep (`just ci`).

- [ ] 1.5 **RED→GREEN**: native dot/mermaid projections — `--format dot`
      and `--format mermaid` emit plain-text graph output (zero external
      crates; byte-stable re-runs; visual grammar: solid = state machine,
      dashed = guards, bold = `emits`, dotted = traceability, red dashed =
      dangling/violations). Parity fixture: the retired
      `scripts/graph_to_dot.jq` output pinned as the expected dot shape.
- [ ] 1.6 **RED→GREEN**: `--view wiring` — file-level producer→consumer
      projection of `constraints.satisfies` edges (specodelic-5qj
      decision, `openspec/research/2026-10-01-wiring-view-decision/`);
      self-loops dropped; corpora with zero satisfies edges emit a labeled
      `no_wiring` view, never a silently clean diagram.

## 2. Guide JSON and transform prototype (`scripts/graph_views.py`)

- [ ] 2.1 **RED**: CLI test asserting `spk guide --json` serves the
      closed value sets (kinds, row shapes, reference fields with allowed
      targets) plus `format_revision` as a JSON envelope. Run — must fail
      (no such command).
- [ ] 2.2 **GREEN**: implement the `guide --json` subcommand in
      `src/main.rs` serializing `src/guide.rs`'s constants (kinds, row
      shapes, format_revision — for value-set-only consumers; the schema
      view does NOT consume this, per D4's 2026-10-04 revision).
- [ ] 2.3 **RED**: script tests against three checked-in fixture corpora —
      empty (zero spec files = intentless: asserts `out_of_scope_refused`,
      exits non-zero with a remediation hint), single-intent
      (`single_intent_sane`), violation-bearing — plus a lint-dirty
      fixture for the other `out_of_scope_refused` leg. Assert: parse of
      the TSV contract, valid Mermaid output, violations rendered as
      annotated elements.
- [ ] 2.4 **GREEN**: implement per-file state-machine view (transition
      edges grouped by owning file; guards annotated; files without
      transitions skipped cleanly; fan-in counts distinct targets per
      D2's multiplicity rule).
- [ ] 2.5 **GREEN**: implement file-level traceability view (collapse to
      intents, fan-in annotation).
- [ ] 2.6 **GREEN**: implement schema view from the acset `Schema`
      value (`acset::schema::canonical()` — lint-gated against
      `specs/specodelic.md` by `schema_matches_typing_table`), labeled
      with `guide::FORMAT_REVISION` (D4 as revised 2026-10-04).
      Derivation proof: render two constructed `Schema` values differing
      in one morphism — the views differ, no renderer edit (the shipped
      `canonical()` is compile-time, so the perturbation test uses
      constructed values, not a fixture doc).
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
- [ ] 3.4 `spk explain graph-views` primer topic (appended at the end of
      the topic list, never renumbered): the view taxonomy, format flags,
      and one-pipe render recipes (`| dot -Tsvg`, `graph-easy` for ASCII
      terminal, mermaid paste targets); docs pages updated.

## 4. Dogfood and follow-ups

- [ ] 4.1 Run the full pipeline over this repo's corpus; confirm the
      rendered views are honestly clean — the corpus is at 0 typing
      violations since the acset-core landing (was 38 when drafted), so
      zero annotated elements must appear AND the violation-rendering
      path must be exercised by the fixture corpora (2.3), not assumed.
- [ ] 4.2 Run the pipeline over `../bajan/specs` (out-of-CI, manual check
      recorded in the change notes): views derive with zero
      corpus-specific code. Update vs the 5qj decision note: bajan now
      carries 4 typed inter-file `satisfies` edges (the "wiring empty
      until ac8" contingency is stale) — the wiring view renders for
      real there.
- [ ] 4.3 ~~File the bajan corpus-feedback beads issue~~ **done ahead of
      implementation — bajan-ac8 filed 2026-09-29 (issue-review pass); the
      implementer only verifies it's still open and cross-references it**.
- [ ] 4.4 Verify `tasks.md` all checked; `just ci` and
      `openspec validate add-graph-views --strict` pass.
