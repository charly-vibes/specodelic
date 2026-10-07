---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-bf5-ss5-1-5-2-opaque-kernel-binding-extraction-and-claim-carrier, pipeline-step:red]
---

RED: command='cargo test --test kernel_binding'; expected failure=6/7 tests fail with 'constraint b1 carries no binding field' — kernel.binding not extracted into the constraints TOML artifact. The 7th (no_binding_column_compiles_byte_identically) passes trivially pre-implementation, correct for a pure-widening assertion. All failures are the intended missing behavior, not setup breakage (fixtures parse and compile; only the binding field is absent).
