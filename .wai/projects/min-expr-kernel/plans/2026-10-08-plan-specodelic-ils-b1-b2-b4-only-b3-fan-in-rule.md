---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-ils-view-scripts-b1-hint-routing-b2-mermaid-escaping-b4-docstring, pipeline-step:plan]
---

PLAN specodelic-ils (B1+B2+B4 only; B3 fan-in rule and B5 changelog naming OUT OF SCOPE).

Desired behavior:
- B1: artifact_cli routes the remediation hint by exception TYPE, not substring — an ArtifactInvalid mentioning 'lint' (the lint-envelope carries-no-issues-list refusal) gets ARTIFACT_HINT; OutOfScopeRefused gets INTENTLESS_HINT; only an OutOfScopeRefused that is lint-dirty gets LINT_HINT. Discriminator: isinstance checks on the exception computed one line above; keep ARTIFACT_INVALID failure code for ArtifactInvalid.
- B2: mermaid_escape (scripts/view_common.py + mirror src/graph.rs) escapes the quote-breaking cases: ", -->, #, %%, and a label's leading end. Deterministic, same output for already-safe text (no double-escaping).
- B4: tests/kernel_grammar.rs header docstring drops 'marker-free whole-cell opt-in', states the marker-based kernel_expr_opt_in contract; comment-only.

Test cases to write (new unit tests in scripts/test_graph_views.py — hint routing; escape cases; and Rust unit tests in src/graph.rs test module for escape cases):
- hint routing: ArtifactInvalid('lint envelope carries no issues list') → hint contains 'matching tool version' (ARTIFACT_HINT) and failure code artifact_invalid; OutOfScopeRefused intentless → 'zero spec files' hint; OutOfScopeRefused lint-dirty → 'fix the reported invariants' hint.
- escape: label with --> stays one line / not read as arrow; label with # not comment; label with %% not directive; label starting end kept as text; label with " escaped; label already safe unchanged (byte-identical).
- B4 meter: grep -c 'marker-free whole-cell opt-in' tests/kernel_grammar.rs == 0.

Narrow test commands:
- python3 -m unittest discover -s scripts -p 'test_graph_views.py'
- cargo test mermaid_escape (Rust unit tests)
Full verification: just ci (includes sync-sections-test python suite + pretender check + openspec-validate).

Files likely touched: scripts/graph_views.py, scripts/view_common.py, scripts/test_graph_views.py, src/graph.rs (mirror + Rust unit tests), tests/kernel_grammar.rs (comment-only). Never touched: openspec/specs/, specs/ corpus, .espectacular/, pretender.toml, .wai/resources/.
