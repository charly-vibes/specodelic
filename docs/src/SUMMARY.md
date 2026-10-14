# Summary

[Introduction](index.md)
[Release Status](./release.md)

---

# Getting Started

- [Installation & Quick Start](installation.md)
- [Command Reference](commands.md)
- [Which command when? — routing a task to the pipeline](decision-guide.md)
- [Glossary — the jargon, defined for a cold read](glossary.md)
- [FAQ — the questions a cold read asks](faq.md)

# Thinking in Specs

- [Specs as the design substrate — a canvas for thinking about the system](design-canvas.md)
- [Design conversations — the specodelic-design skill](design-conversations.md)

# The Format

- [specodelic — the core format](specs/specodelic/spec.md)
- [kinds — closed kind value sets](specs/kinds/spec.md)
- [packs — domain-pack mechanism](specs/packs/spec.md)
- [using packs — the practical guide](using-packs.md)

# Pipeline

- [graph](specs/graph/spec.md)
- [compile](specs/compile/spec.md)
- [model_check](specs/model_check/spec.md)
- [verify](specs/verify/spec.md)
- [rename](specs/rename/spec.md)
- [merge](specs/merge/spec.md)
- [refactor](specs/refactor/spec.md)
- [orchestrate](specs/orchestrate/spec.md)
- [conform — external oracle evidence, outside the lifecycle](conform.md)
- [errors](specs/errors/spec.md)

# Graph Views

- [Graph views — derived diagrams of the corpus](graph-views.md)

# Lint Rules

- [linter.frontmatter](specs/linter-frontmatter/spec.md)
- [linter.schema_shape](specs/linter-schema_shape/spec.md)
- [linter.ears_syntax](specs/linter-ears_syntax/spec.md)
- [linter.coverage](specs/linter-coverage/spec.md)
- [linter.graph_shape](specs/linter-graph_shape/spec.md)
- [linter.model_shape](specs/linter-model_shape/spec.md)
- [linter.referential_integrity](specs/linter-referential_integrity/spec.md)
- [linter.external_completeness](specs/linter-external_completeness/spec.md)
- [linter.failure_shape](specs/linter-failure_shape/spec.md)
- [linter.observability](specs/linter-observability/spec.md)

# Architecture

- [Architecture — the four layers and the lint pipeline](architecture.md)
- [Verification boundaries — the guarantee ladder and known limitations](verification-boundaries.md)

# Tool Relationships

- [specodelic, OpenSpec, and the `ah` companion — who owns what](tool-relationships.md)

# Worked Examples

- [worked example — reservation/order, built layer by layer, failures included](examples/worked-example.md)
- [worked example — the data.lineage pack, failures included](examples/data-lineage.md)
- [worked example — the numeric.predicates pack, failures included](examples/numeric-predicates.md)
- [worked example — the empirical.registry pack, failures included](examples/empirical-registry.md)
- [worked example — the bioimage.data pack, failures included](examples/bioimage-data.md)
- [worked example — the quant.finance pack, failures included](examples/quant-finance.md)
- [reservation-order — the finished spec](examples/reservation-order.md)
- [batch-resume — derived parallelism, worked end to end](examples/batch-resume.md)

# Capability Specs (openspec)

- [acset-core — capability](specs/acset-core/spec.md)
- [acset-writer — capability](specs/acset-writer/spec.md)
- [doctor — capability](specs/doctor/spec.md)
- [embedded-guide — capability](specs/embedded-guide/spec.md)
- [docs-site — capability](specs/docs-site/spec.md)
- [hooks — capability](specs/hooks/spec.md)
- [kernel — capability](specs/kernel/spec.md)
- [lint-findings — capability](specs/lint-findings/spec.md)
- [model-check — capability](specs/model-check/spec.md)
- [spec-integration — capability](specs/spec-integration/spec.md)
- [verification-claims — capability](specs/verification-claims/spec.md)
- [error-contract — capability](specs/error-contract/spec.md)
- [observability — capability](specs/observability/spec.md)
- [migrate — capability](specs/migrate/spec.md)
- [archive-companion — capability](specs/archive-companion/spec.md)
- [parse — capability](specs/parse/spec.md)
- [data-lineage-pack — capability](specs/data-lineage-pack/spec.md)
- [numeric-predicates-pack — capability](specs/numeric-predicates-pack/spec.md)
- [empirical-registry-pack — capability](specs/empirical-registry-pack/spec.md)
- [bioimage-data-pack — capability](specs/bioimage-data-pack/spec.md)
- [quant-finance-pack — capability](specs/quant-finance-pack/spec.md)
- [skill-pack — capability](specs/skill-pack/spec.md)
- [svc-contracts-pack — capability](specs/svc-contracts-pack/spec.md)
