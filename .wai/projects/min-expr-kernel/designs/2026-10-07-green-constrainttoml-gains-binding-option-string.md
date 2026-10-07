---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-bf5-ss5-1-5-2-opaque-kernel-binding-extraction-and-claim-carrier, pipeline-step:green]
---

GREEN: ConstraintToml gains binding: Option<String> (serde skip_serializing_if None — pre-Revision artifacts byte-identical); constraints_doc extracts cells.get("kernel.binding") for invariant-kind rows only, backtick fence stripped per kernel_cell_content precedent, contents never parsed/validated. Narrow: cargo test --test kernel_binding → 7/7; CLI surface: tests/cli/parse_misc::compile_cli_surfaces_binding_in_the_toml_artifact → ok (ships minimal kind:profile pack id 'kernel' — the orphan_vocabulary sanctioned remediation for the dotted column header). just test → all green (32 result groups).
