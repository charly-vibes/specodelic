# Dogfood notes — add-graph-views (task 4.1–4.3 evidence)

Recorded 2026-10-08 by the gre.10 run (specodelic-gre.10). Out-of-CI manual
checks per task 4.2; the repo binary used was built from the landing commit
of gre.9 (all four views + `explain graph-views` present).

## 4.1 — this repo's corpus

- `spk graph specs --json`: 1008 edges, **0 violations, 0 dangling** — the
  corpus is honestly clean since the acset-core landing, and the rendered
  views carry zero annotated elements. The violation-rendering path is NOT
  assumed: it is exercised by the checked-in fixture corpora
  (`tests/fixtures/typing_violations`, `lint_dirty`) pinned by the 2.3
  consumer tests (gre.6) and the prose-perturbation purity test
  `views_from_artifact_only` (gre.8).
- `spk graph specs --view wiring --format mermaid`: renders the real
  producer→consumer file graph (lint → spec files).
- `just docs-graphs`: regenerates `docs/src/views/{schema,states,trace,wiring}.md`
  deterministically; clean-tree assertion holds; `just docs-build` embeds
  all four views in `graph-views.html`.
- `spk explain graph-views`: appended primer topic renders (append-only
  numbering, pinned by `explain_bare_lists_topics_append_only`).

## 4.2 — sibling corpus `../bajan/specs` (read-only)

- Binary run over the sibling corpus with **zero corpus-specific code
  edits** (only the tool binary + CLI flags).
- 7 files, 231 edges, **1 typing violation**, 0 dangling.
- The wiring view renders **4 real inter-file `constraints.satisfies`
  edges** (contradiction.review → graph.model, entity.review → graph.model,
  eval.claims → extraction.claims, query.tools → graph.model) — confirming
  the 5qj decision-note update: the "wiring empty until ac8" contingency is
  stale; bajan typed its wiring in ac8.
- The single violation: `contradiction.review.cr_human_resolution`
  --satisfies--> `graph.model.gm_contradicts_provenance` (target is an
  `invariant` Constraint, not an `extension_point`). This is real corpus
  feedback **for the sibling**, recorded here only — no sibling ticket was
  opened or closed by this run (sibling-tool constraint).

## 4.3 — bajan-ac8 status

- `bajan-ac8` ("Type the extension-point wiring in specs — prose-only
  wiring is invisible to spk graph") is **CLOSED** (closed 2026-09-30 in the
  sibling repo's own tracker), which is exactly why 4.2's wiring view now
  renders real edges. Cross-referenced here; status recorded, not modified.