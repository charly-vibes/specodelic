---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-ils-view-scripts-b1-hint-routing-b2-mermaid-escaping-b4-docstring, pipeline-step:plan]
---

TDD plan for specodelic-ils (B1+B2+B4 only; B3/B5 out of scope):

DESIRED BEHAVIOR
- B1: artifact_cli hint routes by exception type — ArtifactInvalid always gets ARTIFACT_HINT (or INTENTLESS_HINT for OutOfScopeRefused); only a genuine lint-gate refusal (lint-dirty / lint envelope cannot verify) gets LINT_HINT. No substring matching on str(error).
- B2: mermaid_escape (scripts/view_common.py) neutralizes label-breaking sequences in quoted Mermaid labels: quotes (already), '#', '-->' arrows, '%%' comments, and a label that would start a line with 'end' (mermaid reserved word). Mirror the same escape into src/graph.rs mermaid_escape (it emits quoted labels too). Deterministic text (D8).
- B4: tests/kernel_grammar.rs docstring corrected to marker-based opt-in (**kernel:** marker per kernel_expr_opt_in, specs/compile.md). Comment-only.

OUT OF SCOPE
- B3 fan-in disagreement, B5 changelog naming (specodelic-s64), openspec/, specs/, .espectacular/, pretender.toml, .wai/resources.
- No new Rust files; no user-facing CLI semantics beyond what escape output bytes change (docs/src/commands.md only if semantics change — they don't).

TEST CASES (write first, RED)
Python (scripts/ suite — new escape cases in views_purity_cases.py escape module; new B1 cases in state_view_scope_cases.py):
- mermaid_escape('"quoted"') unchanged from today ("&quot;").
- mermaid_escape('a # b') -> 'a # b'.
- mermaid_escape('a --> b') -> 'a &#45;&#45;&#45;&#62; b' (neutralize arrows).
- mermaid_escape('%%') -> '&#37;&#37;'.
- mermaid_escape('end') / multiline starting 'end' — neutralize leading end.
- B1: ArtifactInvalid('lint envelope carries no issues list') -> stderr carries ARTIFACT_HINT, NOT LINT_HINT.
- B1: OutOfScopeRefused('lint-dirty corpus') -> LINT_HINT (real lint-gate refusal still hints fix-invariants).
- B1: OutOfScopeRefused(intentless) -> INTENTLESS_HINT.
- B1: ArtifactInvalid(TSV shape) -> ARTIFACT_HINT.
Rust (tests/cli/ existing escape module or graph test module in src/graph.rs — the Rust mermaid_escape is pub(crate); unit tests in src/graph.rs's test module):
- same escape cases as python.

NARROW TEST COMMAND
python3 -m unittest discover -s scripts -p test_graph_views.py
TMPDIR=/var/tmp/spk-ils cargo test --test cli escape  (or just test-smart)

FULL VERIFICATION
python3 -m unittest discover -s scripts
just ci (fmt-check, clippy -D warnings, tests, release build)
pretender check before commit; grep -c 'marker-free whole-cell opt-in' tests/kernel_grammar.rs == 0

FILES LIKELY TOUCHED
scripts/view_common.py (B2 escape + docstring), scripts/graph_views.py (B1), scripts/views_purity_cases.py + scripts/state_view_scope_cases.py (new tests), src/graph.rs (B2 mirror + rust unit tests in existing test module), tests/kernel_grammar.rs (B4 comment-only)
