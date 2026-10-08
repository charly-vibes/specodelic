---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-ils-view-scripts-b1-hint-routing-b2-mermaid-escaping-b4-docstring, pipeline-step:plan]
---

specodelic-ils view scripts — TDD plan:

DESIRED BEHAVIOR
- B1: scripts/graph_views.py artifact_cli routes the remediation hint by
  exception type, not by substring: ArtifactInvalid (including the lint
  envelope 'lint envelope carries no issues list' refusal from
  view_common._lint_leg_refusal) gets ARTIFACT_HINT; OutOfScopeRefused
  legs keep INTENTLESS_HINT (intentless) / LINT_HINT (lint-dirty or
  lint-envelope-unverifiable). The lint-dirty LINT_HINT stays for
  OutOfScopeRefused whose message names lint invariants.
- B2: mermaid_escape (scripts/view_common.py; mirror src/graph.rs
  mermaid_escape if it shares the gap) escapes the full label-injection
  set: double quotes, Mermaid statement terminators/broken tokens and
  comment/hyperlink starters that can appear in spec-cell text
  (" --> # %% " and a line starting end). Escaping is deterministic
  text output (design D8).
- B4: tests/kernel_grammar.rs docstring drops the 'marker-free
  whole-cell opt-in' contradiction; comment-only, matching the spec
  (kernel_expr_opt_in per-cell **kernel:** marker) and the tests.

OUT OF SCOPE
- B3 fan-in disagreement (own design ticket), B5 changelog naming
  (specodelic-s64), specs/ corpus edits, openspec/, .espectacular/.

TEST CASES TO WRITE
- Python unit tests (scripts/test_view_common.py or test_graph_views.py):
  (1) ArtifactInvalid carrying 'lint envelope carries no issues list'
      routes to ARTIFACT_INVALID + ARTIFACT_HINT in artifact_cli stderr;
  (2) lint-dirty OutOfScopeRefused keeps LINT_HINT; intentless refusal
      keeps INTENTLESS_HINT;
  (3) mermaid_escape: text containing --> , # , %% , and end-line
      starting text renders inert in a mermaid label context (asserted
      as exact escaped output, D8);
- Rust unit tests in src/graph.rs test module (if mirroring B2):
  mermaid_escape on the same injection cases.

NARROW TEST COMMAND
- TMPDIR=/var/tmp/spk-ils python3 -m unittest discover -s scripts
- cargo test --test kernel_grammar (comment-only B4 — no cargo test needed)

FULL VERIFICATION
- just ci (includes views-purity python suite + cargo fmt/clippy/tests)

FILES LIKELY TOUCHED
- scripts/view_common.py, scripts/graph_views.py, scripts/test_view_common.py
  (new/updated), src/graph.rs + its test module (B2 mirror only if shared
  gap confirmed), tests/kernel_grammar.rs (comment-only B4)
