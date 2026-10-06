---
tags: [pipeline-run:tdd-ro5-2026-10-06-add-min-expr-kernel-phase-3-specodelic-7ga-tier-b-kernel-core, pipeline-step:plan]
---

PLAN specodelic-7ga Tier B kernel core (tasks 3.1-3.5).

DESIGN OF RECORD — kernel v0 grammar (closed, decidable):
  expr   := atomic | '¬' expr | expr '∧' expr | '(' expr ')'
          | '∀' var '∈' Object ':' expr | '∃' var '∈' Object ':' expr
  atomic := 'resolves' '(' Morphism ')' | 'unique' '(' Morphism ')'
          | 'acyclic' '(' Morphism ')' | 'reachable' '(' Id ',' Id ',' Morphism ')'
          | term '==' term | term ('<'|'<='|'>'|'>=') term
  term   := Var | Id | Int | '|' Object '|' | Var '.' Morphism | '⊥'
Opt-in rule (sd1 fragment-position, pure widening): a Constraints expr cell is kernel-opted-in iff its stripped text begins with ∀/∃/¬/atomic-keyword+'('/'|', AND parse_citation_expr(cell) is None (slice-1 cells never re-owned), AND fragment_of is None (executable markers win). Mid-span occurrences are mentions (never scanned). Non-member atomic INSIDE an opted-in cell fails labeled naming the atomic + closed set (kernel_grammar_closed). A cell that starts with none of the opt-in tokens stays prose — the honest boundary of record.

GROUNDING (D1): acyclic→query::cyclic_nodes (graph traversal); reachable→query::forward_closure (unknown seed ⇒ unknown, never labeled); resolves→instance dangling check (acset traversal); unique→injectivity of defined morphism values (acset traversal); ==/comparisons/∀/∃→bounded evaluation over finite I(k) with Kleene rules (reuse compile::ThreeValued). Row-typing: graph::kind_index supplies object membership I(k); schema-as-data validates var.m at parse (labeled when morphism not on the bounding object).

FILES: NEW src/kernel.rs (grammar+evaluation+grounding table+registry seam, file-headers compliant); src/lib.rs (pub mod kernel); src/compile.rs (guard_kernel extraction in extract_model_ir + ModelIr field w/ serde default; labeled validation in validate_fragments path for opted-in kernel cells); src/acset/instance.rs (small accessors: object membership, morphism value by id — no ratchet risk at 924/2300). src/model_check.rs UNTOUCHED (ratchet). Tests: NEW tests/kernel_grammar.rs (RED 3.1), tests/kernel_grounding.rs (3.3), tests/kernel_status.rs (3.4 end-to-end through compile extraction + Acset evaluation).

CYCLES: 3.1 RED (grammar tests fail: no kernel module) → 3.2 GREEN (grammar+row-typing) → 3.3 GREEN (atomics+grounding) → 3.4 GREEN (three-valued end-to-end) → 3.5 TIDY (registry seam, nothing registers, D8 gate pinned by test). Gates per step: just test, just lint-specs, spk lint on both delta specs, ah check, openspec validate, cargo fmt/clippy.
