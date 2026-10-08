---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-3-native-dot-mermaid-projections-task-1-5, pipeline-step:red]
---

RED specodelic-gre.3 task 1.5: command=cargo test --test cli parse_misc (dot/mermaid filters). Expected failure=flags absent: clap rejects --format dot|mermaid, exit code 2 on every CLI case (dot_projection_* 5 failed incl zero-file exit Some(2) vs Some(0); mermaid_projection_* 6 failed); unit-free CLI-only suite, no setup breakage. Evidence: 1 passed per filter is the pre-existing unrelated edges/multiplicity test matching the filter substring.
