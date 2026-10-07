---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-bf5-ss5-1-5-2-opaque-kernel-binding-extraction-and-claim-carrier, pipeline-step:plan]
---

specodelic-bf5 §5.1-5.2: kernel.binding opaque claim carrier.
DESIRED: invariant-kind Constraint with kernel.binding cell extracts text verbatim into the compiled constraints TOML artifact as optional field 'binding' (skipped when column absent → byte-identical pre-Revision artifacts, pure widening); flows through CLI compile envelope; empty cell → empty string preserved; contents never parsed/validated/interpreted; non-invariant rows with the cell stay out; no registry built.
TESTS (new tests/kernel_binding.rs): fixtures ≥3 — ordinary text, arbitrary checker-language text (e.g. '-k scenario', '[[tests.shell]]' payloads, garbage bytes) preserved byte-exact; empty cell → binding = ""; absent column → no field (snapshot of pre-Revision bytes unchanged); non-invariant row → not extracted; CLI envelope surface via tests/cli/parse_misc.rs.
OUT OF SCOPE: .espectacular/, openspec/specs/, model_check, contract-TOML authoring (archive-time), tasks.md beyond 5.1/5.2.
NARROW: cargo test --test kernel_binding. FULL: just test-smart; orchestrator runs just ci.
FILES: src/compile.rs (ConstraintToml.binding Option + extraction in constraints_doc), tests/kernel_binding.rs (new), tests/cli/parse_misc.rs, tasks.md §5.1/5.2.
