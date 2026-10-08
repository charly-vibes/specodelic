---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-ils-view-scripts-b1-hint-routing-b2-mermaid-escaping-b4-docstring, pipeline-step:plan]
---

specodelic-ils (B1+B2+B4; B3/B5 out of scope per brief).

DESIRED BEHAVIOR
- B1: artifact_cli (scripts/graph_views.py) routes the remediation hint by exception TYPE, not substring: refused (OutOfScopeRefused) → LINT_HINT when the refusal is the lint leg ('lint envelope cannot verify' / 'lint-dirty corpus'), else INTENTLESS_HINT; artifact failures (ArtifactInvalid — including 'lint envelope carries no issues list') → ARTIFACT_HINT always. Failure label stays artifact_invalid.
- B2: mermaid_escape (scripts/view_common.py + mirror src/graph.rs — both share the quote-only gap) HTML-entity-escapes the label-breaking set: & (entity smuggling), " (quote breakout), < > (HTML tags + the -->/-.->/==> arrow grammar), # (mermaid #nn; entity codes), % (%% comment syntax); newlines flatten to spaces so a label can never start a diagram line (end closes subgraphs, %% opens comments). Deterministic bytes (D8); already-safe text byte-identical.
- B4: tests/kernel_grammar.rs header docstring drops 'marker-free whole-cell opt-in' → per-cell **kernel:** marker opt-in per kernel_expr_opt_in (specs/compile.md:33); comment-only.

OUT OF SCOPE: B3 fan-in disagreement (own design ticket), B5 changelog naming (specodelic-s64; do NOT touch specs/CHANGELOG.md), openspec/specs/, .espectacular/, pretender.toml, .wai/resources/**, state_view.py/traceability_view.py call sites.

TEST CASES (write first — RED)
Python (scripts/test_mermaid_escape.py, scripts/test_hint_routing.py — new; discovery pattern test_*.py):
- mermaid_escape('a --> b') == 'a --&gt; b' (arrow grammar inert)
- mermaid_escape('a # b') == 'a &num; b' (mermaid entity codes inert)
- mermaid_escape('a %% b') == 'a &percnt;&percnt; b' (comment syntax inert)
- mermaid_escape('a "b"') == 'a &quot;b&quot;' (existing contract pinned)
- mermaid_escape('a & b') == 'a &amp; b' (entity smuggling closed)
- mermaid_escape('end\n of line') has no newline and no line starting 'end'
- mermaid_escape('two.c1 (fan-in 2)') byte-identical (no double-escaping of safe text)
- hint routing via run_states_cli: lint envelope ok:true without issues list → stderr has artifact_invalid + ARTIFACT_HINT ('matching tool version'), NOT 'fix the reported invariants' [RED: currently gets LINT_HINT]; lint-dirty → 'fix the reported invariants' (control); intentless → 'zero spec files' (control); short TSV row → artifact_invalid + ARTIFACT_HINT (control).
Rust (src/graph.rs mod tests — the mirror's unit tests): same escape cases on mermaid_escape.

NARROW TEST COMMANDS
- TMPDIR=/var/tmp/spk-ils python3 -m unittest discover -s scripts
- cargo test --lib graph::tests::mermaid

FULL VERIFICATION
- just ci (fmt-check clippy test build-release openspec-validate sync-sections-test summary-completeness lint-doc-examples lint-baseline pretender-check guards)
- pretender check before commit (shrink-only; src/graph.rs 1456/2300 lines has headroom)

FILES LIKELY TOUCHED
scripts/view_common.py (B2), scripts/graph_views.py (B1), scripts/test_mermaid_escape.py + scripts/test_hint_routing.py (new tests), src/graph.rs (B2 mirror + mod tests cases), tests/kernel_grammar.rs (comment-only B4).
